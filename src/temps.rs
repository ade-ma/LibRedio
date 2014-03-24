/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */
#[feature(globs)];

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
	let t1 = ~[Body(sensors::validTokenA), BodyUint(kpn::packetizer, 1000)];
	// flowgraph leg for white temp sensors
	let t2 = ~[BodyDouble(kpn::validTokenManchester, 5e-4), Body(kpn::manchesterd), BodyUint(kpn::packetizer, 1000)];
	let t3 = ~[BodyDouble(kpn::validTokenManchester, 5.5e-4), Body(kpn::manchesterd), BodyUint(kpn::packetizer, 1000)];
	// main flowgraph
	let parsing: ~[Parts] = ~[Body(bitfount::discretize), Body(kpn::rle), BodyDouble(kpn::dle, 0.256e6), Fork, Fork, Leg(t1), Leg(t3), Leg(t2), Funnel, Funnel, Tail(kpn::printSink)];
	let fs: ~[Parts] = ~[HeadDoubleDoubleDouble(bitfount::rtlSource, 434e6, 40.2, 0.256e6), Body(bitfount::trigger), Body(bitfount::filter), Leg(parsing)];
	// spawn
	instant::spinUp(fs, ~[]);
	let (c, p) = channel();
	loop {
		p.recv()
	}
}
