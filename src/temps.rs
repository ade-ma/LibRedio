/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */
extern crate bitfount;
extern crate dsputils;
extern crate kpn;
extern crate instant;
extern crate sensors;
extern crate vidsink;

use std::comm::{Port, Chan};
use instant::{Parts, Head, Body, Tail, Fork, Leg, Funnel};
use kpn::{Token, SourceConf};

fn pkt36(a: Port<Token>, b: Chan<Token>, c: SourceConf) {kpn::packetizer(a,b,c,36)}
fn pkt195(a: Port<Token>, b: Chan<Token>, c: SourceConf) {kpn::packetizer(a,b,c,195)}
fn pktTempA(a: Port<Token>, b: Chan<Token>, c: SourceConf) {kpn::decoder(a,b,c,~[14, 2, 12, 8])}
fn pktTempB(a: Port<Token>, b: Chan<Token>, c: SourceConf) {kpn::decoder(a,b,c,~[15, 6, 5, 8, 2, 9, 1, 4, 2, 9, 4])}

fn main() {
	let conf = SourceConf{Freq: 434e6, Rate: 1.024e6, Period: 5e-4};
	// flowgraph leg for silver&black temp sensor
	let t1 = ~[Body(sensors::validTokenA), Body(pkt36), Body(pktTempA), Body(sensors::sensorUnpackerA)];//, Fork(kpn::tuplicator), Leg(~[Tail(vidsink::vidSink)]), Leg(~[Tail(kpn::printdump)])];
	// flowgraph leg for white temp sensors
	let t2 = ~[Body(kpn::validTokenManchester), Body(kpn::manchesterd), Body(pkt195), Body(pktTempB), Body(sensors::sensorUnpackerB)];
	// main flowgraph
	let parsing: ~[Parts] = ~[Body(bitfount::discretize), Body(kpn::rle), Body(kpn::dle), Fork(kpn::tuplicator), Leg(t1), Leg(t2), Funnel(kpn::twofunnel), Tail(kpn::udpTokenSink)];
	let fs: ~[Parts] = ~[Head(bitfount::rtlSource), Body(bitfount::trigger), Fork(kpn::tuplicator), Leg(~[Tail(vidsink::vidSink)]), Leg(parsing)];
	// spawn
	instant::spinUp(fs, ~[], conf);
	loop {}
}
