/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

extern crate msgpack;
extern crate native;

use native::task::spawn;
use std::comm::{Chan, Port, Data, Select, Handle};

use std::iter::AdditiveIterator;
use msgpack::{Array, Unsigned, Double, Value, String};

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::net::udp::UdpSocket;
use std::io::{Listener, Acceptor};


#[deriving(Eq, Clone, DeepClone)]
pub struct SourceConf {
	Freq: f64,
	Rate: f64,
	Period: f64
}

#[deriving(Eq, Clone, DeepClone)]
pub enum Token {
	Chip(uint),
	Dbl(f64),
	Break(~str),
	Dur(~Token, f64),
	Run(~Token, uint),
	Packet(~[Token]),
}

// run length encoding
pub fn rle(U: Port<Token>, V: Chan<Token>, S: SourceConf){
	let mut x: Token = U.recv();
	let mut i: uint = 1;
	loop {
		let y = U.recv();
		if y != x {
			V.send(Run(~x, i));
			i = 1;
		}
		else {i = i + 1}
		x = y;
	}
}

// accept input infinite sequence of runs, convert counts to duration by dividing by sample rate
pub fn dle(U: Port<Token>, V: Chan<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Run(v, ct) => V.send( Dur ( v, ct as f64 / S.Rate) ),
			_ => (),
		}
	}
}

// duration length decoding
pub fn dld(U: Port<Token>, V: Chan<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Dur(v, dur) => V.send( Run ( v, (dur * S.Rate) as uint)),
			_ => (),
		}
	}
}

// run length decoding
pub fn rld(U: Port<Token>, V: Chan<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Run(~v, ct) => for _ in range(0, ct){V.send(v.clone())},
			_ => (),
		}
	}
}


// manchester 1/2 pulse duration to state matching
pub fn validTokenManchester(U: Port<Token>, V: Chan<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Dur(~v, dur) => {
				if ((0.7*S.Period) < dur) && ( dur < (1.3*S.Period)) { V.send(v)}
				else if (1.7*S.Period < dur) && (dur < (2.3*S.Period)) { V.send(v.clone()); V.send(v);}
				else if (dur > 3.0*S.Period) && (v == Chip(0)) { V.send(Break(~"silence"))}
			},
			_ => ()
		}
	}
}


// manchester state-pair to symbol decoding
pub fn manchesterd(U: Port<Token>, V: Chan<Token>, S: SourceConf) {
	let mut x = U.recv();
	let mut y = U.recv();
	loop {
		let msg = match (x, y.clone()) {
			(Chip(1),Chip(0)) => Chip(1),
			(Chip(0),Chip(1)) => Chip(0),
			(Chip(a), Chip(b)) if a == b => Break(~"manchester collision"),
			(Break(b), _) =>  Break(b),
			(_, Break(b)) =>  Break(b),
			_ => Break(~"else")
		};
		if msg == Break(~"manchester collision") {
			x = y;
			y = U.recv();
		}
		else {
			x = U.recv();
			y = U.recv();
		}
		V.send(msg);
	}
}

#[deriving(Eq)]
enum state {
	matching,
	matched
}

// basic convolutional detector, accepts an infinite sequence, passes all symbols after a match until a 1,0 symbol
pub fn detector(U: Port<Token>, V: Chan<Token>, W: ~[uint]) {
	// surprisingly useless unless implemented in hardware
	let mut m: ~[uint] = range(0,W.len()).map(|_| 0).to_owned_vec();
	let mut state = matching;
	loop {
		match U.recv() {
			Chip(x) if state == matching => {m.push(x);m.shift();},
			Chip(x) if state == matched => {V.send(Chip(x));m.push(x);m.shift();},
			Break(x) => {state = matching; V.send(Break(x)); m = range(0,W.len()).map(|_| 0).to_owned_vec();},
			_ => (),
		}
		if m == W {
			state = matched;
			let x = Break(~"preamble match");
			V.send(x);
			m = range(0,W.len()).map(|_| 0).to_owned_vec();
		}
	}
}

// duplicates infinite sequences
pub fn tuplicator(U: Port<Token>, V: Chan<Token>, W: Chan<Token>){
	loop {
		let y = U.recv();
		V.send(y.clone());
		W.send(y);
	}
}



pub fn twofunnel(U: Port<Token>, V: Port<Token>, W: Chan<Token>){
	let x = W.clone();
	let y = W.clone();
	spawn(proc() { loop { x.send(U.recv()) }});
	spawn(proc() { loop { y.send(V.recv()) }});
}

pub fn packetizer(U: Port<Token>, V: Chan<Token>, S: SourceConf, T: uint) {
	loop {
		let mut m: ~[Token] = ~[];
		'acc : loop {
			if m.len() == T {break 'acc}
			match U.recv() {
				Break(_) => {break 'acc}
				x => (m.push(x))
			}
		}
		if (m.len() + T/10) > T {
			for _ in range(m.len(), T) {m.unshift(Chip(0u))}; // zeropad and prepend - seems good idea for input, not sure about output

			V.send(Packet(m.clone()));
		}
	}
}


pub fn decoder(U: Port<Token>, V: Chan<Token>, Q: SourceConf, T: ~[uint]) {
	loop {
		match U.recv() {
			Packet(p) => {
					let bits: ~[uint] = p.move_iter().filter_map(|x| match x { Chip(a) => { Some(a) }, _ => None }).to_owned_vec();
					let b = eat(bits.slice_from(0), T.clone());
					V.send(Packet(b.move_iter().map(|x| Chip(x)).to_owned_vec()));
			},
			_ => ()
		}
	}
}

pub fn differentiator(U: Port<Token>, V: Chan<Token>, S: SourceConf) {
	let mut x: Token = U.recv();
	loop {
		let y = U.recv();
		if x != y {
			x = y;
			V.send(x.clone());
		}
	}
}

pub fn unpacketizer(U: Port<Token>, V: Chan<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Packet(x) => {for y in x.move_iter() { V.send(y) }},
			y => V.send(y)
		}
	}
}


pub fn printSink(U: Port<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Packet(x) => println!("{:?}", (x.len(), x)),
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


// recursive encoding of a Token to a msgpack Value
pub fn tokenToValue(U: Token) -> Value {
	match U {
		Packet(p) => Array(p.move_iter().map(|x| tokenToValue(x)).to_owned_vec()),
		Dbl(x) => Double(x),
		Chip(x) => Unsigned(x as u64),
		Break(s) => String(s.into_bytes()),
		Dur(~t,d) => Array(~[tokenToValue(t), tokenToValue(Dbl(d))]),
		Run(~t,d) => Array(~[tokenToValue(t), tokenToValue(Chip(d))]),
	}
}

pub fn udpTokenSink(U: Port<Token>, S: SourceConf) {
	let mut sock = UdpSocket::bind(SocketAddr{ip:Ipv4Addr(127,0,0,1), port:9998}).unwrap();
	loop {
		let v = tokenToValue(U.recv());
		sock.sendto(msgpack::to_msgpack(&v), SocketAddr{ip:Ipv4Addr(127,0,0,1), port:9999});
	}
}
