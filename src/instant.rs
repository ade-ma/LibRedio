/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3 */

extern crate native;
extern crate kpn;
extern crate bitfount;

use kpn::{Token, SourceConf};
use native::task::spawn;

// parts of a directed acyclical flowgraph
#[deriving(Clone)]
pub enum Parts{
	Head (fn (Sender<Token>, SourceConf) -> () ), // stream source
	Body (fn (Receiver<Token>, Sender<Token>, SourceConf) -> () ), // stream processor
	Mimo (uint, uint, fn(~[Receiver<Token>], ~[Sender<Token>]) -> () ),
	Tail (fn (Receiver<Token>, SourceConf) -> () ), // stream sink
	Fork (fn (Receiver<Token>, Sender<Token>, Sender<Token>) -> () ), // stream split
	Funnel (fn (Receiver<Token>, Receiver<Token>, Sender<Token>) -> ()), // 2-into-1 stream combiner
	Leg (~[Parts] ),
}

pub fn spinUp(mut fss: ~[Parts], mut ps: ~[Receiver<Token>], mut cs: ~[Sender<Token>], s: SourceConf) -> Option<Receiver<Token>>{
	// spawn ports and channels
	for _ in range(0, fss.len()) {
		let (c, p) = channel();
		cs.push(c);
		ps.push(p);
	}
	let ret = match fss.iter().last().unwrap() {
		&Body(_) => true,
		_ => false,
	};
	// iterate over functions
	for f in fss.move_iter() {
		match f {
			Head(source) => {
				let c = cs.shift().unwrap();
				spawn(proc() { source(c, s.clone()) });
			},
			Body(manip) => {
				let c = cs.shift().unwrap();
				let p = ps.shift().unwrap();
				spawn(proc() { manip(p, c, s.clone()) });
			}
			Mimo(i, o, mimo) => {
				let p = range(0, i).map(|_| { ps.shift().unwrap() }).to_owned_vec();
				let c = range(0, o).map(|_| { cs.shift().unwrap() }).to_owned_vec();
				spawn(proc() { mimo(p, c) });
			}
			Tail(sink) => {
				let p = ps.shift().unwrap();
				spawn(proc() { sink(p, s.clone()) });
			}
			Fork(split) => {
				let (c1, p1) = channel();
				let (c2, p2) = channel();
				let p = ps.shift().unwrap();
				spawn(proc(){ split(p, c1, c2) });
				ps.unshift(p1);
				ps.unshift(p2);
			},
			Funnel(fun) => {
				let p1 = ps.pop().unwrap(); // if we combine, pull two things off the back of our endpoint stack
				let p2 = ps.pop().unwrap();
				let c = cs.shift().unwrap();
				spawn (proc() {fun(p1, p2, c)});
			},
			Leg(l) => {
				let p = ps.shift().unwrap();
				match spinUp(l, ~[p], ~[], s.clone()) {
					Some(x) => ps.push(x), // if we get something back, stick it in the back of our endpoint list
					None => ()
				}
			},
		}
	}
	if ret {
		return ps.shift()
	}
	else {
		return None
	}
}

