extern crate libc;
extern crate core;
extern crate debug;
use libc::{c_int, c_void, size_t};
use std::ptr::null;
use core::mem::transmute;
use std::vec;
use std::comm;
use std::task;
use std::num;


// opaque struct
struct PASimple;

struct PASampleSpec {
	format: c_int,
	rate: u32,
	channels: u8
}

#[link (name="pulse")]
#[link (name="pulse-simple")]

extern "C" {
	fn pa_simple_new(
		server: *const c_void,
		name: *const i8,
		dir: c_int,
		dev: *const c_void,
		stream_name: *const i8,
		ss: *const PASampleSpec,
		pa_channel_map: *const c_void,
		pa_buffer_attr: *const c_void,
		error: &mut c_int
	) -> &PASimple;
	fn pa_simple_read(s: &PASimple, data: *mut c_void, bytes: size_t, error: &mut c_int) -> c_int;
	fn pa_simple_write(s: &PASimple, data: *mut c_void, bytes: size_t, error: &mut c_int) -> c_int;
	fn pa_simple_flush(s: &PASimple, error: &mut c_int) -> c_int;
	fn pa_simple_get_latency(s: &PASimple, error: &mut c_int) -> u64;
}

pub fn pulse_source(cData: comm::Sender<Vec<f32>>, sRate: uint, bSize: uint) {
	
	let ss = PASampleSpec { format: 3, rate: sRate as u32, channels: 1 };
	// pa_stream_direction_t -> enum, record = 2, playback = 1
	unsafe {
		let mut error: c_int = 0;
		let s = pa_simple_new(null(), "rust-pa-simple-source".to_c_str().unwrap(), 2, null(), "pa-source".to_c_str().unwrap(), &ss, null(), null(), &mut error);
		assert_eq!(error, 0);
		'main : loop {
			let mut buffer: Vec<i16> = vec::Vec::from_elem(bSize, 0i16);
			pa_simple_read(s, transmute(buffer.as_mut_ptr()), (bSize*2) as u64, &mut error);
			assert_eq!(error, 0);
			let f32Buffer: Vec<f32> = buffer.iter().map(|&i| (i as f32)).collect();
			cData.send(f32Buffer);
		}
		}
}

pub fn pulse_sink(pData: Receiver<Vec<f32>>, sRate: uint) {
	let ss = PASampleSpec { format: 5, rate: sRate as u32, channels: 1 };
	let mut error: c_int = 0;
	unsafe {
		let s = pa_simple_new(null(), "rust-pa-simple-sink".to_c_str().unwrap(), 1, null(), "pa-sink".to_c_str().unwrap(), &ss, null(), null(), &mut error);
		println!("{:?}", pa_simple_get_latency(s, &mut error));
		'main : loop {
			let samps = pData.recv();
			if samps.len() == 0 { break 'main }
			let size: size_t = (samps.len() as u64)*4;
			pa_simple_write(s, transmute(samps.as_ptr()), size, &mut error);
		}
	}
}
