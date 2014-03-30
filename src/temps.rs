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
	let t1 = ~[Body(sensors::validTokenA), BodyUint(kpn::packetizer, 36), BodyVecUint(kpn::decoder, ~[2, 12, 2, 4, 4, 4, 4, 4]), Body(sensors::sensorUnpackerA)];
	// flowgraph leg for white temp sensors
	let t2 = ~[BodyFloatFloat(kpn::validTokenManchester, 5e-4, 0.4), Body(kpn::manchesterd), BodyUint(kpn::packetizer, 198), BodyVecUint(kpn::decoder, ~[17, 6, 5, 8, 2, 9, 1, 4, 2, 9, 4]), Body(sensors::sensorUnpackerB)];
	let t3 = ~[BodyFloatFloat(kpn::validTokenManchester, 5.5e-4, 0.4), Body(kpn::manchesterd), BodyUint(kpn::packetizer, 89)];
	// main flowgraph
	let parsing = ~[Body(bitfount::discretize), Body(kpn::rle), BodyFloat(kpn::dle, 0.256e6), Fork, Fork, Leg(t1), Leg(t2), Leg(t3), Funnel, Funnel, Fork, Tail(kpn::printSink), Tail(kpn::udpTokenSink)];
	let fs = ~[HeadFloatFloatFloat(bitfount::rtlSource, 434e6, 40.2, 0.256e6), Body(bitfount::trigger), Body(bitfount::filter), Leg(parsing)];
	// spawn
	instant::spinUp(fs, ~[]);
	let (c, p) = channel();
	loop {
		p.recv()
	}
}
