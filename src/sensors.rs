#![feature(globs)]
extern crate kpn;
extern crate time;

use kpn::*;
use std::comm::{Receiver, Sender};

// temperature sensor pulse duration modulated binary protocol symbol matcher
pub fn validTokenA(u: Receiver<Token>, v: Sender<Token>) {
	let mut x: Token = Chip(0);
	loop {
		match (x.clone(), u.recv()) {
			(Dur(~Chip(1), 1e-6..6e-4), Dur(~Chip(0), 1.7e-3..2.6e-3)) => {v.send(Chip(0))}
			(Dur(~Chip(1), 1e-6..6e-4), Dur(~Chip(0), 3.6e-3..4.6e-3)) => {v.send(Chip(1))}
			(_, Dur(~Chip(0), 8.7e-3..1e1)) => {v.send(Break("silence"))}
			(Dur(~Chip(0), 8.7e-3..1e1), _) => {v.send(Break("silence"))}
			(a, b) => {println!("{:?}", (&a,&b)); x = b.clone();}
		}
	}
}

pub fn sensorUnpackerA(u: Receiver<Token>, v: Sender<Token>) {
	loop {
		match u.recv() {
			Packet(p) => {
				let now = time::get_time();
				let t: f32 = now.sec as f32 + now.nsec as f32 * 1e-9;
				let l: ~[uint] = p.clone().move_iter().filter_map(|x| match x { Chip(c) => Some(c), _ => None }).collect();
				match l[0] {
					1 => {
						v.send(Packet(vec!(Packet(p.clone()),
							Chip(0), Chip(l[1]+l[2]), Flt((l[4]*16+l[5]) as f32 / 10f32),
							Flt(t))));
						v.send(Packet(vec!(Packet(p.clone()),
							Chip(1), Chip(l[1]+l[2]), Flt((l[6]*16+l[7]) as f32),
							Flt(t))));
					}
					2 => {
						v.send(Packet(vec!(Packet(p.clone()),
							Chip(0), Chip(l[1]), Flt((l[2]*256+l[3]*16+l[4]) as f32 / 10f32),
							Flt(t))));
						v.send(Packet(vec!(Packet(p.clone()),
							Chip(1), Chip(l[1]), Flt((l[5]*16+l[6]) as f32 / 1.5f32 - 6f32),
							Flt(t))));
					}
					_ => println!("{:?}", &l)
			};
			},
			x => println!("{:?}", x),
		}
	}
}



pub fn sensorUnpackerB(u: Receiver<Token>, v: Sender<Token>) {
	loop {
		match u.recv() {
			Packet(p) => {
				let now = time::get_time();
				let t: f32 = now.sec as f32 + now.nsec as f32 * 1e-9;
				let l: ~[uint] = p.clone().move_iter().filter_map(|x| match x { Chip(c) => Some(c), _ => None }).collect();
				let mut x = l[5] as f32;
				if l[4] == 1 { x = 16.6 - 0.057*(512.0-x);} // magic hardware specific numbers
				else { x = x * 0.057 + 16.6 };
				v.send(Packet(vec!(Packet(p.clone()), Chip(0), Chip(l[2]), Flt(x), Flt(t))));
			},
			y => println!("{:?}", y),
		}
	}
}
