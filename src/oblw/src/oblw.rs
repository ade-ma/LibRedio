extern crate usb;
extern crate collections;
extern crate native;

use usb::libusb::LIBUSB_TRANSFER_TYPE_BULK;
use std::comm;
use std::vec;
use std::collections::bitv;

#[deriving(Clone)]
pub struct Run {
	pub v: uint,
	pub ct: uint
}

pub fn rld(input: Vec<Run>) -> Vec<uint> {
	let mut out: Vec<uint> = vec!();
	for i in input.iter() {
		for a in range(0u, i.ct.clone()) {
			out.push(i.v.clone());
		}
	};
	return out
}

pub fn B2b(bytes: &[u8]) -> bitv::Bitv {
	return bitv::from_bytes(bytes)
}

pub fn v2b(uints: Vec<uint>) -> bitv::Bitv {
	let y: Vec<bool> = uints.iter().map(|&x| x == 1u).collect();
	return bitv::from_fn(y.len(), |x|{*y.get(x)})
}

pub fn b2B(bits: bitv::Bitv) -> vec::Vec<u8> {
	return bits.to_bytes()
}

pub fn r2b(runs: Vec<Run>) -> bitv::Bitv {
	return v2b(rld(runs))
}

fn assemble_packet(y: &mut [u8], x: &[u8], norm: bool) {
	match norm {
		true => for i in range(0, x.len()) {y[i] = x[i];},
		false => for i in range(0, x.len()) {y[i] = x[i]^255u8;}
	}
}

pub fn spawn_bytestream(pDataI: std::comm::Receiver<Vec<u8>>, cDataO: std::comm::Sender<Vec<u8>>, defaultState: bool)  {
	let c = usb::Context::new();
	c.setDebug(2);
	let dev = match c.find_by_vid_pid(0x59e3, 0xf000) {
		Some(x) => x,
		None => fail!("no dev found"),
	};
	let handle = match dev.open() {
		Ok(x) => x,
		Err(code) => fail!("cannot open device {}", code),
	};
	handle.claim_interface(0);
	let ho = handle.clone();
	match defaultState {
		// low state - gnd - turn on inversion
		false => handle.ctrl_read(0x40|0x80, 0x08, 0x40, 0x0653, 0).unwrap(), // 0x693 - PINCTRL3; 0x40 - PORT_INVEN_bm
		// high value - 3v3 - turn off inversion
		true => handle.ctrl_read(0x40|0x80, 0x08, 0x00, 0x0653, 0).unwrap(),
	};

	native::task::spawn(proc() {
		ho.write_stream(0x02, LIBUSB_TRANSFER_TYPE_BULK, 64, 8, |buf| {
			let y = buf.unwrap();
			match pDataI.try_recv() {
				Ok(d) => {assemble_packet(y, d.as_slice(), defaultState); return true;},
				Err(code) => {
					//println!("write_stream err: {:?}", code);
					match defaultState {
						true => assemble_packet(y, [0xffu8, ..64], defaultState),
						false => assemble_packet(y, [0x00u8, ..64], defaultState)
					};
					return true;
				},
			}
		}); });

	let hi = handle.clone();
	hi.read_stream(0x81, LIBUSB_TRANSFER_TYPE_BULK, 64, 8, |res| {
		let y: Vec<u8> = res.unwrap().iter().map(|&x|x).collect();
		cDataO.send_opt(y).is_ok()
	});
}

pub fn bits_to_packets(p: comm::Receiver<bitv::Bitv>, c: comm::Sender<Vec<u8>>) {
	loop {
		let bytes = b2B(p.recv());
		for packet in bytes.slice_from(0).chunks(64) {
			let mut packet: Vec<u8> = packet.iter().map(|&x| x).collect();
			let len = packet.len();
			packet.grow(64-len, 0x00);
			c.send(packet);
		}
	}
}
