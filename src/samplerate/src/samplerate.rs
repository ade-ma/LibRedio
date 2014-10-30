extern crate libc;

use std::vec;
use std::str;
use libc::{c_int, c_void, c_double, c_char, c_float, c_long, c_uint};
use std::comm;

// bindings for SRC - http://www.mega-nerd.com/SRC/index.html

#[allow(non_camel_case_types)]

pub struct SRC_DATA {
	pub data_in: *const f32,
	pub data_out: *mut f32,
	pub input_frames: u64,
	pub output_frames: u64,
	pub input_frames_used: u64,
	pub output_frames_gen: u64,
	pub end_of_input: u32,
	pub src_ratio: f64
}

pub static SRC_SINC_BEST_QUALITY: c_uint = 0;
pub static SRC_SINC_MEDIUM_QUALITY: c_uint = 1;
pub static SRC_SINC_FASTEST: c_uint = 2;
pub static SRC_ZERO_ORDER_HOLD: c_uint = 3;
pub static SRC_LINEAR: c_uint = 4;

#[link(name="samplerate")]
extern "C" {
	pub fn src_new(converter_type: c_int, channels: c_int, error: &mut c_int) -> &c_void;
	pub fn src_delete(state: &c_void ) -> &c_void;
	pub fn src_process(state: &c_void, data: &SRC_DATA) -> c_int;
	pub fn src_get_name(converter_type: c_int) -> &c_char;
	pub fn src_get_description(converter_type: c_int) -> &c_char;
	pub fn src_get_version() -> &c_char;
	pub fn src_set_ratio(state: &mut c_void, new_ratio: c_double) -> c_int;
	pub fn src_is_valid_ratio(ratio: c_double) -> c_int;
	pub fn src_strerror(error: c_int) -> &i8;
	/*pub fn src_callback_new(func: src_callback_t,
		converter_type: c_int,
		channels: c_int,
		error: *mut c_int,
		cb_data: *mut c_void) -> *mut c_void;
	pub fn src_callback_read(state: *mut c_void,
		 src_ratio: c_double,
		 frames: c_long,
		 data: *mut c_float) -> c_long;
	pub fn src_simple(data: *mut SRC_DATA, converter_type: c_int,
		channels: c_int) -> c_int;
	pub fn src_error(state: *mut c_void) -> c_int;
	pub fn src_reset(state: *mut c_void) -> c_int;*/
}


pub fn resample(din: comm::Receiver<Vec<f32>>, dout: comm::Sender<Vec<f32>>, ratio: f64) {
	let mut error: c_int = 0;
	let ctx = unsafe { src_new(1, 1, &mut error)};
	loop {
		let vin = din.recv();
		let lout: uint = ((ratio * vin.len() as f64) + 1f64) as uint;
		let mut vout = vec::Vec::with_capacity(lout);
		let src_data = SRC_DATA {
			data_in : vin.as_ptr(),
			data_out: vout.as_mut_ptr(),
			input_frames: vin.len() as u64,
			output_frames: vout.capacity() as u64,
			input_frames_used: 0,
			output_frames_gen: 0,
			end_of_input: 0,
			src_ratio: ratio
		};
		let error = unsafe { src_process(ctx, &src_data)};
		if error != 0 {
			let error_msg = unsafe { str::raw::from_c_str(src_strerror(error))};
			panic!("{}", error_msg);
		}
		unsafe{vout.set_len(src_data.output_frames_gen as uint)};
		dout.send(vout);
	}
}

/*fn main () {
	let (tx0, rx0) = channel();
	let (tx1, rx1) = channel();
	spawn(proc() { resample(rx0, tx1, 2.0) });
	let v = range(0,1000).map(|x| (x as f32 / 1000.0).sin()).collect();
	tx0.send(v);
	println!("{}", rx1.recv().len());
}*/
