/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

extern crate dsputils;
extern crate num;
use std::comm::{Sender, Receiver, Select, Handle, Messages};

use std::iter::AdditiveIterator;
use std::num::{Float, Num}; 
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::{Listener, Acceptor};

// run length encoding
pub fn rle<T: Eq+Clone+Send>(u: Receiver<T>, v: Sender<(T, uint)>) {
	let mut x = u.recv();
	let mut i: uint = 1;
	loop {
		let y = u.recv();
		if y != x {
			v.send((x.clone(), i));
			i = 1;
		}
		else {i = i + 1}
		x = y;
	}
}

// accept input infinite sequence of runs, convert counts to duration by dividing by sample rate
pub fn dle<T: Eq+Clone+Send>(u: Receiver<(T, uint)>, v: Sender<(T, f32)>, s_rate: uint) {
	loop {
		match u.recv() {
			(x, ct) => v.send( ( x, ct as f32 / s_rate as f32) ),
		}
	}
}

// duration length decoding
pub fn dld<T: Clone+Send>(u: Receiver<(T, f32)>, v: Sender<T>, s_rate: f32) {
	loop {
		match u.recv() {
			(x, dur) => for _ in range(0, (dur*s_rate) as uint) {v.send(x.clone())}
		}
	}
}

// run length decoding
pub fn rld<T: Eq+Clone+Send>(u: Receiver<(T, uint)>, v: Sender<T>) {
	loop {
		match u.recv() {
			(x, ct) => for _ in range(0, ct){v.send(x.clone())},
		}
	}
}

pub fn decoder(u: Receiver<Vec<uint>>, v: Sender<Vec<uint>>, t: &[uint]) {
	loop {
		let p = u.recv();
		if p.len() >= dsputils::sum(t) {
			let b = eat(p.slice_from(0), t);
			v.send(b);
		};
	}
}

pub fn differentiator<T: PartialEq+Send+Clone>(u: Receiver<T>, v: Sender<T>) {
	let mut x = u.recv();
	loop {
		let y = u.recv();
		if x != y {
			x = y;
			v.send(x.clone());
		}
	}
}

pub fn dxdt<T: Send+Clone+Num>(u: Receiver<T>, v: Sender<T>) {
	let mut x = u.recv();
	loop {
		let y = u.recv();
		x = y - x;
		v.send(x.clone());
	}
}
		

pub fn unpacketizer<T: Send+Clone>(u: Receiver<Vec<T>>, v: Sender<T>) {
	loop {
		for x in u.recv().into_iter() {
			v.send(x)
		}
	}
}


pub fn print_sink<T: std::fmt::Show+Send>(u: Receiver<T>) {
	loop {
		println!("{}", u.recv())
	}
}

pub fn b2d(xs: &[uint]) -> uint {
	return range(0, xs.len()).map(|i| (1<<(xs.len()-i-1))*xs[i]).sum();
}

pub fn eat(x: &[uint], is: &[uint]) -> Vec<uint> {
	let mut i = 0;
	let mut out: Vec<uint> = vec!();
	for &index in is.iter() {
		out.push(b2d(x.slice(i, i+index)));
		i = i + index;
	}
	return out
}
pub fn applicator<T: Clone+Send>(u: Receiver<T>, v: Sender<T>, f: |T|->T) {
	loop {
		v.send(f(u.recv()))
	}
}

pub fn applicator_vecs<T: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, f: |&T|->T) {
	loop {
		v.send(u.recv().iter().map(|x|f(x)).collect())
	}
}

pub fn soft_source<T: Send+Clone>(v: Sender<T>, f: |x: Sender<T>|) {
	f(v.clone());
	let (s,r) = channel::<()>();
	r.recv();
}

pub fn looper<T: Send+Clone, U: Send+Clone>(u: Receiver<T>, v: Sender<U>, f: |x: Messages<T>,v: Sender<U>|) {
	f(u.iter(), v)
}

pub fn looper_optional<T: Send+Clone>(u: Receiver<Option<T>>, v: Sender<T>){
	loop {
		match u.recv() {
			Some(d) => v.send(d),
			None => ()
		}
	}
}

pub fn cross_applicator<T: Clone+Send, U: Clone+Send>(u: Receiver<T>, v: Sender<U>, f: |T|->U) {
	loop {
		v.send(f(u.recv()))
	}
}

pub fn cross_applicator_vecs<T: Clone+Send, U: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<U>>, f: |&T|->U) {
	loop {
		v.send(u.recv().iter().map(|x|f(x)).collect())
	}
}

pub fn vec<T: Clone>(u: &[T]) -> Vec<T> {
	u.to_vec()
}

pub fn fork<T: Clone+Send>(u: Receiver<T>, v: &[Sender<T>]) {
	loop {
		let x = u.recv();
		for y in v.iter() {
			y.send(x.clone());
		}
	}
}

pub fn mul<T: Float+Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	loop {
		v.send(u.recv()*c)
	}
}

pub fn mul_vecs<T: Float+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.recv().iter().zip(c.iter()).map(|(&x, &y)| x*y).collect())
	}
}

pub fn sum_across<T: Float+Send>(u: &[Receiver<T>], v: Sender<T>, c: T) {
	loop {
		v.send(u.iter().map(|y| y.recv()).fold(c, |b, a| b+a))
	}
}

pub fn mul_across<T: Float+Send>(u: &[Receiver<T>], v: Sender<T>, c: T) {
	loop {
		v.send(u.iter().map(|y| y.recv()).fold(c, |b, a| b*a))
	}
}

pub fn sum_across_vecs<T: Float+Send>(u: &[Receiver<Vec<T>>], v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.iter().map(|y| y.recv()).fold(c.clone(), |b, a| a.iter().zip(b.iter()).map(|(&d, &e)| d+e).collect()))
	}
}

pub fn sum_vecs<T: Float+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.recv().iter().zip(c.iter()).map(|(&x, &y)| x+y).collect())
	}
}

pub fn sum<T: Float+Send>(u: Receiver<T>, v: Sender<T>, c: T){
	loop {
		v.send(u.recv()+c);
	}
}

pub fn grapes<T: Send>(u: &[Receiver<T>], v: Sender<T>) {
	let mut timer = std::io::Timer::new().unwrap();
	loop {
		for x in u.iter() {
			match x.try_recv() {
				Ok(d) => v.send(d),
				Err(_) => ()
			}
			timer.sleep(std::time::duration::Duration::nanoseconds(10));
		}
	}
}

pub fn delay<T: Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	v.send(c);
	loop {
		v.send(u.recv());
	}
}

pub fn delay_vecs<T: Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	delay(u, v, c);
}

pub fn shaper_optional<T: Send+Clone>(u: Receiver<Option<T>>, v: Sender<Vec<T>>, l: uint) {
	let mut x = vec!();
	loop {
		match u.recv() {
			Some(y) => x.push(y),
			None if x.len() == l => {v.send(x.clone()); x = vec!();},
			None => {x = vec!();},
		}
	}
}

pub fn shaper<T: Send+Clone>(u: Receiver<T>, v: Sender<Vec<T>>, l: uint) {
	loop {
		v.send(range(0, l).map(|_| u.recv()).collect())
	}
}

pub fn shaper_vecs<T: Send+Clone>(u: Receiver<Vec<T>>, v: Sender<T>) {
	for x in u.iter() {
		for y in x.into_iter() {
			v.send(y)
		}
	}
}

pub fn binconv(u: Receiver<Vec<uint>>, v: Sender<Vec<uint>>, l: &[uint]) {
	loop {
		v.send(eat(u.recv().slice_from(0), l.clone()))
	}
}

