/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of GPLv3. */

extern crate num;
extern crate rtlsdr;
extern crate dsputils;

use num::complex::Complex;
use std::sync::mpsc::{Receiver, Sender, channel, Handle, Select};
use rtlsdr::RTLSDR_Dev;

pub fn rtl_source_cmplx(v: Sender<Vec<Complex<f32>>>, c_freq: u32, gain: u32, s_rate: u32) {
	let block_size = 512;
	let dev_handle = rtlsdr::RTLSDR_Dev::open();
	dev_handle.set_sample_rate(s_rate);
	dev_handle.clear_buffer();
	dev_handle.set_gain(gain);
	dev_handle.set_frequency(c_freq);

	let pdata = dev_handle.read_async(block_size);
	'main : loop {
		let samples = match pdata.recv() {
			Ok(x) => rtlsdr::data_to_samples(x),
			Err(_) => break 'main,
		};
		v.send(samples);
	}
	dev_handle.stop_async();
	dev_handle.close();
}

pub fn trigger(u: Receiver<Vec<f32>>, v: Sender<Vec<f32>>) {
    use std::iter::AdditiveIterator;
	let block_size = 512;

	// rtlsdr config
	let trigger_duration: isize = 50;
	let mut trigger: isize = 0;
	let mut sample_buffer: Vec<f32> = vec!(0.0);
	let mut threshold: f32 = 0.0;
	'main: loop {
		trigger -= 1;
		let samples = u.recv().unwrap();
        let s = samples.iter().map(|&x|x).sum();

		// wait for data or exit if data pipe is closed
		// if the buffer's too big, throw it away to prevent OOM
		if sample_buffer.len() > 1000*trigger_duration as usize*block_size {
			sample_buffer = vec!(0.0);
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
			trigger = trigger_duration;
		}

		// if we're triggering, collect samples
		if trigger > 1 {
			sample_buffer.push_all(samples.slice_from(0));
		}

		// if we just finished triggering, filter, discretize, and send samples
		if trigger == 0 {
			v.send(sample_buffer);
			sample_buffer = vec!();
		}
	}

	// stop rtlsdr
}

pub fn discretize(u: Receiver<Vec<f32>>, v: Sender<usize>) {
	loop {
		let sample_buffer = u.recv().unwrap();
		let max: f32 = dsputils::max(sample_buffer.slice_from(0));
		let thresholded: Vec<usize> = sample_buffer.iter().map(|&x| { (x > max/2f32) as usize }).collect();
		for &x in thresholded.iter() {
			v.send(x);
		}
	}
}
