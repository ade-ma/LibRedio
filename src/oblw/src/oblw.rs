extern crate usb;
extern crate collections;

use usb::libusb::LIBUSB_TRANSFER_TYPE_BULK;
use std::vec;
use std::collections::bitv::Bitv;
use std::thread::Thread;
use std::sync::mpsc::{Receiver, Sender, channel, Handle, Select};


#[deriving(Clone)]
pub struct Run {
	pub v: usize,
	pub ct: usize
}

pub fn rld(input: Vec<Run>) -> Vec<usize> {
	let mut out: Vec<usize> = vec!();
	for i in input.iter() {
		for a in 0..i.ct.clone() {
			out.push(i.v.clone());
		}
	};
	return out
}

pub fn B2b(bytes: &[u8]) -> Bitv {
	return Bitv::from_bytes(bytes)
}

pub fn v2b(usizes: Vec<usize>) -> Bitv {
	let y: Vec<bool> = usizes.iter().map(|&x| x == 1).collect();
	return Bitv::from_fn(y.len(), |x|{y[x]})
}

pub fn b2B(bits: Bitv) -> vec::Vec<u8> {
	return bits.to_bytes()
}

pub fn r2b(runs: Vec<Run>) -> Bitv {
	return v2b(rld(runs))
}

fn assemble_packet(y: &mut [u8], x: &[u8], norm: bool) {
	match norm {
		true => for i in range(0, x.len()) {y[i] = x[i];},
		false => for i in range(0, x.len()) {y[i] = x[i]^255u8;}
	}
}

pub fn spawn_bytestream(pDataI: Receiver<Vec<u8>>, cDataO: Sender<Vec<u8>>, defaultState: bool)  {
	let c = usb::Context::new();
	let dev = match c.find_by_vid_pid(0x59e3, 0xf000) {
		Some(x) => x,
		None => panic!("no dev found"),
	};
	let handle = match dev.open() {
		Ok(x) => x,
		Err(code) => panic!("cannot open device {}", code),
	};
	handle.claim_interface(0);
	let ho = handle.clone();
	match defaultState {
		// low state - gnd - turn on inversion
		false => handle.ctrl_read(0x40|0x80, 0x08, 0x40, 0x0653, 0, 10).unwrap(), // 0x693 - PINCTRL3; 0x40 - PORT_INVEN_bm
		// high value - 3v3 - turn off inversion
		true => handle.ctrl_read(0x40|0x80, 0x08, 0x00, 0x0653, 0, 10).unwrap(),
	};
	Thread::spawn(move || {
		ho.write_stream(0x02, LIBUSB_TRANSFER_TYPE_BULK, 64, 8, &mut |buf| {
			let y = buf.unwrap();
			match pDataI.recv() {
				Ok(d) => {assemble_packet(y, d.as_slice(), defaultState); return true;},
				Err(code) => {
					//println!("write_stream err: {:?}", code);
					match defaultState {
						true => assemble_packet(y, &[0xffu8;64], defaultState),
						false => assemble_packet(y, &[0x00u8;64], defaultState)
					};
					return true;
				},
			}
		}); });

	let hi = handle.clone();
	hi.read_stream(0x81, LIBUSB_TRANSFER_TYPE_BULK, 64, 8, &mut |res| {
		let y: Vec<u8> = res.unwrap().iter().map(|&x|x).collect();
		cDataO.send(y).unwrap();
        return true;
	});
}

pub fn bits_to_packets(p: Receiver<Bitv>, c: Sender<Vec<u8>>) {
	loop {
		let bytes = b2B(p.recv().unwrap());
		for packet in bytes.slice_from(0).chunks(64) {
			let mut packet: Vec<u8> = packet.iter().map(|&x|x).collect();
			range(0,64-packet.len()).map(|_|packet.push(0u8)).last();
			c.send(packet).unwrap();
		}
	}
}
