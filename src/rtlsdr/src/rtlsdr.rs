/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

#![allow(unstable)]
extern crate num;
extern crate libc;

use num::complex;
use libc::{c_int, c_uint, c_void};
use std::sync::mpsc::{Receiver, Sender, channel, Handle, Select};
use std::vec;
use std::str;
use std::ffi;
use std::string;
use std::thread::Thread;
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::intrinsics;

#[link(name= "rtlsdr")]

#[repr(C)]
struct RTLSDR_Internal;

#[derive(Clone)]
pub struct RTLSDR_Dev {
    bx: Arc<UnsafeCell<RTLSDR_Internal>>
}

unsafe impl Sync for UnsafeCell<RTLSDR_Internal> {}

extern "C" {
	fn rtlsdr_open(dev: &RTLSDR_Internal, dev_index: u32) -> u32;
	fn rtlsdr_get_device_count() -> u32;
	fn rtlsdr_get_device_name(dev_index: u32) -> *const i8;
	fn rtlsdr_reset_buffer(dev: &RTLSDR_Internal) -> c_int;
	fn rtlsdr_set_center_freq(dev: &RTLSDR_Internal, freq: u32) -> c_int;
	fn rtlsdr_set_tuner_gain(dev: &RTLSDR_Internal, gain: u32) -> c_int;
	fn rtlsdr_set_tuner_gain_mode(dev: &RTLSDR_Internal, mode: u32) -> c_int;
	fn rtlsdr_read_sync(dev: &RTLSDR_Internal, buf: *mut u8, len: u32, n_read: *mut c_int) -> c_int;
	fn rtlsdr_read_async(dev: &RTLSDR_Internal, cb: extern "C" fn(*const u8, u32, &Sender<Vec<u8>>), chan: &Sender<Vec<u8>>, buf_num: u32, buf_len: u32) -> c_int;
	fn rtlsdr_cancel_async(dev: &RTLSDR_Internal) -> c_int;
	fn rtlsdr_set_sample_rate(dev: &RTLSDR_Internal, sps: u32) -> c_int;
	fn rtlsdr_get_sample_rate(dev: &RTLSDR_Internal) -> u32;
	fn rtlsdr_close(dev: &RTLSDR_Internal) -> c_int;
}


impl Drop for RTLSDR_Dev {
    fn drop (&mut self) {
        self.close();
    }
}

impl RTLSDR_Dev {
	pub fn close(&self) {
		unsafe {
			let success = rtlsdr_close(&(*self.bx.get()));
			assert_eq!(success, 0);
		}
	}
	pub fn set_sample_rate(&self, sps: u32) {
		unsafe {
			let success = rtlsdr_set_sample_rate(&(*self.bx.get()), sps);
			assert_eq!(success, 0);
			println!("actual sample rate: {}", rtlsdr_get_sample_rate(&(*self.bx.get())));
		}
	}
	pub fn open() -> RTLSDR_Dev {
		unsafe {
			let mut i: u32 = 0;
            let internal: RTLSDR_Internal = intrinsics::init();
			'tryDevices: loop {
				let success = rtlsdr_open(&internal, i);
				if success == 0 {
					break 'tryDevices
				}
				if i > get_device_count() {
					panic!("no available devices");
				}
				i += 1;
			}
       return RTLSDR_Dev {bx: Arc::new(UnsafeCell::new(internal))};
	   }
	}
	pub fn clear_buffer(&self) {
		unsafe {
			let success = rtlsdr_reset_buffer(&(*self.bx.get()));
			assert_eq!(success, 0);
		}
	}
	pub fn set_frequency(&self, freq: u32) {
		unsafe {
			let success = rtlsdr_set_center_freq(&(*self.bx.get()), freq);
			assert_eq!(success, 0);
		}
	}
	pub fn set_gain(&self, v: u32) {
		unsafe {
			let success = rtlsdr_set_tuner_gain_mode(&(*self.bx.get()), 1);
			assert_eq!(success, 0);
			let success = rtlsdr_set_tuner_gain(&(*self.bx.get()), v);
			assert_eq!(success, 0);
		}
	}
	pub fn set_gain_auto(&self) {
		unsafe {
			let success = rtlsdr_set_tuner_gain_mode(&(*self.bx.get()), 0);
			assert_eq!(success, 0);
		}
	}
	pub fn read_async(&self, block_size: u32) -> Receiver<Vec<u8>> {
		let (chan, port) = channel();
        let bx = self.bx.clone();
		Thread::spawn(move || {
			unsafe{
				rtlsdr_read_async(&(*bx.get()), rtlsdr_callback, &chan, 32, block_size*2);
			}
		});
		return port;
	}
	pub fn stop_async(&self) {
		unsafe {
			let success = rtlsdr_cancel_async(&(*self.bx.get()));
			assert_eq!(success, 0);
		}
	}
	pub fn read_sync(&self, ct: c_uint) -> Vec<u8> {
		unsafe {
			let mut n_read: c_int = 0;
			let mut buffer = vec::Vec::with_capacity(512);
			let success = rtlsdr_read_sync(&(*self.bx.get()), buffer.as_mut_ptr(), ct, &mut n_read);
			assert_eq!(success, 0);
			assert_eq!(ct as i32, n_read);
			return buffer;
		}
	}
}

extern fn rtlsdr_callback(buf: *const u8, len: u32, chan: &Sender<Vec<u8>>) {
	unsafe {
		let data = vec::Vec::from_raw_buf(buf, len as usize);
		chan.send(data);
	}
}

pub fn get_device_count() -> u32 {
	unsafe {
		let x: u32 = rtlsdr_get_device_count();
		return x;
	}
}

pub fn get_device_name(dev_index: u32) -> string::String {
	unsafe {
		let device_string: *const i8 = rtlsdr_get_device_name(dev_index);
		return string::String::from_str(str::from_utf8(ffi::c_str_to_bytes(&device_string)).unwrap());
	}
}
fn i2f(i: u8) -> f32 {i as f32/127.0 - 1.0}
pub fn data_to_samples(data: Vec<u8>) -> Vec<complex::Complex<f32>> {
	data.slice_from(0).chunks(2).map(|i| complex::Complex{re:i2f(i[0]), im:i2f(i[1])}).collect()
}
