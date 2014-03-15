extern crate sndfile;
extern crate kpn;

use kpn::{Token, Packet, SourceConf, Dbl};
use std::num;
use std::comm::Chan;
use std::io;

pub fn wavSource(u: Chan<Token>, s: SourceConf) {
	let mut timer = io::Timer::new().unwrap();
	let mut sndf = sndfile::SndFile::new("./in.wav", sndfile::Read).unwrap();
	let info = sndf.get_sndinfo();
	assert_eq!(info.samplerate as u32, s.Rate as u32);
	assert_eq!(info.channels as u32, 2);
	let mut x: ~[f64] = ~[0.0,.. 1024];
	for _ in range(0, (info.frames/2)/1024) {
		sndf.read_f64(x.as_mut_slice(), 1024);
		let ds = x.chunks(2).map(|z| Dbl(num::hypot(z[0], z[1]))).to_owned_vec();
		u.send(Packet(ds));
		timer.sleep(100);
	}
	let (p, c) = Chan::new();
	p.recv();
	c.send(1u);
}
