#![feature(globs)];
extern crate kpn;
extern crate collections;
extern crate bitfount;
extern crate rtlsdr;
extern crate native;
extern crate vidsink2;
extern crate kissfft;
extern crate num;
extern crate dsputils;
extern crate time;
extern crate pasimple;
extern crate core;
extern crate rustrt;

use core::f32::consts::*;
use kissfft::fft;
use collections::bitv;
use rtlsdr::*;
use pasimple::*;
use kpn::*;
use bitfount::*;
use vidsink2::*;
use std::task;
use num::complex;
use std::rand::{random, Closed01};
use std::comm::{Receiver, Sender, channel, Messages};
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
    task::TaskBuilder::new().named("rtl_source_cmplx").spawn(proc() {
                                                             rtl_source_cmplx(txN078001N079001,
                                                                              433900000, 402, 256000)
                                                         });
    task::TaskBuilder::new().named("cross_applicator_vecs").spawn(proc() {
                                                                  cross_applicator_vecs(rxN078001N079001,
                                                                                        txN079001N080001,
                                                                                        |x|{x.norm()})
                                                              });
    task::TaskBuilder::new().named("trigger").spawn(proc() {
                                                    trigger(rxN079001N080001,
                                                            txN080001N081001)
                                                });
    task::TaskBuilder::new().named("discretize").spawn(proc() {
                                                       discretize(rxN080001N081001,
                                                                  txN081001N082001)
                                                   });
    task::TaskBuilder::new().named("rle").spawn(proc() {
                                                rle(rxN081001N082001,
                                                    txN082001N083001) });
    task::TaskBuilder::new().named("fork").spawn(proc() {
                                                 fork(rxN083001N0840,
                                                      [txN083001N084001,
                                                       txN083001N084002]) });
    task::TaskBuilder::new().named("dle").spawn(proc() {
                                                dle(rxN082001N083001,
                                                    txN083001N0840, 256000)
                                            });
    task::TaskBuilder::new().named("looper").spawn(proc() {
                                                   looper(rxN083001N084001,
                                                          txN084001N085001,
                                                          |mut a,b| {a.map(|x| {b.send(match x { (1, 2e-4...6e-4) => match a.next().unwrap() { (0, 1.5e-3...2.5e-3) => Some(0u), (0, 3.5e-3...4.5e-3) => Some(1u), _ => None}, _     => None})}).last();()})
                                               });
    task::TaskBuilder::new().named("looper").spawn(proc() {
                                                   looper(rxN083001N084002,
                                                          txN084002N085002,
                                                          |mut a,b| {a.map(|x| {b.send(match x {(1, ref d@125e-6...250e-6) | (1, ref d @500e-6...650e-6) => {match a.next().unwrap() {(0, ref e @500e-6...650e-6) | (0, ref e@125e-6...250e-6) => Some(if d > e {1u} else {0u}),_ => None}}, _ => None})}).last();()})
                                               });
    task::TaskBuilder::new().named("fork").spawn(proc() {
                                                 fork(rxN085001N0860,
                                                      [txN085001N086001,
                                                       txN085001N086002]) });
    task::TaskBuilder::new().named("shaper_optional").spawn(proc() {
                                                            shaper_optional(rxN084001N085001,
                                                                            txN085001N0860,
                                                                            36)
                                                        });
    task::TaskBuilder::new().named("shaper_optional").spawn(proc() {
                                                            shaper_optional(rxN084002N085002,
                                                                            txN085002N086003,
                                                                            24)
                                                        });
    task::TaskBuilder::new().named("binconv").spawn(proc() {
                                                    binconv(rxN085001N086001,
                                                            txN086001N087001,
                                                            [4,8,4,12,8]) });
    task::TaskBuilder::new().named("binconv").spawn(proc() {
                                                    binconv(rxN085001N086002,
                                                            txN086002N087002,
                                                            [4,8,2,10,12]) });
    task::TaskBuilder::new().named("applicator").spawn(proc() {
                                                       applicator(rxN085002N086003,
                                                                  txN086003N090003,
                                                                  |mut x|{x.push(0); x})
                                                   });
    task::TaskBuilder::new().named("cross_applicator").spawn(proc() {
                                                             cross_applicator(rxN086001N087001,
                                                                              txN087001N088001,
                                                                              |x| {if x[0] == 5 {Some(x)} else {None}})
                                                         });
    task::TaskBuilder::new().named("cross_applicator").spawn(proc() {
                                                             cross_applicator(rxN086002N087002,
                                                                              txN087002N088002,
                                                                              |x| {if x[0] != 5 {Some(x)} else {None}})
                                                         });
    task::TaskBuilder::new().named("looper_optional").spawn(proc() {
                                                            looper_optional(rxN087001N088001,
                                                                            txN088001N089001)
                                                        });
    task::TaskBuilder::new().named("looper_optional").spawn(proc() {
                                                            looper_optional(rxN087002N088002,
                                                                            txN088002N089002)
                                                        });
    task::TaskBuilder::new().named("cross_applicator_vecs").spawn(proc() {
                                                                  cross_applicator_vecs(rxN088001N089001,
                                                                                        txN089001N090001,
                                                                                        |&x| {x as f32})
                                                              });
    task::TaskBuilder::new().named("cross_applicator_vecs").spawn(proc() {
                                                                  cross_applicator_vecs(rxN088002N089002,
                                                                                        txN089002N090002,
                                                                                        |&x| {x as f32})
                                                              });
    task::TaskBuilder::new().named("mul_vecs").spawn(proc() {
                                                     mul_vecs(rxN089001N090001,
                                                              txN090001N091001,
                                                              vec!(1., 1., 1., 1e-1, 1.))
                                                 });
    task::TaskBuilder::new().named("mul_vecs").spawn(proc() {
                                                     mul_vecs(rxN089002N090002,
                                                              txN090002N091001,
                                                              vec!(1., 1., 1., 1e-1, 1e-1))
                                                 });
    task::TaskBuilder::new().named("cross_applicator_vecs").spawn(proc() {
                                                                  cross_applicator_vecs(rxN086003N090003,
                                                                                        txN090003N091001,
                                                                                        |&x| {x as f32})
                                                              });
    task::TaskBuilder::new().named("grapes").spawn(proc() {
                                                   grapes([rxN090001N091001,
                                                           rxN090002N091001,
                                                           rxN090003N091001],
                                                          txN091001N092001)
                                               });
    task::TaskBuilder::new().named("differentiator").spawn(proc() {
                                                           differentiator(rxN091001N092001,
                                                                          txN092001N093001)
                                                       });
    task::TaskBuilder::new().named("cross_applicator").spawn(proc() {
                                                             cross_applicator(rxN092001N093001,
                                                                              txN093001N094001,
                                                                              |x| {(time::get_time().sec, x)})
                                                         });
    task::TaskBuilder::new().named("print_sink").spawn(proc() {
                                                       print_sink(rxN093001N094001)
                                                   });
}