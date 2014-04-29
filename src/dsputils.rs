/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */

extern crate num;

use num::complex;

// basic tools for type coercion and FIR filter creation, useful for DSP
// mad props to Bob Maling for his work @ http://musicdsp.org/showArchiveComment.php?ArchiveID=194
use std::num;
use std::iter::AdditiveIterator;
use std::num::Zero;
use std::f32;

// helper functions useful for FFT work
pub fn asRe ( d: ~[f32] ) -> ~[complex::Complex32] { return d.iter().map(|&x| {complex::Cmplx {re: x, im: 0.0}}).collect::<~[complex::Complex32]>();}
pub fn asF32 ( d: ~[complex::Complex32] ) -> ~[f32] { return d.iter().map(|&x| {if num::abs(x.im) < 0.001 { x.re } else { let (m,p) = x.to_polar(); m*num::signum(p) }}).collect::<~[f32]>(); }
pub fn asF64 ( d: ~[f32] ) -> ~[f64] { return d.iter().map(|&x| x as f64).collect(); }


pub fn sum<T: Num+Ord+Primitive>(xs: &[T]) -> T {
	let mut out: T = num::zero();
	if xs.len() != 0 {
		for i in range(0, xs.len()) {
			out = out + xs[i];
		}
	}
	return out
}

#[inline(always)]
pub fn mean<T: Num+Ord+Primitive+ToPrimitive>(xs: &[T]) -> f32 {
	let l = xs.len() as f32;
	let s = sum(xs).to_f32().unwrap()/l;
	return s/l
}

#[inline(always)]
pub fn max<T: Float+Num>(xs: &[T]) -> T {
	xs.iter().fold(xs[0], |a, &b| a.max(b))
}

#[inline(always)]
pub fn min<T: Float+Num>(xs: &[T]) -> T {
	xs.iter().fold(xs[0], |a, &b| a.min(b))
}

#[inline(always)]
pub fn convolve<T: Num+Ord+Primitive+ToPrimitive>(u: &[T], v: &[T]) -> Vec<T> {
	u.windows(v.len()).map(|x| {x.iter().zip(v.iter()).map(|(&x, &y)| x*y).sum()}).collect()
}

// filter code accepts:
//	m, uint, tap length
//	fc, f32, decimal ratio of corner to sampling freqs

pub fn window(m: uint) -> Vec<f32> {
	let N = m as f32;
	let pi = f32::consts::PI;
	// blackman-nuttall coefficients
	let a: ~[f32] = ~[0.3635819, 0.4891775, 0.1365995, 0.0106411];
	// blackman-harris window coefficients
	// let a: ~[f32] = ~[0.35875, 0.48829, 0.14128, 0.01168];
	// hamming window coefficients
	// let a: ~[f32] = ~[0.54, 0.46, 0.0, 0.0];
	range(0, m + 1).map(|x| {
		let n = x as f32;
		a[0] - a[1]*(2f32*pi*n/(N-1f32)).cos()+a[2]*(4f32*pi*n/(N-1f32)).cos()-a[3]*(6f32*pi*n/(N-1f32).cos())
	}).collect()
}

pub fn sinc(m: uint, fc: f32) -> Vec<f32> {
	// fc should always specify corner below nyquist
	assert!(fc < 0.5);
	let pi = f32::consts::PI;
	range(0, m).map(|x| -> f32 {
		let n = x as f32 - m as f32/2f32;
		let mut r = 2f32*fc;
		if n != 0.0 { r = (2f32*pi*fc*n).sin()/(pi*n); }
		r
	}).collect()
}

// low-pass filter
pub fn lpf(m: uint, fc: f32) -> Vec<f32> {
//	assert_eq!(m % 2, 1);
	let w = window(m);
	let s = sinc(m, fc);
	w.iter().zip(s.iter()).map(|(&x, &y)| x * y).collect()
}

// high-pass filter
pub fn hpf(m: uint, fc: f32) -> Vec<f32> {
	let l = lpf(m, fc);
	let mut h: Vec<f32> = l.iter().map(|&x| -x ).collect();
	*h.get_mut(m/2-1) += 1.0;
	return h;
}

// band-stop filter
pub fn bsf(m: uint, fc1: f32, fc2: f32) -> Vec<f32> {
	let lp = lpf(m, fc1);
	let hp = hpf(m, fc2);
	let mut h: Vec<f32> = lp.iter().zip(hp.iter()).map(|(&x, &y)| x+y).collect();
	*h.get_mut(m/2-1) -= 1.0;
	return h;
}

// bandpass filter
pub fn bpf(m:uint, fc1: f32, fc2: f32) -> Vec<f32> {
	let b = bsf(m, fc1, fc2);
	b.iter().map(|&x| -x ).collect()
}
