extern crate libc;
extern crate core;

use libc::{c_int, c_void, size_t};
use std::ptr::null;
use core::mem::transmute;
use std::sync::mpsc::{Receiver,Sender,channel};
use std::ffi::CString;

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
		name: &[u8],
		dir: c_int,
		dev: *const c_void,
		stream_name: &[u8],
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

pub fn pulse_source(c_data: Sender<Vec<f32>>, s_rate: usize, b_size: usize) {
	
	let ss = paSampleSpec { format: 3, rate: s_rate as u32, channels: 1 };
	// pa_stream_direction_t -> enum, record = 2, playback = 1
	unsafe {
		let mut error: c_int = 0;
		let s = pa_simple_new(null(), CString::from_slice("rust-pa-simple-source".as_bytes()).as_bytes_with_nul(), 2, null(), CString::from_slice("pa-source".as_bytes()).as_bytes_with_nul(), &ss, null(), null(), &mut error);
		assert_eq!(error, 0);
		'main : loop {
			let mut buffer: Vec<i16> = [0..b_size].iter().map(|_|0i16).collect();
			pa_simple_read(s, transmute(buffer.as_mut_ptr()), (b_size*2) as u64, &mut error);
			assert_eq!(error, 0);
			let f32_buffer: Vec<f32> = buffer.iter().map(|&i| (i as f32)).collect();
			c_data.send(f32_buffer).unwrap();
		}
		}
}

pub fn pulse_sink(p_data: Receiver<Vec<f32>>, s_rate: usize) {
	let ss = paSampleSpec { format: 5, rate: s_rate as u32, channels: 1 };
	let mut error: c_int = 0;
	unsafe {
		let s = pa_simple_new(null(), CString::from_slice("rust-pa-simple-sink".as_bytes()).as_bytes_with_nul(), 1, null(), CString::from_slice("pa-sink".as_bytes()).as_bytes_with_nul(), &ss, null(), null(), &mut error);
		println!("{}", pa_simple_get_latency(s, &mut error));
		'main : loop {
			let samps = p_data.recv().unwrap();
			if samps.len() == 0 { break 'main }
			let size: size_t = (samps.len() as u64)*4;
			pa_simple_write(s, transmute(samps.as_ptr()), size, &mut error);
		}
	}
}
