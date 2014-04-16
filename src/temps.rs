/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */
#![feature(globs)]

extern crate bitfount;
extern crate dsputils;
extern crate kpn;
extern crate instant;
extern crate sensors;

use std::comm::{Receiver, Sender, channel};
use instant::*;
use kpn::{Token};

fn main() {
	// flowgraph leg for silver&black temp sensor
	let t1 = ~[Body(sensors::validTokenA), Body(proc(p,c) {kpn::packetizer(p, c, 36)}), Body(proc(p, c) {kpn::decoder(p,c, ~[2, 12, 2, 4, 4, 4, 4, 4])}), Body(sensors::sensorUnpackerA)];
	let parsing = ~[Body(bitfount::discretize), Body(kpn::rle), Body(proc(p,c){kpn::dle(p,c, 0.256e6 as uint)}), Leg(t1), Tail(kpn::printSink)];
	let fs = ~[Head(proc(c){bitfount::rtlSource(c, 434e6 as u32, 402, 0.256e6 as u32)}), Body(bitfount::trigger), Body(bitfount::filter), Leg(parsing)];
	// spawn
	instant::spinUp(fs, ~[]);
	let (c, p) = channel();
	loop {
		p.recv()
	}
}
