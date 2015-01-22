extern crate kpn;
extern crate collections;
extern crate bitfount;
extern crate rtlsdr;
extern crate vidsink2;
extern crate kissfft;
extern crate num;
extern crate dsputils;
extern crate time;
extern crate pasimple;
extern crate core;
//extern crate rustrt;

use core::f32::consts::*;
use kissfft::fft;
use collections::bitv;
use rtlsdr::*;
use pasimple::*;
use kpn::*;
use bitfount::*;
use vidsink2::*;
use std::thread;
use num::complex;
use std::rand::{random, Closed01};
use std::sync::mpsc::{Receiver, Sender, channel};//, Messages};
use std::vec;

pub fn main() {
    let (txN078001N079001, rxN078001N079001) = channel();
    let (txN079001N080001, rxN079001N080001) = channel();
    let (txN080001N081001, rxN080001N081001) = channel();
    let (txN081001N082001, rxN081001N082001) = channel();
    let (txN082001N083001, rxN082001N083001) = channel();
    let (txN083001N0840, rxN083001N0840) = channel();
    let (txN083001N084001, rxN083001N084001) = channel();
    let (txN083001N084002, rxN083001N084002) = channel();
    let (txN084001N085001, rxN084001N085001) = channel();
    let (txN084002N085002, rxN084002N085002) = channel();
    let (txN085001N0860, rxN085001N0860) = channel();
    let (txN085001N086001, rxN085001N086001) = channel();
    let (txN085001N086002, rxN085001N086002) = channel();
    let (txN085002N086003, rxN085002N086003) = channel();
    let (txN086001N087001, rxN086001N087001) = channel();
    let (txN086002N087002, rxN086002N087002) = channel();
    let (txN086003N090003, rxN086003N090003) = channel();
    let (txN087001N088001, rxN087001N088001) = channel();
    let (txN087002N088002, rxN087002N088002) = channel();
    let (txN088001N089001, rxN088001N089001) = channel();
    let (txN088002N089002, rxN088002N089002) = channel();
    let (txN089001N090001, rxN089001N090001) = channel();
    let (txN089002N090002, rxN089002N090002) = channel();
    let (txN090001N091001, rxN090001N091001) = channel();
    let (txN090002N091001, rxN090002N091001) = channel();
    let (txN090003N091001, rxN090003N091001) = channel();
    let (txN091001N092001, rxN091001N092001) = channel();
    let (txN092001N093001, rxN092001N093001) = channel();
    let (txN093001N094001, rxN093001N094001) = channel();
    thread::Builder::new().name(String::from_str("rtl_source_cmplx")).spawn(move || {
                                                             rtl_source_cmplx(txN078001N079001,
                                                                              433900000, 402, 256000)
                                                         });
    thread::Builder::new().name(String::from_str("cross_applicator_vecs")).spawn(move || {
                                                                  cross_applicator_vecs(rxN078001N079001,
                                                                                        txN079001N080001,
                                                                                        &|&:x|{x.norm()})
                                                              });
    thread::Builder::new().name(String::from_str("trigger")).spawn(move || {
                                                    trigger(rxN079001N080001,
                                                            txN080001N081001)
                                                });
    thread::Builder::new().name(String::from_str("discretize")).spawn(move || {
                                                       discretize(rxN080001N081001,
                                                                  txN081001N082001)
                                                   });
    thread::Builder::new().name(String::from_str("rle")).spawn(move || {
                                                rle(rxN081001N082001,
                                                    txN082001N083001) });
    thread::Builder::new().name(String::from_str("fork")).spawn(move || {
                                                 fork(rxN083001N0840,
                                                      &[txN083001N084001,
                                                       txN083001N084002]) });
    thread::Builder::new().name(String::from_str("dle")).spawn(move || {
                                                dle(rxN082001N083001,
                                                    txN083001N0840, 256000)
                                            });
    thread::Builder::new().name(String::from_str("looper")).spawn(move || {
                                                   looper(rxN083001N084001,
                                                          txN084001N085001,
                                                          &mut|&mut:mut a,b| {a.map(|&mut :x| {b.send(match x { (1, 2e-4...6e-4) => match a.next().unwrap() { (0, 1.5e-3...2.5e-3) => Some(0us), (0, 3.5e-3...4.5e-3) => Some(1us), _ => None}, _     => None})}).last();()})
                                               });
    thread::Builder::new().name(String::from_str("looper")).spawn(move || {
                                                   looper(rxN083001N084002,
                                                          txN084002N085002,
                                                          &mut|&mut:mut a,b| {a.map(|&mut :x| {b.send(match x {(1, ref d@125e-6...250e-6) | (1, ref d @500e-6...650e-6) => {match a.next().unwrap() {(0, ref e @500e-6...650e-6) | (0, ref e@125e-6...250e-6) => Some(if d > e {1us} else {0us}),_ => None}}, _ => None})}).last();()})
                                               });
    thread::Builder::new().name(String::from_str("fork")).spawn(move || {
                                                 fork(rxN085001N0860,
                                                      &[txN085001N086001,
                                                       txN085001N086002]) });
    thread::Builder::new().name(String::from_str("shaper_optional")).spawn(move || {
                                                            shaper_optional(rxN084001N085001,
                                                                            txN085001N0860,
                                                                            36)
                                                        });
    thread::Builder::new().name(String::from_str("shaper_optional")).spawn(move || {
                                                            shaper_optional(rxN084002N085002,
                                                                            txN085002N086003,
                                                                            24)
                                                        });
    thread::Builder::new().name(String::from_str("binconv")).spawn(move || {
                                                    binconv(rxN085001N086001,
                                                            txN086001N087001,
                                                            &[4,8,4,12,8]) });
    thread::Builder::new().name(String::from_str("binconv")).spawn(move || {
                                                    binconv(rxN085001N086002,
                                                            txN086002N087002,
                                                            &[4,8,2,10,12]) });
    thread::Builder::new().name(String::from_str("applicator")).spawn(move || {
                                                       applicator(rxN085002N086003,
                                                                  txN086003N090003,
                                                                  &|&:mut x|{x.push(0); x})
                                                   });
    thread::Builder::new().name(String::from_str("cross_applicator")).spawn(move || {
                                                             cross_applicator(rxN086001N087001,
                                                                              txN087001N088001,
                                                                              &|&:x| {if x[0] == 5 {Some(x)} else {None}})
                                                         });
    thread::Builder::new().name(String::from_str("cross_applicator")).spawn(move || {
                                                             cross_applicator(rxN086002N087002,
                                                                              txN087002N088002,
                                                                              &|&:x| {if x[0] != 5 {Some(x)} else {None}})
                                                         });
    thread::Builder::new().name(String::from_str("looper_optional")).spawn(move || {
                                                            looper_optional(rxN087001N088001,
                                                                            txN088001N089001)
                                                        });
    thread::Builder::new().name(String::from_str("looper_optional")).spawn(move || {
                                                            looper_optional(rxN087002N088002,
                                                                            txN088002N089002)
                                                        });
    thread::Builder::new().name(String::from_str("cross_applicator_vecs")).spawn(move || {
                                                                  cross_applicator_vecs(rxN088001N089001,
                                                                                        txN089001N090001,
                                                                                        &|&x| {x as f32})
                                                              });
    thread::Builder::new().name(String::from_str("cross_applicator_vecs")).spawn(move || {
                                                                  cross_applicator_vecs(rxN088002N089002,
                                                                                        txN089002N090002,
                                                                                        &|&x| {x as f32})
                                                              });
    thread::Builder::new().name(String::from_str("mul_vecs")).spawn(move || {
                                                     mul_vecs(rxN089001N090001,
                                                              txN090001N091001,
                                                              vec!(1., 1., 1., 1e-1, 1.))
                                                 });
    thread::Builder::new().name(String::from_str("mul_vecs")).spawn(move || {
                                                     mul_vecs(rxN089002N090002,
                                                              txN090002N091001,
                                                              vec!(1., 1., 1., 1e-1, 1e-1))
                                                 });
    thread::Builder::new().name(String::from_str("cross_applicator_vecs")).spawn(move || {
                                                                  cross_applicator_vecs(rxN086003N090003,
                                                                                        txN090003N091001,
                                                                                        &|&x| {x as f32})
                                                              });
    thread::Builder::new().name(String::from_str("grapes")).spawn(move || {
                                                   grapes(&[rxN090001N091001,
                                                           rxN090002N091001,
                                                           rxN090003N091001],
                                                          txN091001N092001)
                                               });
    thread::Builder::new().name(String::from_str("differentiator")).spawn(move || {
                                                           differentiator(rxN091001N092001,
                                                                          txN092001N093001)
                                                       });
    thread::Builder::new().name(String::from_str("cross_applicator")).spawn(move || {
                                                             cross_applicator(rxN092001N093001,
                                                                              txN093001N094001,
                                                                              &|x| {(time::get_time().sec, x)})
                                                         });
    thread::Builder::new().name(String::from_str("print_sink")).spawn(move || {
                                                       print_sink(rxN093001N094001)
                                                   });
}
