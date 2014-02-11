/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-NC-SA 4.0. */

extern mod extra;
extern mod bitfount;
extern mod dsputils;
extern mod kpn;

use std::comm::{Port,Chan};
use dsputils::eat;
use kpn::{Break, Chit, Symbol, SourceConf};

// support grey-and-black temperature and humidity sensors
pub fn tempSinkA(P: Port<Symbol>, Q: SourceConf) {
	let mut packets: ~[~[uint]] = ~[];
	loop {
		let mut bits: ~[uint] = ~[];
		'recv: loop {
			match P.recv() {
				Break(_) => break 'recv,
				Chit(x) => bits.push(x),
				x => println!("{:?}", x)
			}
		}
		if bits.len() == 36 {
			let packet = eat(bits, ~[14, 2, 12, 8]);
			packets.push(packet)
		}
		println!("{}", packets.len());
		if packets.len() == 4 {
			match (&packets[0], &packets[1], &packets[2], &packets[3]) {
				(a, b, c, d) if (b == a) || (b == c) || (b == d) => {
					println!("p: {:x}", b[0]);
					println!("s: {}", b[1]+1);
					println!("t: {} degC", b[2] as f32 / 10f32);
					println!("h: {} %", b[3]);
				},
				_ => ()
			}
			packets = ~[];
		}
	}
}

// support white temperature / temperature and humidity sensors
pub fn tempSinkB(P: Port<Symbol>, Q: SourceConf) {
	loop {
		let mut bits: ~[uint] = ~[];
		'recv: loop {
			match P.recv() {
				Break(_) => break 'recv,
				Chit(x) => bits.push(x),
				x => println!("{:?}", x)
			}
		}
		let x = match bits.len() {
			184 => {
				println!("184");
				let a = bits.slice(4,54);
				let b = bits.slice(54,119).slice_from(15);
				let c = bits.slice_from(119).slice_from(15);
				if ( a == b ) || ( a == c ) { a } else if ( b == c ) || (b == a) { b } else { bits.slice_to(0) }
			},
			195 => {
				println!("195");
				let a = bits.slice(15,65);
				let b = bits.slice(65,130).slice_from(15);
				let c = bits.slice_from(130).slice_from(15);
				if ( a == b ) || ( a == c ) { a } else if ( b == c ) || (b == a) { b } else { bits.slice_to(0) }
			},
			_ => bits.slice_to(0),
		};
		if x.len() == 50 {
			let packet = eat(x, ~[6, 5, 8, 2, 9, 1, 4, 2, 9, 4]);
			println!("{:?}", packet);
		}
	}
}
