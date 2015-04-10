#![feature(old_io)]
#![feature(core)]
#![feature(std_misc)]

/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

extern crate dsputils;
extern crate core;

use std::sync::mpsc::{Receiver, Sender, channel, Handle, Select};
use std::iter::*;
use core::num::{Float}; 
use std::net::{SocketAddr, Ipv4Addr};

// run length encoding
pub fn rle<T: Eq+Clone+Send>(u: Receiver<T>, v: Sender<(T, usize)>) {
	let mut x = u.recv().unwrap();
	let mut i: usize = 1;
	loop {
		let y = u.recv().unwrap();
		if y != x {
			v.send((x.clone(), i)).unwrap();
			i = 1;
		}
		else {i = i + 1}
		x = y;
	}
}

// accept input infinite sequence of runs, convert counts to duration by dividing by sample rate
pub fn dle<T: Eq+Clone+Send>(u: Receiver<(T, usize)>, v: Sender<(T, f32)>, s_rate: usize) {
	loop {
		match u.recv().unwrap() {
			(x, ct) => v.send( ( x, ct as f32 / s_rate as f32) ).unwrap(),
		}
	}
}

// duration length decoding
pub fn dld<T: Clone+Send>(u: Receiver<(T, f32)>, v: Sender<T>, s_rate: f32) {
	loop {
		match u.recv().unwrap() {
			(x, dur) => for _ in (0..(dur*s_rate) as usize) {v.send(x.clone()).unwrap()},
		}
	}
}

// run length decoding
pub fn rld<T: Eq+Clone+Send>(u: Receiver<(T, usize)>, v: Sender<T>) {
	loop {
		match u.recv().unwrap() {
			(x, ct) => for _ in (0..ct){v.send(x.clone()).unwrap()},
		}
	}
}

/* not sure what's going on here.
pub fn decoder(u: Receiver<Vec<usize>>, v: Sender<Vec<usize>>, t: &[usize]) {
	use std::iter::AdditiveIterator;
	loop {
		let p = u.recv().unwrap();
		let i: usize = t.iter().map(|&x|x).sum();
		if p.len() >= i {
			let b = eat(&p[0..], t);
			v.send(b).unwrap();
		};
	}
}*/


// drop repeated samples
pub fn differentiator<T: PartialEq+Send+Clone>(u: Receiver<T>, v: Sender<T>) {
	let mut x = u.recv().unwrap();
	loop {
		let y = u.recv().unwrap();
		if x != y {
			x = y;
			v.send(x.clone()).unwrap();
		}
	}
}

// first-order discrete time difference function
pub fn dxdt<T: Send+Clone+Float>(u: Receiver<T>, v: Sender<T>) {
	let mut x = u.recv().unwrap();
	loop {
		let y = u.recv().unwrap();
		x = y - x;
		v.send(x.clone()).unwrap();
	}
}
		
// unpack vecs to a list of elements
pub fn unpacketizer<T: Send+Clone>(u: Receiver<Vec<T>>, v: Sender<T>) {
	loop {
		for x in u.recv().unwrap().into_iter() {
			v.send(x).unwrap()
		}
	}
}

// stringify arbitrary objects and print to stdout
pub fn print_sink<T: std::fmt::Debug+Send>(u: Receiver<T>) {
	loop {
		println!("{:?}", u.recv().unwrap())
	}
}

// transform a list of binary digits to an unsigned integer
pub fn b2d(xs: &[usize]) -> usize {
	return (0..xs.len()).map(|i| (1<<(xs.len()-i-1))*xs[i]).sum();
}

// transform a list of binary digits and a list of bit widths into a list of unsigned integers
pub fn eat(x: &[usize], is: &[usize]) -> Vec<usize> {
	let mut i = 0;
	let mut out: Vec<usize> = vec!();
	for &index in is.iter() {
		out.push(b2d(&x[i..i+index]));
		i = i + index;
	}
	return out
}

// map a function |T|->T across Channel<T>
pub fn applicator<T: Clone+Send>(u: Receiver<T>, v: Sender<T>, f: &Fn(T)->T) {
	loop {
		v.send(f(u.recv().unwrap())).unwrap()
	}
}

// map a function |T|->T across Channel<Vec<T>>
pub fn applicator_vecs<T: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, f: &Fn(&T)->T) {
	loop {
		v.send(u.recv().unwrap().iter().map(|x|f(x)).collect()).unwrap()
	}
}

// run Fn(Sender<T>), wait for main loop exit
pub fn soft_source<T: Send+Clone>(v: Sender<T>, f: &Fn(Sender<T>)) {
	f(v.clone());
	let (s,r) = channel::<()>();
	r.recv().unwrap();
}

// apply a function accepting an iterator and a sender across an input stream
pub fn looper<T: Send+Clone, U: Send+Clone>(u: Receiver<T>, v: Sender<U>, f: &Fn(std::sync::mpsc::Iter<T>, Sender<U>)) {
	f(u.iter(), v)
}

// take maybe-T to T
pub fn looper_optional<T: Send+Clone>(u: Receiver<Option<T>>, v: Sender<T>){
	loop {
		match u.recv().unwrap() {
			Some(d) => v.send(d).unwrap(),
			None => ()
		}
	}
}

// map a function |T|->U across Channel<T>-><U>
pub fn cross_applicator<T: Clone+Send, U: Clone+Send>(u: Receiver<T>, v: Sender<U>, f: &Fn(T)->U) {
	loop {
		v.send(f(u.recv().unwrap())).unwrap()
	}
}

// map a function |T|->U across Channel <Vec<T>>-><U>
pub fn cross_applicator_vecs<T: Clone+Send, U: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<U>>, f: &Fn(&T)->U) {
	loop {
		v.send(u.recv().unwrap().iter().map(|x|f(x)).collect()).unwrap()
	}
}

// convenience function
pub fn vec<T: Clone>(u: &[T]) -> Vec<T> {
	u.to_vec()
}

// duplicate a stream of type T
pub fn fork<T: Clone+Send>(u: Receiver<T>, v: &[Sender<T>]) {
	loop {
		let x = u.recv().unwrap();
		for y in v.iter() {
			y.send(x.clone()).unwrap();
		}
	}
}

// scale a stream of type T by a constant
pub fn mul<T: Float+Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	loop {
		v.send(u.recv().unwrap()*c).unwrap()
	}
}

// scale a stream of vectors type T by a vector of constants
pub fn mul_vecs<T: Float+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.recv().unwrap().iter().zip(c.iter()).map(|(&x, &y)| x*y).collect()).unwrap()
	}
}

// offset a stream of type T by a constant
pub fn sum_across<T: Float+Send>(u: &[Receiver<T>], v: Sender<T>, c: T) {
	loop {
		v.send(u.iter().map(|y| y.recv().unwrap()).fold(c, |b, a| b+a)).unwrap()
	}
}

// scale streams of type T by eachother and a constant value
pub fn mul_across<T: Float+Send>(u: &[Receiver<T>], v: Sender<T>, c: T) {
	loop {
		v.send(u.iter().map(|y| y.recv().unwrap()).fold(c, |b, a| b*a)).unwrap()
	}
}

// sum streams of vectors type T with eachother and a vector of constants
pub fn sum_across_vecs<T: Float+Send>(u: &[Receiver<Vec<T>>], v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.iter().map(|y| y.recv().unwrap()).fold(c.clone(), |b, a| a.iter().zip(b.iter()).map(|(&d, &e)| d+e).collect())).unwrap()
	}
}

// offset a stream of vectors of type T by a vector of constants
pub fn sum_vecs<T: Float+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.recv().unwrap().iter().zip(c.iter()).map(|(&x, &y)| x+y).collect()).unwrap()
	}
}

// discrete-time accumulator, init with a constant value
pub fn sum<T: Float+Send>(u: Receiver<T>, v: Sender<T>, c: T){
	loop {
		v.send(u.recv().unwrap()+c).unwrap();
	}
}

// asynchronous merge of a list of streams to a single stream
pub fn grapes<T: Send>(u: &[Receiver<T>], v: Sender<T>) {
	let mut timer = std::old_io::Timer::new().unwrap();
	loop {
		for x in u.iter() {
			match x.try_recv().unwrap() {
				d => v.send(d).unwrap(),
			}
			timer.sleep(std::time::duration::Duration::nanoseconds(10));
		}
	}
}

// discrete-time stream offset with specified t=0 value
pub fn delay<T: Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	v.send(c).unwrap();
	loop {
		v.send(u.recv().unwrap());
	}
}

pub fn delay_vecs<T: Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	delay(u, v, c);
}

// map a stream of optional values of type T with a specified number of sequential values of to a vector of type T of length l
pub fn shaper_optional<T: Send+Clone>(u: Receiver<Option<T>>, v: Sender<Vec<T>>, l: usize) {
	let mut x = vec!();
	loop {
		match u.recv().unwrap() {
			Some(y) => x.push(y),
			None if x.len() == l => {v.send(x.clone()); x = vec!();},
			None => {x = vec!();},
		}
	}
}

// map a stream of type T to vectors of type T and length l
pub fn shaper<T: Send+Clone>(u: Receiver<T>, v: Sender<Vec<T>>, l: usize) {
	loop {
		v.send((0..l).map(|_| u.recv().unwrap()).collect()).unwrap()
	}
}

// unwrap a stream of Vec<T> to a stream of T
pub fn shaper_vecs<T: Send+Clone>(u: Receiver<Vec<T>>, v: Sender<T>) {
	for x in u.iter() {
		for y in x.into_iter() {
			v.send(y).unwrap()
		}
	}
}

// apply eat to a stream of chunks of binary data
// - eat - transform a list of binary digits and a list of bit widths into a list of unsigned integers
pub fn binconv(u: Receiver<Vec<usize>>, v: Sender<Vec<usize>>, l: &[usize]) {
	loop {
		v.send(eat(&u.recv().unwrap()[0..], l.clone())).unwrap()
	}
}

