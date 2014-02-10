/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-NC-SA 4.0. */
extern mod extra;
extern mod bitfount;
extern mod dsputils;
extern mod kpn;
extern mod native;
extern mod instant;
extern mod unpackers;

use instant::{Parts, Head, Body, Tail};
use kpn::{Break, Chit, Symbol, SourceConf};
use unpackers::tempSinkA;

fn main() {
	let conf = SourceConf{Freq: 433.8e6, Rate: 1.024e6, Period: 0.0};
	let fs: ~[Parts] = ~[Head(bitfount::bitfount), Body(kpn::rle), Body(kpn::dle), Body(kpn::validSymbolTemp), Tail(tempSinkA)];
	instant::spinUp(fs, conf);
	loop {}

}
