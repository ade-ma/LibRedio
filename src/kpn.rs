/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

extern crate msgpack;
extern crate native;

use native::task::spawn;
use std::comm::{Sender, Receiver, Data, Select, Handle};

use std::iter::AdditiveIterator;
use msgpack::{Array, Unsigned, Double, Value, String};

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::net::udp::UdpSocket;
use std::io::{Listener, Acceptor};


use std::io::net::unix::UnixListener;
use std::io::{Listener, Acceptor};

#[deriving(Eq, Clone)]
pub struct SourceConf {
	Freq: f64,
	Rate: f64,
	Period: f64
}

#[deriving(Eq, Clone)]
pub enum Token {
	Chip(uint),
	Dbl(f64),
	Break(&'static str),
	Dur(~Token, f64),
	Run(~Token, uint),
	Packet(~[Token]),
}

// run length encoding
pub fn rle(u: Receiver<Token>, v: Sender<Token>, s: SourceConf) {
	let mut x: Token = u.recv();
	let mut i: uint = 1;
	loop {
		let y = u.recv();
		if y != x {
			v.send(Run(~x, i));
			i = 1;
		}
		else {i = i + 1}
		x = y;
	}
}

// accept input infinite sequence of runs, convert counts to duration by dividing by sample rate
pub fn dle(u: Receiver<Token>, v: Sender<Token>, s: SourceConf) {
	loop {
		match u.recv() {
			Run(x, ct) => v.send( Dur ( x, ct as f64 / s.Rate) ),
			_ => (),
		}
	}
}

// duration length decoding
pub fn dld(u: Receiver<Token>, v: Sender<Token>, s: SourceConf) {
	loop {
		match u.recv() {
			Dur(x, dur) => v.send( Run ( x, (dur * s.Rate) as uint)),
			_ => (),
		}
	}
}

// run length decoding
pub fn rld(u: Receiver<Token>, v: Sender<Token>, s: SourceConf) {
	loop {
		match u.recv() {
			Run(~x, ct) => for _ in range(0, ct){v.send(x.clone())},
			_ => (),
		}
	}
}


// manchester 1/2 pulse duration to state matching
pub fn validTokenManchester(u: Receiver<Token>, v: Sender<Token>, s: SourceConf) {
	loop {
		match u.recv() {
			Dur(~x, dur) => {
				if ((0.7*s.Period) < dur) && ( dur < (1.3*s.Period)) { v.send(x)}
				else if (1.4*s.Period < dur) && (dur < (2.6*s.Period)) { v.send(x.clone()); v.send(x);}
				else if (dur > 3.0*s.Period) && (x == Chip(0)) { v.send(Break("silence"))};
			},
			_ => ()
		}
	}
}


// manchester state-pair to symbol decoding
pub fn manchesterd(u: Receiver<Token>, v: Sender<Token>, s: SourceConf) {
	let mut x = u.recv();
	let mut y = u.recv();
	loop {
		let msg = match (x, y.clone()) {
			(Chip(1), Chip(0)) => Chip(1),
			(Chip(0), Chip(1)) => Chip(0),
			(Break("silence"), Chip(1)) => Chip(0),
			(Chip(a), Chip(b)) if a == b => Break("manchester collision"),
			(Break(b),  _) =>  Break(b),
			(_, Break(b)) =>  Break(b),
			_ => Break("else")
		};
		if msg == Break("manchester collision") {
			x = y;
			y = u.recv();
		}
		else {
			x = u.recv();
			y = u.recv();
		}
		v.send(msg);
	}
}

#[deriving(Eq)]
enum state {
	matching,
	matched
}

// basic convolutional detector, accepts an infinite sequence, passes all symbols after a match until a 1,0 symbol
pub fn detector(u: Receiver<Token>, v: Sender<Token>, W: ~[uint]) {
	// surprisingly useless unless implemented in hardware
	let mut m: ~[uint] = range(0,W.len()).map(|_| 0).to_owned_vec();
	let mut state = matching;
	loop {
		match u.recv() {
			Chip(x) if state == matching => {m.push(x);m.shift();},
			Chip(x) if state == matched => {v.send(Chip(x));m.push(x);m.shift();},
			Break(x) => {state = matching; v.send(Break(x)); m = range(0,W.len()).map(|_| 0).to_owned_vec();},
			_ => (),
		}
		if m == W {
			state = matched;
			let x = Break("preamble match");
			v.send(x);
			m = range(0,W.len()).map(|_| 0).to_owned_vec();
		}
	}
}

// duplicates infinite sequences
pub fn tuplicator(u: Receiver<Token>, v: Sender<Token>, W: Sender<Token>) {
	loop {
		let y = u.recv();
		v.send(y.clone());
		W.send(y);
	}
}



pub fn twofunnel(u: Receiver<Token>, v: Receiver<Token>, W: Sender<Token>) {
	let x = W.clone();
	let y = W.clone();
	spawn(proc() { loop { x.send(u.recv()) }});
	spawn(proc() { loop { y.send(v.recv()) }});
}

pub fn packetizer(u: Receiver<Token>, v: Sender<Token>, s: SourceConf, T: uint) {
	loop {
		let mut m: ~[Token] = ~[];
		'acc : loop {
			if m.len() == T {break 'acc}
			match u.recv() {
				Break(_) => {break 'acc}
				x => (m.push(x))
			}
		}
		if m.len() > 0 {
			v.send(Packet(m.clone()));
		}
	}
}


pub fn decoder(u: Receiver<Token>, v: Sender<Token>, Q: SourceConf, T: ~[uint]) {
	loop {
		match u.recv() {
			Packet(p) => {
					let bits: ~[uint] = p.move_iter().filter_map(|x| match x { Chip(a) => { Some(a) }, _ => None }).to_owned_vec();
					let b = eat(bits.slice_from(0), T.clone());
					v.send(Packet(b.move_iter().map(|x| Chip(x)).to_owned_vec()));
			},
			_ => ()
		}
	}
}

pub fn differentiator(u: Receiver<Token>, v: Sender<Token>, s: SourceConf) {
	let mut x: Token = u.recv();
	loop {
		let y = u.recv();
		if x != y {
			x = y;
			v.send(x.clone());
		}
	}
}

pub fn unpacketizer(u: Receiver<Token>, v: Sender<Token>, s: SourceConf) {
	loop {
		match u.recv() {
			Packet(x) => {for y in x.move_iter() { v.send(y) }},
			y => v.send(y)
		}
	}
}


pub fn printSink(u: Receiver<Token>, s: SourceConf) {
	loop {
		match u.recv() {
			Packet(x) => {
				if x.len() > 1 {
					println!("{:?}", (x.len(), x.iter().filter_map(|z| match z {
						&Dbl(y) => Some(y as uint),
						&Chip(y) => Some(y),
						y => None
					}).to_owned_vec()))
				}},
			x => println!("{:?}", x),
		}
	}
}

pub fn b2d(In: &[uint]) -> uint {
	return range(0, In.len()).map(|x| (1<<(In.len()-x-1))*In[x]).sum();
}

pub fn eat(x: &[uint], is: ~[uint]) -> ~[uint] {
	let mut i = 0;
	let mut out: ~[uint] = ~[];
	for &index in is.iter() {
		out.push(b2d(x.slice(i, i+index)));
		i = i + index;
	}
	return out
}


// recursive encoding of a Token to a msgpack value
pub fn tokenTovalue(u: Token) -> Value {
	match u {
		Packet(p) => Array(p.move_iter().map(|x| tokenTovalue(x)).to_owned_vec()),
		Dbl(x) => Double(x),
		Chip(x) => Unsigned(x as u64),
		Break(s) => String(s.into_owned().into_bytes()),
		Dur(~t,d) => Array(~[tokenTovalue(t), tokenTovalue(Dbl(d))]),
		Run(~t,d) => Array(~[tokenTovalue(t), tokenTovalue(Chip(d))]),
	}
}

pub fn udpTokenSink(u: Receiver<Token>, s: SourceConf) {
	let mut sock = UdpSocket::bind(SocketAddr{ip:Ipv4Addr(127,0,0,1), port:9998}).unwrap();
	loop {
		let v = tokenTovalue(u.recv());
		sock.sendto(msgpack::to_msgpack(&v), SocketAddr{ip:Ipv4Addr(127,0,0,1), port:9999});
	}
}

pub fn unixTokenSink(u: Receiver<Token>, s: SourceConf) {
	let server = Path::new("/tmp/ratpakSink");
	let mut stream = UnixListener::bind(&server);
	let c = stream.listen().incoming().next().unwrap();
	spawn(proc() {
		loop {
			let v = tokenTovalue(u.recv());
			c.clone().write(msgpack::to_msgpack(&v));
		}
	});
}
