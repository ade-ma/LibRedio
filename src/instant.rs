/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3 */

extern crate native;
extern crate kpn;
extern crate bitfount;

use kpn::Token;
use native::task;

pub enum Parts<T>{
	Head (proc (Sender<T>):Send),
	Body (proc (Receiver<T>, Sender<T>):Send),
	Tail (proc (Receiver<T>):Send),
	Leg (Vec<Parts<T>>),
	Fork,
	Funnel,
}

pub fn spinUp<T: Send+Clone>(fss: Vec<Parts<T>>, mut ps: Vec<Receiver<T>>) -> Option<Receiver<T>>{
	// spawn ports and channels
	let ret = match fss.iter().last().unwrap() {
		&Body(_) => true,
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
				task::spawn_opts(def, proc(){g(c)});
			},
			Body(g) => {
				println!("body: {:?}", ps.len());
				let (c, pn) = channel();
				let p = ps.shift().unwrap();
				ps.unshift(pn);
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				task::spawn_opts(def, proc(){g(p, c)});
			}
			Tail(g) => {
				println!("tail: {:?}", ps.len());
				def.name = Some(std::str::Owned(format!("{:?}", g)));
				let p = ps.pop().unwrap();
				task::spawn_opts(def, proc(){g(p)});
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
				match spinUp(g, vec!(p)) {
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

