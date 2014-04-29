/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

//extern crate msgpack;
extern crate native;
extern crate dsputils;

use native::task::spawn;
use std::comm::{Sender, Receiver, Select, Handle};

use std::iter::AdditiveIterator;
//use msgpack::{Array, Unsigned, Double, Value, String, Float};

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::net::udp::UdpSocket;
use std::io::{Listener, Acceptor};


use std::io::net::unix::UnixListener;
use std::io::{Listener, Acceptor};

#[deriving(Eq, Clone, Show)]
pub enum Token {
	Break(&'static str),
	Dur(uint, f32),
	Run(uint, uint),
}

// run length encoding
pub fn rle(u: Receiver<uint>, v: Sender<Token>) {
	let mut x = u.recv();
	let mut i: uint = 1;
	loop {
		let y = u.recv();
		if y != x {
			v.send(Run(x, i));
			i = 1;
		}
		else {i = i + 1}
		x = y;
	}
}

// accept input infinite sequence of runs, convert counts to duration by dividing by sample rate
pub fn dle(u: Receiver<Token>, v: Sender<Token>, sRate: uint) {
	loop {
		match u.recv() {
			Run(x, ct) => v.send( Dur ( x, ct as f32 / sRate as f32) ),
			_ => (),
		}
	}
}

// duration length decoding
pub fn dld(u: Receiver<Token>, v: Sender<Token>, sRate: f32) {
	loop {
		match u.recv() {
			Dur(x, dur) => v.send( Run ( x, (dur * sRate) as uint)),
			_ => (),
		}
	}
}

// run length decoding
pub fn rld(u: Receiver<Token>, v: Sender<uint>) {
	loop {
		match u.recv() {
			Run(x, ct) => for _ in range(0, ct){v.send(x.clone())},
			_ => (),
		}
	}
}

pub fn decoder(u: Receiver<Vec<uint>>, v: Sender<Vec<uint>>, t: ~[uint]) {
	loop {
		let p = u.recv();
		if p.len() >= dsputils::sum(t.slice_from(0)) {
			let b = eat(p.slice_from(0), t.clone());
			v.send(b);
		};
	}
}

pub fn differentiator<T: Eq+Send+Clone>(u: Receiver<T>, v: Sender<T>) {
	let mut x = u.recv();
	loop {
		let y = u.recv();
		if x != y {
			x = y;
			v.send(x.clone());
		}
	}
}

pub fn unpacketizer<T: Send+Clone>(u: Receiver<Vec<T>>, v: Sender<T>) {
	loop {
		for x in u.recv().move_iter() {
			v.send(x)
		}
	}
}


pub fn printSink<T: std::fmt::Show+Send>(u: Receiver<T>) {
	loop {
		println!("{}", u.recv())
	}
}

pub fn b2d(xs: &[uint]) -> uint {
	return range(0, xs.len()).map(|i| (1<<(xs.len()-i-1))*xs[i]).sum();
}

pub fn eat(x: &[uint], is: ~[uint]) -> Vec<uint> {
	let mut i = 0;
	let mut out: Vec<uint> = vec!();
	for &index in is.iter() {
		out.push(b2d(x.slice(i, i+index)));
		i = i + index;
	}
	return out
}
