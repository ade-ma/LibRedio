/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */
extern crate bitfount;
extern crate dsputils;
extern crate kpn;
extern crate instant;
extern crate sensors;

use std::comm::{Receiver, Sender};
use instant::{Parts, Head, Body, Tail, Fork, Leg, Funnel};
use kpn::{Token, SourceConf};

fn pkt36(a: Receiver<Token>, b: Sender<Token>, c: SourceConf) {kpn::packetizer(a,b,c,36)}
fn pkt1k(a: Receiver<Token>, b: Sender<Token>, c: SourceConf) {kpn::packetizer(a,b,c,1000)}

fn main() {
	let conf = SourceConf{Freq: 434e6, Rate: 0.256e6, Period: 5.5e-4};
	let fs: ~[Parts] = ~[Head(bitfount::rtlSource), Body(bitfount::trigger), Body(bitfount::filter), Body(bitfount::discretize), Body(kpn::rle), Body(kpn::dle), Body(kpn::validTokenManchester), Body(kpn::manchesterd), Body(pkt1k), Tail(kpn::printSink)];
	instant::spinUp(fs, ~[], ~[], conf);
	let (c, p) = channel();
	loop {
		p.recv()
	}
}
