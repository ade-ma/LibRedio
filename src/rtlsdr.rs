/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

extern crate num;
extern crate native;
extern crate libc;
use num::complex;

use native::task::spawn;
use libc::{c_int, c_uint, c_void};
use std::slice;
use std::comm::{Sender, Receiver, channel};
use std::ptr;
use std::num;
use std::vec;
use std::string;
use std::str;

#[link(name= "rtlsdr")]

extern "C" {
	fn rtlsdr_open(dev: & *mut c_void, devIndex: u32) -> u32;
	fn rtlsdr_get_device_count() -> u32;
	fn rtlsdr_get_device_name(devIndex: u32) -> *const i8;
	fn rtlsdr_reset_buffer(dev: *mut c_void) -> c_int;
	fn rtlsdr_set_center_freq(dev: *mut c_void, freq: u32) -> c_int;
	fn rtlsdr_set_tuner_gain(dev: *mut c_void, gain: u32) -> c_int;
	fn rtlsdr_set_tuner_gain_mode(dev: *mut c_void, mode: u32) -> c_int;
	fn rtlsdr_read_sync(dev: *mut c_void, buf: *mut u8, len: u32, n_read: *mut c_int) -> c_int;
	fn rtlsdr_read_async(dev: *mut c_void, cb: extern "C" fn(*const u8, u32, &Sender<Vec<u8>>), chan: &Sender<Vec<u8>>, buf_num: u32, buf_len: u32) -> c_int;
	fn rtlsdr_cancel_async(dev: *mut c_void) -> c_int;
	fn rtlsdr_set_sample_rate(dev: *mut c_void, sps: u32) -> c_int;
	fn rtlsdr_get_sample_rate(dev: *mut c_void) -> u32;
	fn rtlsdr_close(dev: *mut c_void) -> c_int;
}

pub fn close(dev: *mut c_void){
	unsafe {
		let success = rtlsdr_close(dev);
		assert_eq!(success, 0);
	}
}

pub fn setSampleRate(dev: *mut c_void, sps: u32) {
	unsafe {
		let success = rtlsdr_set_sample_rate(dev, sps);
		assert_eq!(success, 0);
		println!("actual sample rate: {}", rtlsdr_get_sample_rate(dev));
	}
}

pub fn getDeviceCount() -> u32 {
	unsafe {
		let x: u32 = rtlsdr_get_device_count();
		return x;
	}
}

pub fn openDevice() -> *mut c_void {
	unsafe {
		let mut i: u32 = 0;
		let devStructPtr: *mut c_void = ptr::mut_null();
		'tryDevices: loop {
			let success = rtlsdr_open(&devStructPtr, i);
			if success == 0 {
				break 'tryDevices
			}
			if i > getDeviceCount() {
				fail!("no available devices");
			}
			i += 1;
		}
	return devStructPtr;
	}
}

pub fn getDeviceName(devIndex: u32) -> string::String {
	unsafe {
		let deviceString: *const i8 = rtlsdr_get_device_name(devIndex);
		return str::raw::from_c_str(deviceString);
	}
}

pub fn clearBuffer(device: *mut c_void) {
	unsafe {
		let success = rtlsdr_reset_buffer(device);
		assert_eq!(success, 0);
	}
}

pub fn setFrequency(device: *mut c_void, freq: u32) {
	unsafe {
		let success = rtlsdr_set_center_freq(device, freq);
		assert_eq!(success, 0);
	}
}

pub fn setGain(device: *mut c_void, v: u32) {
	unsafe {
		let success = rtlsdr_set_tuner_gain_mode(device, 1);
		assert_eq!(success, 0);
		let success = rtlsdr_set_tuner_gain(device, v);
		assert_eq!(success, 0);
	}
}

pub fn setGainAuto(device: *mut c_void) {
	unsafe {
		let success = rtlsdr_set_tuner_gain_mode(device, 0);
		assert_eq!(success, 0);
	}
}

extern fn rtlsdr_callback(buf: *const u8, len: u32, chan: &Sender<Vec<u8>>) {
	unsafe {
		let data = vec::raw::from_buf(buf, len as uint);
		chan.send(data);
	}
}

pub fn readAsync(dev: *mut c_void, blockSize: u32) -> Receiver<Vec<u8>> {
	let (chan, port) = channel();
	spawn(proc() {
		unsafe{
			rtlsdr_read_async(dev, rtlsdr_callback, &chan, 32, blockSize*2);
		}
	});
	return port;
}

pub fn stopAsync(dev: *mut c_void) -> () {
	unsafe {
		let success = rtlsdr_cancel_async(dev);
		assert_eq!(success, 0);
	}
}

pub fn readSync(dev: *mut c_void, ct: c_uint) -> Vec<u8> {
	unsafe {
		let mut n_read: c_int = 0;
		let mut buffer = vec::Vec::with_capacity(512);
		let success = rtlsdr_read_sync(dev, buffer.as_mut_ptr(), ct, &mut n_read);
		assert_eq!(success, 0);
		assert_eq!(ct as i32, n_read);
		return buffer;
	}
}

fn i2f(i: u8) -> f32 {i as f32/127.0 - 1.0}
pub fn dataToSamples(data: Vec<u8>) -> Vec<complex::Complex<f32>> {
	data.slice_from(0).chunks(2).map(|i| complex::Complex{re:i2f(i[0]), im:i2f(i[1])}).collect()
}
