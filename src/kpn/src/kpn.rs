/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

extern crate dsputils;
extern crate num;

use std::sync::mpsc::{Receiver, Sender, channel, Handle, Select};
use std::iter::*;
use std::num::{Float}; 
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::{Listener, Acceptor};

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
			(x, dur) => for _ in range(0, (dur*s_rate) as usize) {v.send(x.clone()).unwrap()},
		}
	}
}

// run length decoding
pub fn rld<T: Eq+Clone+Send>(u: Receiver<(T, usize)>, v: Sender<T>) {
	loop {
		match u.recv().unwrap() {
			(x, ct) => for _ in range(0, ct){v.send(x.clone()).unwrap()},
		}
	}
}

pub fn decoder(u: Receiver<Vec<usize>>, v: Sender<Vec<usize>>, t: &[usize]) {
	use std::iter::AdditiveIterator;
	loop {
		let p = u.recv().unwrap();
		let i: AdditiveIterator<usize> = t.iter();
		if p.len() >= i.sum() {
			let b = eat(p.slice_from(0), t);
			v.send(b).unwrap();
		};
	}
}

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

pub fn dxdt<T: Send+Clone+Float>(u: Receiver<T>, v: Sender<T>) {
	let mut x = u.recv().unwrap();
	loop {
		let y = u.recv().unwrap();
		x = y - x;
		v.send(x.clone()).unwrap();
	}
}
		

pub fn unpacketizer<T: Send+Clone>(u: Receiver<Vec<T>>, v: Sender<T>) {
	loop {
		for x in u.recv().unwrap().into_iter() {
			v.send(x).unwrap()
		}
	}
}


pub fn print_sink<T: std::fmt::Show+Send>(u: Receiver<T>) {
	loop {
		println!("{}", u.recv().unwrap())
	}
}

pub fn b2d(xs: &[usize]) -> usize {
	return range(0, xs.len()).map(|i| (1<<(xs.len()-i-1))*xs[i]).sum();
}

pub fn eat(x: &[usize], is: &[usize]) -> Vec<usize> {
	let mut i = 0;
	let mut out: Vec<usize> = vec!();
	for &index in is.iter() {
		out.push(b2d(x.slice(i, i+index)));
		i = i + index;
	}
	return out
}
pub fn applicator<T: Clone+Send>(u: Receiver<T>, v: Sender<T>, f: Fn(T)->T) {
	loop {
		v.send(f(u.recv().unwrap())).unwrap()
	}
}

pub fn applicator_vecs<T: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, f: Fn(&T)->T) {
	loop {
		v.send(u.recv().unwrap().iter().map(|x|f(x)).collect()).unwrap()
	}
}

pub fn soft_source<T: Send+Clone>(v: Sender<T>, f: Fn(Sender<T>)) {
	f(v.clone());
	let (s,r) = channel::<()>();
	r.recv().unwrap();
}

pub fn looper<T: Send+Clone, U: Send+Clone>(u: Receiver<T>, v: Sender<U>, f: Fn(std::sync::mpsc::Iter<T>, Sender<U>)) {
	f(u.iter(), v)
}

pub fn looper_optional<T: Send+Clone>(u: Receiver<Option<T>>, v: Sender<T>){
	loop {
		match u.recv().unwrap() {
			Some(d) => v.send(d).unwrap(),
			None => ()
		}
	}
}

pub fn cross_applicator<T: Clone+Send, U: Clone+Send>(u: Receiver<T>, v: Sender<U>, f: Fn(T)->U) {
	loop {
		v.send(f(u.recv().unwrap())).unwrap()
	}
}

pub fn cross_applicator_vecs<T: Clone+Send, U: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<U>>, f: Fn(&T)->U) {
	loop {
		v.send(u.recv().unwrap().iter().map(|x|f(x)).collect()).unwrap()
	}
}

pub fn vec<T: Clone>(u: &[T]) -> Vec<T> {
	u.to_vec()
}

pub fn fork<T: Clone+Send>(u: Receiver<T>, v: &[Sender<T>]) {
	loop {
		let x = u.recv().unwrap();
		for y in v.iter() {
			y.send(x.clone()).unwrap();
		}
	}
}

pub fn mul<T: Float+Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	loop {
		v.send(u.recv().unwrap()*c).unwrap()
	}
}

pub fn mul_vecs<T: Float+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.recv().unwrap().iter().zip(c.iter()).map(|(&x, &y)| x*y).collect()).unwrap()
	}
}

pub fn sum_across<T: Float+Send>(u: &[Receiver<T>], v: Sender<T>, c: T) {
	loop {
		v.send(u.iter().map(|y| y.recv().unwrap()).fold(c, |b, a| b+a)).unwrap()
	}
}

pub fn mul_across<T: Float+Send>(u: &[Receiver<T>], v: Sender<T>, c: T) {
	loop {
		v.send(u.iter().map(|y| y.recv().unwrap()).fold(c, |b, a| b*a)).unwrap()
	}
}

pub fn sum_across_vecs<T: Float+Send>(u: &[Receiver<Vec<T>>], v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.iter().map(|y| y.recv().unwrap()).fold(c.clone(), |b, a| a.iter().zip(b.iter()).map(|(&d, &e)| d+e).collect())).unwrap()
	}
}

pub fn sum_vecs<T: Float+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.recv().unwrap().iter().zip(c.iter()).map(|(&x, &y)| x+y).collect()).unwrap()
	}
}

pub fn sum<T: Float+Send>(u: Receiver<T>, v: Sender<T>, c: T){
	loop {
		v.send(u.recv().unwrap()+c).unwrap();
	}
}

pub fn grapes<T: Send>(u: &[Receiver<T>], v: Sender<T>) {
	let mut timer = std::io::Timer::new().unwrap();
	loop {
		for x in u.iter() {
			match x.try_recv().unwrap() {
				d => v.send(d).unwrap(),
			}
			timer.sleep(std::time::duration::Duration::nanoseconds(10));
		}
	}
}

pub fn delay<T: Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	v.send(c).unwrap();
	loop {
		v.send(u.recv().unwrap());
	}
}

pub fn delay_vecs<T: Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	delay(u, v, c);
}

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

pub fn shaper<T: Send+Clone>(u: Receiver<T>, v: Sender<Vec<T>>, l: usize) {
	loop {
		v.send(range(0, l).map(|_| u.recv().unwrap()).collect()).unwrap()
	}
}

pub fn shaper_vecs<T: Send+Clone>(u: Receiver<Vec<T>>, v: Sender<T>) {
	for x in u.iter() {
		for y in x.into_iter() {
			v.send(y).unwrap()
		}
	}
}

pub fn binconv(u: Receiver<Vec<usize>>, v: Sender<Vec<usize>>, l: &[usize]) {
	loop {
		v.send(eat(u.recv().unwrap().slice_from(0), l.clone())).unwrap()
	}
}

