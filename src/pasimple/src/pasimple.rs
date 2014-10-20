extern crate libc;
extern crate core;

use libc::{c_int, c_void, size_t};
use std::ptr::null;
use core::mem::transmute;
use std::vec;
use std::comm;

// opaque struct
#[repr(C)]
struct paSimple;

#[repr(C)]
struct paSampleSpec {
	format: c_int,
	rate: u32,
	channels: u8
}

#[link (name="pulse")]
#[link (name="pulse-simple")]

#[repr(C)]
extern "C" {
	fn pa_simple_new(
		server: *const c_void,
		name: *const i8,
		dir: c_int,
		dev: *const c_void,
		stream_name: *const i8,
		ss: *const paSampleSpec,
		pa_channel_map: *const c_void,
		pa_buffer_attr: *const c_void,
		error: &mut c_int
	) -> &paSimple;
	fn pa_simple_read(s: &paSimple, data: *mut c_void, bytes: size_t, error: &mut c_int) -> c_int;
	fn pa_simple_write(s: &paSimple, data: *mut c_void, bytes: size_t, error: &mut c_int) -> c_int;
	fn pa_simple_flush(s: &paSimple, error: &mut c_int) -> c_int;
	fn pa_simple_get_latency(s: &paSimple, error: &mut c_int) -> u64;
}

pub fn pulse_source(c_data: comm::Sender<Vec<f32>>, s_rate: uint, b_size: uint) {
	
	let ss = paSampleSpec { format: 3, rate: s_rate as u32, channels: 1 };
	// pa_stream_direction_t -> enum, record = 2, playback = 1
	unsafe {
		let mut error: c_int = 0;
		let s = pa_simple_new(null(), "rust-pa-simple-source".to_c_str().unwrap(), 2, null(), "pa-source".to_c_str().unwrap(), &ss, null(), null(), &mut error);
		assert_eq!(error, 0);
		'main : loop {
			let mut buffer: Vec<i16> = vec::Vec::from_elem(b_size, 0i16);
			pa_simple_read(s, transmute(buffer.as_mut_ptr()), (b_size*2) as u64, &mut error);
			assert_eq!(error, 0);
			let f32_buffer: Vec<f32> = buffer.iter().map(|&i| (i as f32)).collect();
			c_data.send(f32_buffer);
		}
		}
}

pub fn pulse_sink(p_data: Receiver<Vec<f32>>, s_rate: uint) {
	let ss = paSampleSpec { format: 5, rate: s_rate as u32, channels: 1 };
	let mut error: c_int = 0;
	unsafe {
		let s = pa_simple_new(null(), "rust-pa-simple-sink".to_c_str().unwrap(), 1, null(), "pa-sink".to_c_str().unwrap(), &ss, null(), null(), &mut error);
		println!("{}", pa_simple_get_latency(s, &mut error));
		'main : loop {
			let samps = p_data.recv();
			if samps.len() == 0 { break 'main }
			let size: size_t = (samps.len() as u64)*4;
			pa_simple_write(s, transmute(samps.as_ptr()), size, &mut error);
		}
	}
}
