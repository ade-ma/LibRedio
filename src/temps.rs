/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */
extern mod extra;
extern mod bitfount;
extern mod dsputils;
extern mod kpn;
extern mod native;
extern mod instant;

use std::comm::{Port, Chan};
use instant::{Parts, Head, Body, Tail, Fork, Leg};
use kpn::{Symbol, SourceConf};

fn pkt36(a: Port<Symbol>, b: Chan<Symbol>, c: SourceConf) {kpn::packetizer(a,b,c,36)}
fn pkt195(a: Port<Symbol>, b: Chan<Symbol>, c: SourceConf) {kpn::packetizer(a,b,c,195)}
fn pktTempA(a: Port<Symbol>, b: Chan<Symbol>, c: SourceConf) {kpn::decoder(a,b,c,~[14, 2, 12, 8])}
fn pktTempB(a: Port<Symbol>, b: Chan<Symbol>, c: SourceConf) {kpn::decoder(a,b,c,~[15, 6, 5, 8, 2, 9, 1, 4, 2, 9, 4])}

fn main() {
	let conf = SourceConf{Freq: 434e6, Rate: 1.024e6, Period: 5e-4};
	// flowgraph leg for silver&black temp sensor
	let t1 = ~[Body(kpn::validSymbolTemp), Body(pkt36), Body(pktTempA), Tail(kpn::printdump)];
	// flowgraph leg for white temp sensors
	let t2 = ~[Body(kpn::validSymbolManchester), Body(kpn::manchesterd), Body(pkt195), Body(pktTempB), Tail(kpn::printdump)];
	// main flowgraph
	let fs: ~[Parts] = ~[Head(bitfount::rtlSource), Body(bitfount::trigger), Body(kpn::rle), Body(kpn::dle), Fork(kpn::tuplicator), Leg(t1), Leg(t2)];
	// spawn
	instant::spinUp(fs, ~[], conf);
	loop {}
}
