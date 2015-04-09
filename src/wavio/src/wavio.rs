#![feature(collections)]
#![feature(convert)]


extern crate sndfile;
extern crate num;

use num::complex::Complex;
use std::sync::mpsc::{Receiver, Sender, channel, Handle, Select};
use sndfile::OpenMode::Read;

pub fn wav_source_f32(u: Sender<f32>, fname: &str, s_rate: u32) {
	let mut sndf = sndfile::SndFile::new(fname, Read).unwrap();
	let info = sndf.get_sndinfo();
	assert_eq!(info.samplerate as u32, s_rate);
	assert_eq!(info.channels as u32, 1);
	let mut x: Vec<f32> = Vec::new();
	x.push_all(&[0f32; 1024]);
	for _ in (0..(info.frames/2)/1024) {
		sndf.read_f32(x.as_mut_slice(), 1024);
		for &z in x.iter() {
			u.send(z).unwrap()
		}
	}
	let (c, p) = channel();
	p.recv().unwrap();
	c.send(()).unwrap();
}

pub fn wav_source_complex_f32(u: Sender<Complex<f32>>, fname: &str, s_rate: u32) {
	let mut sndf = sndfile::SndFile::new(fname, Read).unwrap();
	let info = sndf.get_sndinfo();
	assert_eq!(info.samplerate as u32, s_rate);
	assert_eq!(info.channels as u32, 2);
	let mut x: Vec<f32> = Vec::new();
	x.push_all(&[0f32; 1024]);
	for _ in (0..(info.frames/2)/1024) {
		sndf.read_f32(x.as_mut_slice(), 1024);
		for z in x.as_slice().chunks(2) {
			u.send(num::Complex{re: z[0], im: z[1]}).unwrap();
		}
	}
	let (c, p) = channel();
	p.recv().unwrap();
	c.send(()).unwrap();
}
