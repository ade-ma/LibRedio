/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-NC-SA 4.0. */
extern mod extra;
extern mod bitfount;
extern mod dsputils;
extern mod kpn;
extern mod native;
extern mod instant;
extern mod unpackers;

use instant::{Parts, Head, Body, Tail, Fork, Leg};
use kpn::{Symbol, SourceConf};
use unpackers::{tempSinkA, tempSinkB};

fn main() {
	let conf = SourceConf{Freq: 434e6, Rate: 1.024e6, Period: 5e-4};
	let t1 = ~[Body(kpn::validSymbolTemp), Tail(tempSinkA)];
	let t2 = ~[Body(kpn::validSymbolManchester), Body(kpn::manchesterd), Tail(tempSinkB)];
	let fs: ~[Parts] = ~[Head(bitfount::bitfount), Body(kpn::rle), Body(kpn::dle), Fork(kpn::tuplicator), Leg(t1), Leg(t2)];
	instant::spinUp(fs, ~[], conf);
	loop {}
}
