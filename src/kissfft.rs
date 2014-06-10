extern crate native;
extern crate num;
extern crate libc;

use std::vec;
use num::complex;
use std::ptr;
use libc::{c_int, size_t};
use std::comm;

#[link(name= "kissfft")]
extern "C" {
	fn kiss_fft_alloc(nfft: u32, inverse_fft: u32, mem: *u8, lenmem: *u64) -> *u8;
	fn kiss_fft(cfg: *u8, fin: *complex::Complex<f32>, mut fout: *mut complex::Complex<f32>);
	fn kiss_fft_cleanup();
}

pub fn fft(pin: comm::Receiver<Vec<complex::Complex<f32>>>, cout: comm::Sender<Vec<complex::Complex<f32>>>, blockSize: u32, inv: u32) {
	let kiss_fft_cfg = unsafe {kiss_fft_alloc(blockSize, inv, ptr::null(), ptr::null())};
	loop {
		let mut fout: Vec<complex::Complex<f32>> = vec::Vec::with_capacity(blockSize as uint);
		unsafe {fout.set_len(blockSize as uint)}
		let din = pin.recv();
		assert!(din.len() == blockSize as uint);
		unsafe {
			kiss_fft(kiss_fft_cfg, din.as_ptr(), fout.as_mut_ptr());
			cout.send(fout);
		}
	}
	unsafe { kiss_fft_cleanup(); }
}
