extern crate sndfile;
extern crate kpn;

use std::num;
use std::comm::Sender;
use std::io;

pub fn wavSource(u: Sender<Vec<f32>>, sRate: u32) {
	let mut timer = io::Timer::new().unwrap();
	let mut sndf = sndfile::SndFile::new("./in.wav", sndfile::Read).unwrap();
	let info = sndf.get_sndinfo();
	assert_eq!(info.samplerate as u32, sRate);
	assert_eq!(info.channels as u32, 2);
	let mut x: ~[f32] = ~[0.0,.. 1024];
	for _ in range(0, (info.frames/2)/1024) {
		sndf.read_f32(x.as_mut_slice(), 1024);
		u.send(x.chunks(2).map(|z| Flt(z[0].hypot(z[1]))).collect());
		timer.sleep(100);
	}
	let (c, p) = channel();
	p.recv();
	c.send(());
}
