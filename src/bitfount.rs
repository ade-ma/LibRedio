/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-NC-SA 4.0. */
#[ crate_id = "bitfount" ];
#[ crate_type = "lib" ];

extern mod extra;
extern mod rtlsdr;
extern mod dsputils;
extern mod kpn;

use extra::complex;
use std::comm::Chan;
use kpn::{Symbol, Chit, SourceConf, Dbl, Packet};

// this is a stop-gap solution for demodulation - right now, it just triggers and discretizes against midpoint, outputting a sequence of symbols
// this works adequately for OOK / manchester encoded symbols, but will require refactoring to support FSK-type protocols

pub fn rtlSource(V: Chan<Symbol>, conf: SourceConf) {
	let bSize = 512;
	let devHandle = rtlsdr::openDevice();
	rtlsdr::setSampleRate(devHandle, conf.Rate as u32);
	rtlsdr::clearBuffer(devHandle);
	rtlsdr::setGain(devHandle, 402);
	rtlsdr::setFrequency(devHandle, conf.Freq as u32);

	let pdata = rtlsdr::readAsync(devHandle, bSize as u32);
	'main : loop {
		let samples = match pdata.recv_opt() {
			Some(x) => rtlsdr::dataToSamples(x),
			None => break 'main,
		};

		let normalized: ~[f64] = samples.iter().map(|x| x.norm()).collect();
		V.send(Packet(normalized.move_iter().map(|x| Dbl(x)).to_owned_vec()))
		//normalized.move_iter().map(|x| V.send(Dbl(x))).to_owned_vec();
	}
	rtlsdr::stopAsync(devHandle);
	rtlsdr::close(devHandle);
}

pub fn trigger(U: Port<Symbol>, V: Chan<Symbol>, conf: SourceConf) {
	let bSize = 512;

	// rtlsdr config
	let triggerDuration: int = 200;
	let mut trigger: int = 0;
	let mut sampleBuffer: ~[f64] = ~[0.0];
	let mut threshold: f64 = 0.0;

	'main: loop {
		trigger -= 1;
		let samples = match U.recv() {

			Packet(p) => p.move_iter().filter_map(|x| match x { Dbl(d) => Some(d), _ => None }).to_owned_vec(),
			_ => ~[],
		};
		let s = dsputils::sum(samples.clone());

		// wait for data or exit if data pipe is closed
		// if the buffer's too big, throw it away to prevent OOM
		if sampleBuffer.len() > 1000*triggerDuration as uint*bSize {
			sampleBuffer = ~[0.0];
			println!("{:?}", threshold);
		}

		// if we're just running, set the threshold equal to the sum of the first sample chunk
		if threshold == 0.0 {
			threshold = s;
		}

		// if we're not triggered, update threshold with the sum
		if trigger < 0 {
			threshold += s/10f64;
			threshold -= threshold*0.2f64;
		}

		// if the sum is greater than the threshold, trigger
		if s > threshold*4.0 {
			trigger = triggerDuration;
		}

		// if we're triggering, collect samples
		if trigger > 1 {
			sampleBuffer.push_all_move(samples);
		}

		// if we just finished triggering, filter, discretize, and send samples
		if trigger == 0 {
			println!("{:?}", (trigger, s, threshold));
			let filtered: ~[f64] = dsputils::convolve(sampleBuffer, dsputils::lpf(63, 0.02).move_iter().map(|x| x as f64).to_owned_vec());
			let max: f64 = dsputils::max(filtered.clone());
			let thresholded: ~[uint] = filtered.iter().map(|&x| { (x > max/2f64) as uint }).collect();
			for &v in thresholded.iter() {
				V.send(Chit(v));
			}
			sampleBuffer = ~[];
		}
	}

	// stop rtlsdr
}
