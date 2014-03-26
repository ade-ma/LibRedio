#[feature(globs)];
extern crate kpn;
extern crate time;

use kpn::*;
use std::comm::{Receiver, Sender};

// temperature sensor pulse duration modulated binary protocol symbol matcher
pub fn validTokenA(u: Receiver<Token>, v: Sender<Token>) {
	loop {
		match u.recv() {
			Dur(~va, dura) => {
				if (va == Chip(1)) && (4e-4 < dura) && (dura < 6e-4) {
					match u.recv() {
						Dur(_, durb) => {
							if (1.7e-3 < durb) && (durb < 2.2e-3) {v.send(Chip(0))}
							else if (3.6e-3 < durb) && (durb < 4.2e-3) {v.send(Chip(1))}
							else if durb > 8.7e-3 {v.send(Break("silence"))}
						},
						_ => ()
					}
				}
			}
			_=> ()
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
						v.send(Packet(~[Packet(p.clone()),
							Chip(0), Chip(l[1]+l[2]), Flt((l[4]*16+l[5]) as f32 / 10f32),
							Flt(t)]));
						v.send(Packet(~[Packet(p.clone()),
							Chip(1), Chip(l[1]+l[2]), Flt((l[6]*16+l[7]) as f32),
							Flt(t)]));
					}
					2 => {
						v.send(Packet(~[Packet(p.clone()),
							Chip(0), Chip(l[1]+l[2]), Flt((l[3]*16+l[4]) as f32 / 10f32),
							Flt(t)]));
						v.send(Packet(~[Packet(p.clone()),
							Chip(1), Chip(l[1]+l[2]), Flt((l[5]*16+l[6]) as f32 / 1.5f32 - 6f32),
							Flt(t)]));
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
				v.send(Packet(~[Packet(p.clone()), Chip(0), Chip(l[2]), Flt(x), Flt(t)]));
			},
			y => println!("{:?}", y),
		}
	}
}
