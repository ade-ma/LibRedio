use std::process::Command;
use std::env;

fn main() {
	Command::new("make").args(&["-C", "libkissfft", "install"]).status().unwrap();
	println!("cargo:rustc-link-lib=dylib=kissfft");
}

