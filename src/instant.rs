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

// guard for heterogenous vector of stream endpoints
enum Either{
	P(Port<Token>),
	C(Chan<Token>)
}

// accepts a list of guarded functions, instantiates a directed acyclical flowgraph
pub fn spinUp(mut fss: ~[Parts], mut ps: ~[Either], conf: SourceConf) -> Option<Either>{
	// spawn ports and channels
	for _ in range(0, fss.len()) {
		let (p, c) = Chan::new();
		ps.push(C(c));
		ps.push(P(p));
	}
	let ret = match fss.iter().last().unwrap() {
		&Body(_) => true,
		_ => false,
	};
	// iterate over functions, shifting ports and channels out of the previously created vector
	for _ in range(0, fss.len()) {
		match (fss.shift(), ps.shift()) {
			(Some(Head(source)), Some(C(c))) => { // if we have a source, we should have a channel
				spawn(proc() { source(c, conf.clone()) }) ;
			},
			(Some(Body(manip)), Some(P(p))) => {
				match ps.shift() {
					Some(C(c)) => spawn(proc() { manip(p, c, conf.clone()) }),
					_ => ()
				}
			}
			(Some(Tail(sink)), Some(P(p))) => { // if we have a sink, we should have a port
				spawn(proc() { sink(p, conf.clone()) });
			}
			(Some(Fork(split)), Some(P(p))) => { // if we have a fork, we should have a port
				let (p1, c1) = Chan::new();
				let (p2, c2) = Chan::new();
				spawn(proc(){ split(p, c1, c2) });
				ps.unshift(P(p1)); // use this as a port for another piece
				ps.unshift(P(p2)); // ditto
			},
			(Some(Funnel(fun)), Some(C(c1))) => { // if we have a funnel...
				match (ps.pop(), ps.pop()) { // grab two ports off the back of the list of endpoints
					(Some(P(p1)), Some(P(p2))) => spawn (proc() {fun(p1, p2, c1)}),
					_ => (),
				}
			}
			(Some(Leg(l)), Some(P(p))) => { // if we have a leg, consisting of an indeterminant number of body segments and a sink,
				match spinUp(l, ~[P(p)], conf.clone()) { // recurse
					Some(x) => ps.push(x), // if we get something back, stick it in the back of our endpoint list
					None => ()
				}
			}
			(x,y) => println!("{:?}", (x,y)),
		}
	}
	if ret {
		return ps.shift()
	}
	else {
		return None
	}
}

