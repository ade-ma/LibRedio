/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3 */

extern crate native;
extern crate kpn;
extern crate bitfount;

use kpn::Token;
use native::task;

// parts of a directed acyclical flowgraph
#[deriving(Clone)]
pub enum Parts{
	Head (fn (Sender<Token>) -> () ),
	HeadFloat (fn (Sender<Token>, f32) -> (), f32 ),
	HeadFloatFloat (fn (Sender<Token>, f32, f32) -> (), f32, f32 ),
	HeadFloatFloatFloat (fn (Sender<Token>, f32, f32, f32) -> (), f32, f32, f32 ),
	Body (fn (Receiver<Token>, Sender<Token>) -> () ),
	BodyUint (fn (Receiver<Token>, Sender<Token>, uint) -> (), uint ),
	BodyVecUint (fn (Receiver<Token>, Sender<Token>, ~[uint]) -> (), ~[uint]),
	BodyFloat (fn (Receiver<Token>, Sender<Token>, f32) -> (), f32),
	BodyVecFloat (fn (Receiver<Token>, Sender<Token>, ~[f32]) -> (), ~[f32]),
	BodyFloatFloat (fn (Receiver<Token>, Sender<Token>, f32, f32) -> (), f32, f32),
	BodyFloatFloatFloat (fn (Receiver<Token>, Sender<Token>, f32, f32, f32) -> (), f32, f32, f32),
	Tail (fn (Receiver<Token>) -> () ), // stream g
	Fork,
	Funnel,
	Leg (~[Parts] ),
}

pub fn spinUp(fss: ~[Parts], mut ps: ~[Receiver<Token>]) -> Option<Receiver<Token>>{
	// spawn ports and channels
	let ret = match fss.iter().last().unwrap() {
		&Body(_) => true,
		&BodyUint(_, _) => true,
		&BodyVecUint(_, _) => true,
		&BodyFloat(_, _) => true,
		&BodyVecFloat(_, _) => true,
		&BodyFloatFloat(_, _, _) => true,
		&BodyFloatFloatFloat(_, _, _, _) => true,
		_ => false,
	};
	// iterate over functions
	for f in fss.move_iter() {
		let mut def = std::task::TaskOpts::new();
		match f {
			Head(g) => {
				let (c, p) = channel();
				ps.push(p);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(c) });
			},
			HeadFloat(g, v) => {
				let (c, p) = channel();
				ps.push(p);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(c, v) });
			}
			HeadFloatFloat(g, v1, v2) => {
				let (c, p) = channel();
				ps.unshift(p);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(c, v1, v2) });
			}
			HeadFloatFloatFloat(g, v1, v2, v3) => {
				println!("head: {:?}", ps.len());
				let (c, p) = channel();
				ps.unshift(p);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(c, v1, v2, v3) });
			}
			Body(g) => {
				println!("body: {:?}", ps.len());
				let (c, pn) = channel();
				let p = ps.shift().unwrap();
				ps.unshift(pn);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(p, c) });
			}
			BodyUint(g, v) => {
				println!("body: {:?}", ps.len());
				let (c, pn) = channel();
				let p = ps.shift().unwrap();
				ps.unshift(pn);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(p, c, v) });
			}
			BodyVecUint(g, v) => {
				println!("body: {:?}", ps.len());
				let (c, pn) = channel();
				let p = ps.shift().unwrap();
				ps.unshift(pn);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(p, c, v) });
			}
			BodyFloat(g, v) => {
				println!("body: {:?}", ps.len());
				let (c, pn) = channel();
				let p = ps.shift().unwrap();
				ps.unshift(pn);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(p, c, v) });
			}
			BodyVecFloat(g, v) => {
				println!("body: {:?}", ps.len());
				let (c, pn) = channel();
				let p = ps.shift().unwrap();
				ps.unshift(pn);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(p, c, v) });
			}
			BodyFloatFloat(g, v1, v2) => {
				println!("body: {:?}", ps.len());
				let (c, pn) = channel();
				let p = ps.shift().unwrap();
				ps.unshift(pn);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(p, c, v1, v2) });
			}
			BodyFloatFloatFloat(g, v1, v2, v3) => {
				println!("body: {:?}", ps.len());
				let (c, pn) = channel();
				let p = ps.shift().unwrap();
				ps.unshift(pn);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc() { g(p, c, v1, v2, v3) });
			}
			Tail(g) => {
				println!("tail: {:?}", ps.len());
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				let p = ps.pop().unwrap();
				task::spawn_opts(def, proc() { g(p) });
			}
			Fork => {
				println!("fork: {:?}", ps.len());
				let p = ps.shift().unwrap();
				let (c1, p1) = channel();
				let (c2, p2) = channel();
				def.name = Some(std::str::Owned(~"Fork"));
				task::spawn_opts(def, proc() {
					loop {
						let y = p.recv();
						c1.send(y.clone());
						c2.send(y);
					}
				});
				ps.unshift(p1);
				ps.unshift(p2);
			}
			Funnel => {
				println!("funnel: {:?}", ps.len());
				def.name = Some(std::str::Owned(~"Funnel"));
				let p1 = ps.pop().unwrap();
				let p2 = ps.pop().unwrap();
				let (c, p) = channel();
				ps.push(p);
				let x = c.clone();
				let y = c.clone();
				task::spawn_opts(def, proc() {
					loop {
						x.send(p1.recv());
					}
				});
				let mut def = std::task::TaskOpts::new();
				def.name = Some(std::str::Owned(~"Funnel"));
				task::spawn_opts(def, proc() {
					loop {
						y.send(p2.recv());
					}
				});
			},
			Leg(g) => {
				println!("leg: {:?}", ps.len());
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				let p = ps.shift().unwrap();
				match spinUp(g, ~[p]) {
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

