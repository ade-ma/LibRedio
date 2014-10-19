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
use native::task;
use num::complex;
use std::rand::{random, Closed01};
use std::comm::{Receiver, Sender, channel, Messages};
use std::vec;

pub fn main() {
    let (txN002001N003001, rxN002001N003001) = channel();
    let (txN003001N004001, rxN003001N004001) = channel();
    let (txN004001N005001, rxN004001N005001) = channel();
    let (txN005001N006001, rxN005001N006001) = channel();
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("pulse_source".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() { pulse_source(txN002001N003001, 8000, 512)
                         });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("cross_applicator_vecs".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             cross_applicator_vecs(rxN002001N003001,
                                                 txN003001N004001,
                                                 |x|{complex::Complex::new(*x, core::num::Zero::zero())})
                         });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("fft".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             fft(rxN003001N004001, txN004001N005001, 512, 0)
                         });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("cross_applicator_vecs".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             cross_applicator_vecs(rxN004001N005001,
                                                 txN005001N006001,
                                                 |x|{x.norm()}) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("vid_sink_vecs".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() { vidsink_vecs(rxN005001N006001) });
}pub fn biquaddf2<T: core::num::Float +
                 core::kinds::Send>(rxN009001N010001: Receiver<T>,
                                    txN012004N013005: Sender<T>, a0: T, a1: T,
                                    a2: T, b1: T, b2: T) {
    let (txN010001N0110, rxN010001N0110) = channel();
    let (txN010001N011001, rxN010001N011001) = channel();
    let (txN010001N011002, rxN010001N011002) = channel();
    let (txN011001N0120, rxN011001N0120) = channel();
    let (txN011001N012001, rxN011001N012001) = channel();
    let (txN011001N012002, rxN011001N012002) = channel();
    let (txN011001N012003, rxN011001N012003) = channel();
    let (txN011002N012004, rxN011002N012004) = channel();
    let (txN012001N0130, rxN012001N0130) = channel();
    let (txN012001N013001, rxN012001N013001) = channel();
    let (txN012001N013002, rxN012001N013002) = channel();
    let (txN012002N010001, rxN012002N010001) = channel();
    let (txN012003N012004, rxN012003N012004) = channel();
    let (txN013001N010001, rxN013001N010001) = channel();
    let (txN013002N012004, rxN013002N012004) = channel();
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("fork".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             fork(rxN010001N0110,
                                  [txN010001N011001, txN010001N011002]) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("sum_across".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             sum_across([rxN009001N010001, rxN012002N010001,
                                        rxN013001N010001], txN010001N0110,
                                       core::num::Zero::zero()) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("fork".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             fork(rxN011001N0120,
                                  [txN011001N012001, txN011001N012002,
                                   txN011001N012003]) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("delay".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             delay(rxN010001N011001, txN011001N0120, core::num::Zero::zero()) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("mul".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             mul(rxN010001N011002, txN011002N012004, a0) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("fork".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             fork(rxN012001N0130,
                                  [txN012001N013001, txN012001N013002]) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("delay".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             delay(rxN011001N012001, txN012001N0130, core::num::Zero::zero()) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("mul".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             mul(rxN011001N012002, txN012002N010001, b1) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("mul".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             mul(rxN011001N012003, txN012003N012004, a1) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("sum_across".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             sum_across([rxN011002N012004, rxN012003N012004,
                                        rxN013002N012004], txN012004N013005,
                                       core::num::Zero::zero()) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("mul".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             mul(rxN012001N013001, txN013001N010001, b2) });
    native::task::spawn_opts(std::rt::task::TaskOpts{on_exit: None,
                                                     name:
                                                         Some("mul".into_maybe_owned()),
                                                     stack_size: None,},
                             proc() {
                             mul(rxN012001N013002, txN013002N012004, a2) });
}
