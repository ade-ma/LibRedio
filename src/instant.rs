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
	Head (fn (Chan<Token>, SourceConf) -> () ), // stream source
	Body (fn (Port<Token>, Chan<Token>, SourceConf) -> () ), // stream processor
	Tail (fn (Port<Token>, SourceConf) -> () ), // stream sink
	Fork (fn (Port<Token>, Chan<Token>, Chan<Token>) -> () ), // stream split
	Funnel (fn (Port<Token>, Port<Token>, Chan<Token>) -> ()), // 2-into-1 stream combiner
	Leg (~[Parts] ), // stream
}

pub fn spinUp(mut fss: ~[Parts], mut ps: ~[Port<Token>], mut cs: ~[Chan<Token>], s: SourceConf) -> Option<Port<Token>>{
	// spawn ports and channels
	for _ in range(0, fss.len()) {
		let (p, c) = Chan::new();
		cs.push(c);
		ps.push(p);
	}
	let ret = match fss.iter().last().unwrap() {
		&Body(_) => true,
		_ => false,
	};
	// iterate over functions, shifting ports and channels out of the previously created vector
	for _ in range(0, fss.len()) {
		match fss.shift() {
			Some(Head(source)) => {
				let c = cs.shift().unwrap();
				spawn(proc() { source(c, s.clone()) }) ;
			},
			Some(Body(manip)) => {
				let c = cs.shift().unwrap();
				let p = ps.shift().unwrap();
				spawn(proc() { manip(p, c, s.clone()) });
			}
			Some(Tail(sink)) => {
				let p = ps.shift().unwrap();
				spawn(proc() { sink(p, s.clone()) });
			}
			Some(Fork(split)) => {
				let (p1, c1) = Chan::new();
				let (p2, c2) = Chan::new();
				let p = ps.shift().unwrap();
				spawn(proc(){ split(p, c1, c2) });
				ps.unshift(p1); // use this as a port for another piece
				ps.unshift(p2); // ditto
			},
			Some(Funnel(fun)) => {
				let p1 = ps.pop().unwrap();
				let p2 = ps.pop().unwrap();
				let c = cs.shift().unwrap();
				spawn (proc() {fun(p1, p2, c)});
			}
			Some(Leg(l)) => {
				let p = ps.shift().unwrap();
				match spinUp(l, ~[p], ~[], s.clone()) {
					Some(x) => ps.push(x), // if we get something back, stick it in the back of our endpoint list
					None => ()
				}
			}
			x => println!("{:?}", x),
		}
	}
	if ret {
		return ps.shift()
	}
	else {
		return None
	}
}

