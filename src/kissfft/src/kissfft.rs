#![feature(libc)]
extern crate num;
extern crate libc;

use std::vec;
use num::complex;
use std::ptr;

use std::sync::mpsc::{Receiver, Sender, channel, Handle, Select};

#[link(name= "kissfft")]
extern "C" {
	fn kiss_fft_alloc(nfft: u32, inverse_fft: u32, mem: *mut u8, lenmem: *mut u64) -> *const u8;
	fn kiss_fft(cfg: *const u8, fin: *const complex::Complex<f32>, mut fout: *mut complex::Complex<f32>);
	fn kiss_fft_cleanup();
}

pub fn fft(pin: Receiver<Vec<complex::Complex<f32>>>, cout: Sender<Vec<complex::Complex<f32>>>, block_size: u32, inv: u32) {
	let kiss_fft_cfg = unsafe {kiss_fft_alloc(block_size, inv, ptr::null_mut(), ptr::null_mut())};
	loop {
		let mut fout: Vec<complex::Complex<f32>> = vec::Vec::with_capacity(block_size as usize);
		unsafe {fout.set_len(block_size as usize)}
		let din = pin.recv().unwrap();
		assert!(din.len() == block_size as usize);
		unsafe {
			kiss_fft(kiss_fft_cfg, din.as_ptr(), fout.as_mut_ptr());
			cout.send(fout).unwrap();
		}
	}
	unsafe { kiss_fft_cleanup(); }
}
