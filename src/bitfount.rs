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
use kpn::{Symbol, Chit, SourceConf};

// this is a stop-gap solution for demodulation - right now, it just triggers and discretizes against midpoint, outputting a sequence of symbols
// this works adequately for OOK / manchester encoded symbols, but will require refactoring to support FSK-type protocols

pub fn bitfount(outChan: Chan<Symbol>, conf: SourceConf) {

	// rtlsdr config
	let bSize = 512;
	let devHandle = rtlsdr::openDevice();
	rtlsdr::setSampleRate(devHandle, conf.Rate as u32);
	rtlsdr::clearBuffer(devHandle);
	rtlsdr::setGain(devHandle, 402);
	rtlsdr::setFrequency(devHandle, conf.Freq as u32);

	let pdata = rtlsdr::readAsync(devHandle, bSize as u32);

	let triggerDuration: int = 200;
	let mut trigger: int = 0;
	let mut sampleBuffer: ~[f32] = ~[0.0];
	let mut threshold: f32 = 0.0;

	'main: loop {

		// wait for data or exit if data pipe is closed
		let data = match pdata.recv_opt() {
			Some(x) => x,
			None => break 'main,
		};

		let samples: ~[complex::Complex32] = rtlsdr::dataToSamples(data);
		let normalized: ~[f32] = samples.iter().map(|x| x.norm()).collect();
		let s = dsputils::sum(normalized.clone());
		trigger -= 1;

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
			threshold += s/10f32;
			threshold -= threshold*0.2f32;
		}

		// if the sum is greater than the threshold, trigger
		if s > threshold*4.0 {
			trigger = triggerDuration;
		}

		// if we're triggering, collect samples
		if trigger > 1 {
			sampleBuffer.push_all_move(normalized);
		}

		// if we just finished triggering, filter, discretize, and send samples
		if trigger == 0 {
			println!("{:?}", (trigger, s, threshold));
			let filtered: ~[f32] = dsputils::convolve(sampleBuffer, dsputils::lpf(63, 0.02));
			let max: f32 = dsputils::max(filtered.clone());
			let thresholded: ~[uint] = filtered.iter().map(|&x| { (x > max/2f32) as uint }).collect();
			for &v in thresholded.iter() {
				outChan.send(Chit(v));
			}
			sampleBuffer = ~[];
		}
	}

	// stop rtlsdr
	rtlsdr::stopAsync(devHandle);
	rtlsdr::close(devHandle);
}
