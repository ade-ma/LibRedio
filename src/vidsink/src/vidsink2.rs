extern crate sdl2;
extern crate dsputils;

use std::sync::mpsc::{Receiver, Sender, channel, Handle, Select};
use std::thread::Thread;
use std::intrinsics::floorf32;

pub fn draw_vector_as_barplot (renderer: &sdl2::render::Renderer, mut data: Vec<f32>) {
	// downsample to 800px if needbe
	let (sw, sh) = renderer.get_output_size().unwrap();
	let len = data.len();
	let px: usize = sw as usize;
	let decimate_factor = (px as f32 - (0.5f32*(data.len() as f32/px as f32)) ) / data.len() as f32;
	data = data.iter().enumerate().filter_map(|(x, &y)|
        unsafe {
		    if ((x as f32*decimate_factor) - floorf32(x as f32*decimate_factor)) < decimate_factor { Some(y) } else { None }
        }
	 ).collect();
	// black screen background
	renderer.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
	renderer.clear();
	// calculate bar width
	let width: f32 = sw as f32 / (data.len() as f32);
	let height: f32 = sh as f32;
	// find max value
	let dmax = dsputils::max(data.slice_from(0));
	let dmin = dsputils::min(data.slice_from(0));
	// calculate height scale value
	let scale: f32 = height / ((dmax-dmin));
	let width = if width > 1.0 { width } else { 1.0 };
	renderer.set_draw_color(sdl2::pixels::Color::RGB(0, 127, 7));
	let rs: Vec<sdl2::rect::Rect> = range(0, data.len()).map(|i| {
		let x = data[i];
		let mut yf = if dmin > 0.0 { height } else {height*0.5f32};
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

pub fn vidsink_vecs(pDataC: Receiver<Vec<f32>>) {
	let window =  match sdl2::video::Window::new("sdl2 vidsink", sdl2::video::WindowPos::PosCentered, sdl2::video::WindowPos::PosCentered, 1300, 600, sdl2::video::SHOWN) {
		Ok(window) => window,
		Err(err) => panic!("")
	};
	let renderer =  match sdl2::render::Renderer::from_window(window, sdl2::render::RenderDriverIndex::Auto, sdl2::render::SOFTWARE){
		Ok(renderer) => renderer,
		Err(err) => panic!("")
	};
	renderer.set_logical_size(1300, 600);
	'main : loop {
		match sdl2::event::poll_event() {
			sdl2::event::Event::Quit(_) => break 'main,
			_ => {}
		}
		match pDataC.recv() {
			Ok(d) => {
				draw_vector_as_barplot(&renderer, d);
			}
			_ => ()
		}
		renderer.present()
	}
	sdl2::quit();
}
pub fn vidsink(pDataC: Receiver<f32>, size: usize) {
	let window =  match sdl2::video::Window::new("sdl2 vidsink", sdl2::video::WindowPos::PosCentered, sdl2::video::WindowPos::PosCentered, 1300, 600, sdl2::video::SHOWN) {
		Ok(window) => window,
		Err(err) => panic!("")
	};
	let renderer =  match sdl2::render::Renderer::from_window(window, sdl2::render::RenderDriverIndex::Auto, sdl2::render::SOFTWARE){
		Ok(renderer) => renderer,
		Err(err) => panic!("")
	};
	let mut data: Vec<f32> = range(0, size).map(|_|0f32).collect();
	renderer.set_logical_size(1300, 600);
	'main : loop {
		match sdl2::event::poll_event() {
			sdl2::event::Event::Quit(_) => break 'main,
			_ => {}
		}
		data.pop();
		data.insert(0, pDataC.recv().unwrap());
		draw_vector_as_barplot(&renderer, data.clone());
		renderer.present()
	}
	sdl2::quit();
}
pub fn many_vidsink(u: Vec<Receiver<Vec<f32>>>) {
	sdl2::init(sdl2::INIT_VIDEO);
	let l = u.len() as isize;
	for x in u.into_iter() {
		let window =  match sdl2::video::Window::new("sdl2 vidsink", sdl2::video::WindowPos::PosCentered, sdl2::video::WindowPos::PosCentered, 1300/l, 600, sdl2::video::SHOWN) {
			Ok(window) => window,
			Err(err) => panic!("")
		};
		let renderer =  match sdl2::render::Renderer::from_window(window, sdl2::render::RenderDriverIndex::Auto, sdl2::render::SOFTWARE){
			Ok(renderer) => renderer,
			Err(err) => panic!("")
		};
		renderer.set_logical_size(1300/l, 600);
		'main : loop {
			match sdl2::event::poll_event() {
				sdl2::event::Event::Quit(_) => break 'main,
				_ => {}
			};
			match x.recv() {
				Ok(d) => draw_vector_as_barplot(&renderer, d),
				Err(_) => {}
			};
			renderer.present();
		};
	    sdl2::quit();
	}
}

