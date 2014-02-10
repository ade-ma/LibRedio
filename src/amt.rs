/* Copyright Ian Daniher, 2013, 2014.
Distributed under the terms of CC BY-NC-SA 4.0. */

extern mod extra;
extern mod bitfount;
extern mod dsputils;
extern mod kpn;
extern mod instant;
extern mod unpackers;

use instant::{Parts, Head, Body, Tail};
use kpn::{Symbol, SourceConf};
use unpackers::tempSinkB;

fn main() {
	let conf = SourceConf{Freq: 434e6, Rate: 1.024e6, Period: 5e-4};
	let fs = ~[Head(bitfount::bitfount), Body(kpn::rle), Body(kpn::dle), Body(kpn::validSymbolManchester), Body(kpn::manchesterd), Tail(tempSinkB)];
	instant::spinUp(fs, conf);
	loop {}
}
