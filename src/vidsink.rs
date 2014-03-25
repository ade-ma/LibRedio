extern crate sdl;
extern crate time;
extern crate native;
extern crate kpn;

use native::task::spawn;
use std::f32;
use std::comm;
use kpn::{Token, Packet, Flt, SourceConf};

pub fn drawVectorAsBarPlot (screen: &sdl::video::Surface, mut data: ~[f32]){
	let len: uint = data.len() as uint;
	let px: uint = (screen.get_width()) as uint;
	let decimateFactor = (px as f32 - (0.5f32*(data.len() as f32/1024f32)) ) / data.len() as f32;
	data = data.iter().enumerate().filter_map(|(x, &y)|
		if ((x as f32*decimateFactor) - (x as f32*decimateFactor).floor()) < decimateFactor { Some(y) } else { None }
	 ).to_owned_vec();
	// black screen background
	screen.fill_rect(Some(sdl::Rect {x: 0 as i16, y: 0 as i16, w: screen.get_width(), h: screen.get_height()}), sdl::video::RGB(0,0,0));
	// calculate bar width
	let width: f32 = screen.get_width() as f32 / (data.len() as f32);
	let height: f32 = screen.get_height() as f32;
	// find max value
	let &dmax: &f32 = data.iter().max().unwrap();
	let &dmin: &f32 = data.iter().min().unwrap();
	// calculate height scale value
	let scale: f32 = height / (dmax-dmin);
	assert!(width > 1.0);
	data.iter().enumerate().map(|(i, &x)| {
		let mut yf = height*1.0f32;
		let mut hf = scale*x;
		if x > 0f32 {yf -= x*scale;}
		if x < 0f32 {hf = -1f32*hf;}
		let r = sdl::Rect {
			x: ((screen.get_width() as f32)- width*(i as f32 + 1.0)) as i16,
			y: yf as i16,
			w: (width) as u16,
			h: hf as u16};
		screen.fill_rect(Some(r), sdl::video::RGB(0,127,0));
	}).len();
}

pub fn doWorkWithPEs (pDataC: comm::Port<~[f32]>) {
	let mut lastDraw: u64 = 0;
	sdl::init([sdl::InitVideo]);
	sdl::wm::set_caption("rust-sdl", "rust-sdl");
	let screen = match sdl::video::set_video_mode(1366, 640, 32, [sdl::video::HWSurface], [sdl::video::DoubleBuf]) {
		Ok(screen) => screen,
		Err(err) => fail!(format!("failed to set video mode: {:?}", err))
		};
		'main : loop {
			let x = pDataC.recv_opt();
			match x {
				Some(d) => drawVectorAsBarPlot(screen, d),
				_ => ()
			}
			'event : loop {
				let ev = sdl::event::poll_event();
				match ev {
					sdl::event::QuitEvent => break 'main,
					sdl::event::NoEvent => {break 'event},
					_ => {}
				}
			}
			if (time::precise_time_ns() - lastDraw) > ((1f32/30f32)*1e9) as u64 {
			lastDraw = time::precise_time_ns();
			screen.flip();
		}
	}
	sdl::quit();
}

pub fn spawnVectorVisualSink() -> comm::Chan<~[f32]> {
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::Chan::new();
	spawn(proc() {
		doWorkWithPEs(pData);
	});
	return cData;
}

pub fn vidSink(U: Port<Token>, S: SourceConf) {
	let c = spawnVectorVisualSink();
	let mut x: ~[f32] = ~[0.0f32, ..102];
	//let mut y = true;
	loop {
		match U.recv() {
			Packet(p) => {x = p.move_iter().filter_map(|x| match x { Flt(x) => Some(x), _ => None }).to_owned_vec()},
			Flt(d)  => {x.pop(); x.unshift(d)},
			_ => (),
		}
		c.send(x.clone());
		//y = true^y;
	}
}

