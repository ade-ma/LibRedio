extern crate sndfile;
extern crate num;

use num::complex::Complex;
use std::comm::Sender;

pub fn wavSourceF32(u: Sender<f32>, fname: &str, sRate: u32) {
	let mut sndf = sndfile::SndFile::new(fname, sndfile::Read).unwrap();
	let info = sndf.get_sndinfo();
	assert_eq!(info.samplerate as u32, sRate);
	assert_eq!(info.channels as u32, 1);
	let mut x: ~[f32] = ~[0.0,.. 1024];
	for _ in range(0, (info.frames/2)/1024) {
		sndf.read_f32(x.as_mut_slice(), 1024);
		for &z in x.iter() {
			u.send(z)
		}
	}
	let (c, p) = channel();
	p.recv();
	c.send(());
}

pub fn wavSourceComplexF32(u: Sender<Complex<f32>>, fname: &str, sRate: u32) {
	let mut sndf = sndfile::SndFile::new(fname, sndfile::Read).unwrap();
	let info = sndf.get_sndinfo();
	assert_eq!(info.samplerate as u32, sRate);
	assert_eq!(info.channels as u32, 2);
	let mut x: ~[f32] = ~[0.0,.. 1024];
	for _ in range(0, (info.frames/2)/1024) {
		sndf.read_f32(x.as_mut_slice(), 1024);
		for z in x.chunks(2) {
			u.send(Complex{re: z[0], im: z[1]});
		}
	}
	let (c, p) = channel();
	p.recv();
	c.send(());
}
