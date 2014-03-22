extern crate kpn;
extern crate time;

use kpn::{Token, Chip, Packet, SourceConf, Dbl, Break, Dur};
use std::comm::{Receiver, Sender};

// temperature sensor pulse duration modulated binary protocol symbol matcher
pub fn validTokenA(U: Receiver<Token>, V: Sender<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Dur(~va, dura) => {
				if (va == Chip(1)) && (4e-4 < dura) && (dura < 6e-4) {
					match U.recv() {
						Dur(_, durb) => {
							if (1.7e-3 < durb) && (durb < 2.2e-3) {V.send(Chip(0))}
							else if (3.6e-3 < durb) && (durb < 4.2e-3) {V.send(Chip(1))}
							else if durb > 8.7e-3 {V.send(Break("silence"))}
						},
						_ => ()
					}
				}
			}
			_=> ()
		}
	}
}

pub fn sensorUnpackerA(U: Receiver<Token>, V: Sender<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Packet(p) => {
				let now = time::get_time();
				let t: f64 = now.sec as f64 + now.nsec as f64 * 1e-9;
				let l = p.clone().move_iter().filter_map(|x| match x { Chip(c) => Some(c), _ => None }).to_owned_vec();
				V.send(Packet(~[Packet(p.clone()), 
					Chip(0), Chip(l[0]+l[1]), Dbl(l[2] as f64 / 10f64), 
					Dbl(t)]));
				V.send(Packet(~[Packet(p.clone()), 
					Chip(1), Chip(l[0]+l[1]), Dbl(l[3] as f64), 
					Dbl(t)]));
			},
			x => println!("{:?}", x),
		}
	}
}

pub fn sensorUnpackerB(U: Receiver<Token>, V: Sender<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Packet(p) => {
				let now = time::get_time();
				let t: f64 = now.sec as f64 + now.nsec as f64 * 1e-9;
				let l = p.clone().move_iter().filter_map(|x| match x { Chip(c) => Some(c), _ => None }).to_owned_vec();
				let mut v = l[5] as f64;
				if l[4] == 1 { v = 16.6 - 0.057*(512.0-v);} // magic hardware specific numbers
				else { v = v * 0.057 + 16.6 };
				V.send(Packet(~[Packet(p.clone()), Chip(0), Chip(l[2]), Dbl(v), Dbl(t)]));
			},
			x => println!("{:?}", x),
		}
	}
}
