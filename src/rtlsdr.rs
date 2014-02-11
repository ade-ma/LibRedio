/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-SA 4.0. */
extern mod extra;
extern mod native;

use native::task::spawn;
use std::str;
use std::libc::{c_int, c_uint, c_void};
use std::vec;
use std::comm::{Chan,Port};
use std::ptr;

use extra::complex;

#[link(name= "rtlsdr")]

extern "C" {
	fn rtlsdr_open(dev: **c_void, devIndex: u32) -> u32;
	fn rtlsdr_get_device_count() -> u32;
	fn rtlsdr_get_device_name(devIndex: u32) -> *i8;
	fn rtlsdr_reset_buffer(dev: *c_void) -> c_int;
	fn rtlsdr_set_center_freq(dev: *c_void, freq: u32) -> c_int;
	fn rtlsdr_set_tuner_gain(dev: *c_void, gain: u32) -> c_int;
	fn rtlsdr_set_tuner_gain_mode(dev: *c_void, mode: u32) -> c_int;
	fn rtlsdr_read_sync(dev: *c_void, buf: *mut u8, len: u32, n_read: *c_int) -> c_int;
	fn rtlsdr_read_async(dev: *c_void, cb: extern "C" fn(*u8, u32, &Chan<~[u8]>), chan: &Chan<~[u8]>, buf_num: u32, buf_len: u32) -> c_int;
	fn rtlsdr_cancel_async(dev: *c_void) -> c_int;
	fn rtlsdr_set_sample_rate(dev: *c_void, sps: u32) -> c_int;
	fn rtlsdr_get_sample_rate(dev: *c_void) -> u32;
	fn rtlsdr_close(dev: *c_void) -> c_int;
}

pub fn close(dev: *c_void){
	unsafe {
		let success = rtlsdr_close(dev);
		assert_eq!(success, 0);
	}
}

pub fn setSampleRate(dev: *c_void, sps: u32) {
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

pub fn openDevice() -> *c_void {
	unsafe {
		let mut i: u32 = 0;
		let mut devStructPtr: *c_void = ptr::null();
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

pub fn getDeviceName(devIndex: u32) -> ~str {
	unsafe {
		let deviceString: *i8 = rtlsdr_get_device_name(devIndex);
		return str::raw::from_c_str(deviceString);
	}
}

pub fn clearBuffer(device: *c_void) {
	unsafe {
		let success = rtlsdr_reset_buffer(device);
		assert_eq!(success, 0);
	}
}

pub fn setFrequency(device: *c_void, freq: u32) {
	unsafe {
		let success = rtlsdr_set_center_freq(device, freq);
		assert_eq!(success, 0);
	}
}

pub fn setGain(device: *c_void, v: u32) {
	unsafe {
		let success = rtlsdr_set_tuner_gain_mode(device, 1);
		assert_eq!(success, 0);
		let success = rtlsdr_set_tuner_gain(device, v);
		assert_eq!(success, 0);
	}
}

pub fn setGainAuto(device: *c_void) {
	unsafe {
		let success = rtlsdr_set_tuner_gain_mode(device, 0);
		assert_eq!(success, 0);
	}
}

extern fn rtlsdr_callback(buf: *u8, len: u32, chan: &Chan<~[u8]>) {
	unsafe {
		let data = vec::raw::from_buf_raw(buf, len as uint);
		chan.send(data);
	}
}

pub fn readAsync(dev: *c_void, blockSize: u32) -> ~Port<~[u8]> {
	let (port, chan): (Port<~[u8]>, Chan<~[u8]>) = Chan::new();
	do spawn {
		unsafe{
			rtlsdr_read_async(dev, rtlsdr_callback, &chan, 32, blockSize*2);
		}
	}
	return ~port;
}

pub fn stopAsync(dev: *c_void) -> () {
	unsafe {
		let success = rtlsdr_cancel_async(dev);
		assert_eq!(success, 0);
	}
}

pub fn readSync(dev: *c_void, ct: c_uint) -> ~[u8] {
	unsafe {
		let n_read: c_int = 0;
		let mut buffer: ~[u8] = ~[0, ..512];
		let success = rtlsdr_read_sync(dev, buffer.as_mut_ptr(), ct, &n_read);
		assert_eq!(success, 0);
		assert_eq!(ct as i32, n_read);
		return buffer;
	}
}

fn i2f(i: u8) -> f32 {(i as f32)/127.0 - 1.0}
pub fn dataToSamples(data: ~[u8]) -> ~[complex::Complex32] {
	let samples = data.chunks(2).map(|i| complex::Cmplx{re:i2f(i[0]), im:i2f(i[1])}).collect();
	return samples;
}
