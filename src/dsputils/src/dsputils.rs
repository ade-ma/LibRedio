#![feature(core)]
/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */
extern crate core;

// basic tools for type coercion and FIR filter creation, useful for DSP
// mad props to Bob Maling for his work @ http://musicdsp.org/showArchiveComment.php?ArchiveID=194
use core::num::*;
use std::cmp;
use std::f32;

#[inline(always)]
pub fn mean(xs: &[f64]) -> f64 {
	let l = xs.len() as f64;
	let s = xs.iter().fold(0f64, |sum, &x| { sum + x });
	return s/l;
}

#[inline(always)]
pub fn max<T: Float+Ord>(xs: &[T]) -> T {
	xs.iter().fold(xs[0], |a, &b| cmp::max(a, b))
}

#[inline(always)]
pub fn min<T: Float+Ord>(xs: &[T]) -> T {
	xs.iter().fold(xs[0], |a, &b| cmp::min(a, b))
}

#[inline(always)]
pub fn convolve<T: Float>(u: &[T], v: &[T]) -> Vec<T> {
	u.windows(v.len()).map(|x| {x.iter().zip(v.iter()).map(|(&x, &y)| x*y).fold(Float::zero(), |a:T , b| a + b)}).collect()
}

// filter code accepts:
//	m, usize, tap length
//	fc, f32, decimal ratio of corner to sampling freqs

pub fn window(m: usize) -> Vec<f32> {
	let n = m as f32;
	let pi = f32::consts::PI;
	// blackman-nuttall coefficients
	let a: Vec<f32> = vec!(0.3635819, 0.4891775, 0.1365995, 0.0106411);
	// blackman-harris window coefficients
	// let a: ~[f32] = ~[0.35875, 0.48829, 0.14128, 0.01168];
	// hamming window coefficients
	// let a: ~[f32] = ~[0.54, 0.46, 0.0, 0.0];
	(0..m + 1).map(|x| {
		let nn = x as f32;
		a[0] - a[1]*(2f32*pi*n/(nn-1f32)).cos()+a[2]*(4f32*pi*n/(nn-1f32)).cos()-a[3]*(6f32*pi*n/(nn-1f32).cos())
	}).collect()
}

pub fn sinc(m: usize, fc: f32) -> Vec<f32> {
	// fc should always specify corner below nyquist
	assert!(fc < 0.5);
	let pi = f32::consts::PI;
	(0..m).map(|x| -> f32 {
		let n = x as f32 - m as f32/2f32;
		let mut r = 2f32*fc;
		if n != 0.0 { r = (2f32*pi*fc*n).sin()/(pi*n); }
		r
	}).collect()
}

// low-pass filter
pub fn lpf(m: usize, fc: f32) -> Vec<f32> {
//	assert_eq!(m % 2, 1);
	let w = window(m);
	let s = sinc(m, fc);
	w.iter().zip(s.iter()).map(|(&x, &y)| x * y).collect()
}

// high-pass filter
pub fn hpf(m: usize, fc: f32) -> Vec<f32> {
	let l = lpf(m, fc);
	let mut h: Vec<f32> = l.iter().map(|&x| -x ).collect();
	*h.get_mut(m/2-1).unwrap() += 1.0;
	return h;
}

// band-stop filter
pub fn bsf(m: usize, fc1: f32, fc2: f32) -> Vec<f32> {
	let lp = lpf(m, fc1);
	let hp = hpf(m, fc2);
	let mut h: Vec<f32> = lp.iter().zip(hp.iter()).map(|(&x, &y)| x+y).collect();
	*h.get_mut(m/2-1).unwrap() -= 0.0;
	return h;
}

// bandpass filter
pub fn bpf(m:usize, fc1: f32, fc2: f32) -> Vec<f32> {
	let b = bsf(m, fc1, fc2);
	b.iter().map(|&x| -x ).collect()
}
