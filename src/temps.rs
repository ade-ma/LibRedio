/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */
extern crate bitfount;
extern crate dsputils;
extern crate kpn;
extern crate instant;
extern crate sensors;

use std::comm::{Port, Chan};
use instant::{Parts, Head, Body, Tail, Fork, Leg, Funnel};
use kpn::{Token, SourceConf};

fn pkt1k(a: Port<Token>, b: Chan<Token>, c: SourceConf) {kpn::packetizer(a,b,c,1000)}

fn main() {
	let conf = SourceConf{Freq: 434e6, Rate: 0.256e6, Period: 5e-4};
	// flowgraph leg for silver&black temp sensor
	let t1 = ~[Body(sensors::validTokenA), Body(pkt1k)];
	// flowgraph leg for white temp sensors
	let t2 = ~[Body(kpn::validTokenManchester), Body(kpn::manchesterd), Body(pkt1k)];
	// main flowgraph
	let parsing: ~[Parts] = ~[Body(bitfount::discretize), Body(kpn::rle), Body(kpn::dle), Fork(kpn::tuplicator), Fork(kpn::tuplicator), Tail(kpn::printSink), Leg(t1), Leg(t2), Funnel(kpn::twofunnel), Fork(kpn::tuplicator), Tail(kpn::udpTokenSink), Tail(kpn::printSink)];
	let fs: ~[Parts] = ~[Head(bitfount::rtlSource), Body(bitfount::trigger), Body(bitfount::filter), Leg(parsing)];
	// spawn
	instant::spinUp(fs, ~[], ~[], conf);
	let (p, c) = Chan::new();
	loop {
		p.recv()
	}
}
