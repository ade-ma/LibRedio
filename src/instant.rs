/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-NC-SA 4.0. */

extern mod native;
extern mod kpn;
extern mod bitfount;

use kpn::{Symbol, SourceConf};
use native::task::spawn;

// parts of a directed acyclical flowgraph
#[deriving(Clone)]
pub enum Parts{
	Head (fn (Chan<Symbol>, SourceConf) -> () ), // stream source
	Body (fn (Port<Symbol>, Chan<Symbol>, SourceConf) -> () ), // stream processor
	Tail (fn (Port<Symbol>, SourceConf) -> () ), // stream sink
	Fork (fn (Port<Symbol>, Chan<Symbol>, Chan<Symbol>) -> () ), // stream split
	Leg (~[Parts] ), // stream
}

// guard for heterogenous vector of stream endpoints
enum Either{
	P(Port<Symbol>),
	C(Chan<Symbol>)
}

// accepts a list of guarded functions, instantiates a directed acyclical flowgraph
pub fn spinUp(mut fss: ~[Parts], mut ps: ~[Either], conf: SourceConf) {
	// spawn ports and channels
	for _ in range(0, fss.len()) {
		let (p, c) = Chan::new();
		ps.push(C(c));
		ps.push(P(p));
	}
	// iterate over functions, shifting ports and channels out of the previously created vector
	for _ in range(0, fss.len()) {
		match (fss.shift(), ps.shift()) {
			(Some(Head(source)), Some(C(c))) => { // if we have a source, we should have a channel
				do spawn { source(c, conf.clone()) } ;
			},
			(Some(Body(manip)), Some(P(p))) => {
				match ps.shift() {
					Some(C(c)) => do spawn { manip(p, c, conf.clone()) },
					_ => ()
				}
			}
			(Some(Tail(sink)), Some(P(p))) => { // if we have a sink, we should have a port
				do spawn { sink(p, conf.clone()) } ;
			}
			(Some(Fork(split)), Some(P(p))) => { // if we have a fork, we should have a port
				let (p1, c1) = Chan::new();
				let (p2, c2) = Chan::new();
				do spawn { split(p, c1, c2) }
				for p in (~[p1, p2]).move_iter() { // for each of the new ports
					match fss.shift() {
						Some(Leg(l)) => do spawn { spinUp(l, ~[P(p)], conf.clone());}, // spinUp the the new leg with the port
						Some(x) => fss.unshift(x),
						_ => (),
					}
				}
			}
			_ => {}
		}
	}
}

