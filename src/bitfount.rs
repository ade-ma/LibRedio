/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of GPLv3. */
#[ crate_id = "bitfount" ];
#[ crate_type = "lib" ];

extern crate num;
extern crate rtlsdr;
extern crate dsputils;

use num::complex::Complex;
use std::comm::Sender;

pub fn rtlSourceCmplx(v: Sender<Vec<Complex<f32>>>, cFreq: u32, gain: u32, sRate: u32) {
	let bSize = 512;
	let devHandle = rtlsdr::openDevice();
	rtlsdr::setSampleRate(devHandle, sRate);
	rtlsdr::clearBuffer(devHandle);
	rtlsdr::setGain(devHandle, gain);
	rtlsdr::setFrequency(devHandle, cFreq);

	let pdata = rtlsdr::readAsync(devHandle, bSize);
	'main : loop {
		let samples = match pdata.recv_opt() {
			Ok(x) => rtlsdr::dataToSamples(x),
			Err(_) => break 'main,
		};
		v.send(samples);
	}
	rtlsdr::stopAsync(devHandle);
	rtlsdr::close(devHandle);
}

pub fn trigger(u: Receiver<Vec<f32>>, v: Sender<Vec<f32>>) {
	let bSize = 512;

	// rtlsdr config
	let triggerDuration: int = 50;
	let mut trigger: int = 0;
	let mut sampleBuffer: Vec<f32> = vec!(0.0);
	let mut threshold: f32 = 0.0;

	'main: loop {
		trigger -= 1;
		let samples = u.recv();
		let s = dsputils::sum(samples.slice_from(0));

		// wait for data or exit if data pipe is closed
		// if the buffer's too big, throw it away to prevent OOM
		if sampleBuffer.len() > 1000*triggerDuration as uint*bSize {
			sampleBuffer = vec!(0.0);
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
			sampleBuffer.push_all(samples.slice_from(0));
		}

		// if we just finished triggering, filter, discretize, and send samples
		if trigger == 0 {
			v.send(sampleBuffer);
			sampleBuffer = vec!();
		}
	}

	// stop rtlsdr
}

pub fn filter(u: Receiver<Vec<f32>>, v: Sender<Vec<f32>>) {
	let lpf: Vec<f32> = dsputils::lpf(63, 0.02).move_iter().map(|x| x as f32).collect();
	loop {
		let x = u.recv();
		let filtered: Vec<f32> = dsputils::convolve(x.slice_from(0), lpf.slice_from(0));
		v.send(filtered);
	}
}

pub fn discretize(u: Receiver<Vec<f32>>, v: Sender<uint>) {
	loop {
		let sampleBuffer = u.recv();
		let max: f32 = dsputils::max(sampleBuffer.slice_from(0));
		let thresholded: Vec<uint> = sampleBuffer.iter().map(|&x| { (x > max/2f32) as uint }).collect();
		for &x in thresholded.iter() {
			v.send(x);
		}
	}
}
