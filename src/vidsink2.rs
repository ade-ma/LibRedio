extern crate sdl2;
extern crate dsputils;
extern crate native;
extern crate kpn;


use kpn::{Token, Packet, Flt};
use std::comm;
use native::task::spawn;

pub fn drawVectorAsBarPlot (renderer: &sdl2::render::Renderer, mut data: Vec<f32>) {
	// downsample to 800px if needbe
	let (sw, sh) = renderer.get_output_size().unwrap();
	let len: uint = data.len() as uint;
	let px: uint = sw as uint;
	let decimateFactor = (px as f32 - (0.5f32*(data.len() as f32/px as f32)) ) / data.len() as f32;
	data = data.iter().enumerate().filter_map(|(x, &y)|
		if ((x as f32*decimateFactor) - (x as f32*decimateFactor).floor()) < decimateFactor { Some(y) } else { None }
	 ).collect();
	// black screen background
	renderer.set_draw_color(sdl2::pixels::RGB(0, 0, 0));
	renderer.clear();
	// calculate bar width
	let width: f32 = sw as f32 / (data.len() as f32);
	let height: f32 = sh as f32;
	// find max value
	let dmax = dsputils::max(data.slice_from(0));
	let dmin = dsputils::min(data.slice_from(0));
	// calculate height scale value
	let scale: f32 = height / (2f32*(dmax-dmin));
	let width = if width > 1.0 { width } else { 1.0 };
	renderer.set_draw_color(sdl2::pixels::RGB(0, 127, 7));
	let rs: ~[sdl2::rect::Rect] = range(0, data.len()).map(|i| {
		let &x = data.get(i);
		let mut yf = height*0.5f32;
		let mut hf = scale*x;
		if x > 0f32 {yf -= x*scale;}
		if x < 0f32 {hf = -1f32*hf;}
		sdl2::rect::Rect {
			x: ((sw as f32) - width*(i as f32 + 1.0)) as i32,
			y: yf as i32,
			w: width as i32,
			h: hf as i32}
	}).collect();
	renderer.fill_rects(rs.slice_from(0));
}

pub fn doWorkWithPEs (pDataC: comm::Receiver<Vec<f32>>) {
	//sdl2::init([sdl2::InitVideo]);
	let window =  match sdl2::video::Window::new("sdl2 vidsink", sdl2::video::PosCentered, sdl2::video::PosCentered, 1300, 600, sdl2::video::Shown) {
		Ok(window) => window,
		Err(err) => fail!("")
	};
	let renderer =  match sdl2::render::Renderer::from_window(window, sdl2::render::DriverAuto, sdl2::render::Software){
		Ok(renderer) => renderer,
		Err(err) => fail!("")
	};
	renderer.set_logical_size(1300, 600);
	'main : loop {
		match sdl2::event::poll_event() {
			sdl2::event::QuitEvent(_) => break 'main,
			_ => {}
		}
		match pDataC.try_recv() {
			Ok(d) => {
				drawVectorAsBarPlot(renderer, d);
			}
			_ => ()
		}
		renderer.present()
	}
	sdl2::quit();
}

pub fn spawnVectorVisualSink() -> (comm::Sender<Vec<f32>>) {
	let (cData, pData) = comm::channel();
	spawn(proc(){ doWorkWithPEs(pData)});
	return cData;
}

pub fn vidSink(u: Receiver<Token>) {
	let c = spawnVectorVisualSink();
	let mut x: Vec<f32> = std::vec::Vec::from_slice([0.0f32,.. 1300]);
	loop {
		match u.recv() {
			Packet(p) => {x = p.move_iter().filter_map(|x| match x { Flt(x) => Some(-1.0*x), _ => None }).collect()},
			Flt(d)  => {x.pop(); x.unshift(d)},
			_ => (),
		}
		c.send(x.clone());
	}
}

