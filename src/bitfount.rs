/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-NC-SA 4.0. */
#[ crate_id = "bitfount" ];
#[ crate_type = "lib" ];

extern crate num;
extern crate rtlsdr;
extern crate dsputils;
extern crate kpn;

use num::complex;
use std::comm::Sender;
use kpn::{Token, Chip, Flt, Packet};

pub fn rtlSource(v: Sender<Token>, cFreq: f32, gain: f32, sRate: f32) {
	let bSize = 512;
	let devHandle = rtlsdr::openDevice();
	rtlsdr::setSampleRate(devHandle, sRate as u32);
	rtlsdr::clearBuffer(devHandle);
	rtlsdr::setGain(devHandle, (gain * 10.0) as u32);
	rtlsdr::setFrequency(devHandle, cFreq as u32);

	let pdata = rtlsdr::readAsync(devHandle, bSize as u32);
	'main : loop {
		let samples = match pdata.recv_opt() {
			Some(x) => rtlsdr::dataToSamples(x),
			None => break 'main,
		};

		let normalized: ~[f32] = samples.iter().map(|x| x.norm()).collect();
		v.send(Packet(normalized.move_iter().map(|x| Flt(x)).collect()))
	}
	rtlsdr::stopAsync(devHandle);
	rtlsdr::close(devHandle);
}

pub fn rtlSourceSync(v: Sender<Token>, cFreq: f32, gain: f32, sRate: f32) {
	let bSize = 512;
	let devHandle = rtlsdr::openDevice();
	rtlsdr::setSampleRate(devHandle, sRate as u32);
	rtlsdr::clearBuffer(devHandle);
	rtlsdr::setGain(devHandle, (gain * 10.0) as u32);
	rtlsdr::setFrequency(devHandle, cFreq as u32);

	'main : loop {
		let x = rtlsdr::readSync(devHandle, bSize as u32);
		let samples = rtlsdr::dataToSamples(x);
		let normalized: ~[f32] = samples.iter().map(|x| (x.re*x.re+x.im*x.im).sqrt()).collect();
		v.send(Packet(normalized.move_iter().map(|x| Flt(x)).collect()))
	}
	rtlsdr::close(devHandle);
}

pub fn trigger(u: Receiver<Token>, v: Sender<Token>) {
	let bSize = 512;

	// rtlsdr config
	let triggerDuration: int = 50;
	let mut trigger: int = 0;
	let mut sampleBuffer: ~[f32] = ~[0.0];
	let mut threshold: f32 = 0.0;

	'main: loop {
		trigger -= 1;
		let samples = match u.recv() {
			Packet(p) => p.move_iter().filter_map(|x| match x { Flt(d) => Some(d), _ => None }).collect(),
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
			threshold += s/1000f32;
			threshold -= threshold*0.002f32;
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
			v.send(Packet(sampleBuffer.move_iter().map(|x| Flt(x)).collect()));
			println!("{:?}", (trigger, s, threshold));
			sampleBuffer = ~[];
		}
	}

	// stop rtlsdr
}

pub fn filter(u: Receiver<Token>, v: Sender<Token>) {
	let lpf: ~[f32] = dsputils::lpf(63, 0.02).move_iter().map(|x| x as f32).collect();
	loop {
		let sampleBuffer = match u.recv() {
			Packet(p) => p.move_iter().filter_map(|x| match x { Flt(d) => Some(d), _ => None}).collect(),
			_ => ~[],
		};
		let filtered: ~[f32] = dsputils::convolve(sampleBuffer, lpf.slice_from(0));
		v.send(Packet(filtered.move_iter().map(|x| Flt(x)).collect()));
	}
}

pub fn discretize(u: Receiver<Token>, v: Sender<Token>) {
	loop {
		let sampleBuffer = match u.recv() {
			Packet(p) => p.move_iter().filter_map(|x| match x { Flt(d) => Some(d), _ => None}).collect(),
			_ => ~[],
		};
		println!("{:?}", sampleBuffer.len());
		let max: f32 = dsputils::max(sampleBuffer.slice_from(0));
		let thresholded: ~[uint] = sampleBuffer.iter().map(|&x| { (x > max/2f32) as uint }).collect();
		for &x in thresholded.iter() {
			v.send(Chip(x));
		}
	}
}
