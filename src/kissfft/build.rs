use std::process::Command;
use std::env;

fn main() {
	Command::new("make").args(&["-C", "libkissfft"]).status().unwrap();
	println!("cargo:rustc-link-search=native={}{}", env::var("PWD").unwrap(), "/libkissfft/");
	println!("cargo:rustc-link-lib=static=kissfft");
}

