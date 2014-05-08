extern crate collections;
extern crate oblw;
extern crate toml;
extern crate libc;

use collections::bitv;
use std::strbuf;

pub fn getCode(x: ~str) -> bitv::Bitv {
	let root = toml::parse_from_file("outlets.toml").unwrap();
	let mut q = strbuf::StrBuf::from_str("db.");
	q.push_str(x);
	let data = root.lookup(q.as_slice());
	match data {
		Some(data) => {
			let data = data.get_vec().unwrap();
			let mut y = vec!();
			for _ in range(0, 5) {
				data.iter().map(|bit| {
					match bit {
						&toml::PosInt(1) => y.push_all_move( vec!(oblw::Run {v: 1, ct: 561}, oblw::Run {v: 0, ct: 187})) ,
						&toml::PosInt(0) => y.push_all_move( vec!(oblw::Run {v: 1, ct: 187}, oblw::Run {v: 0, ct: 561})) ,
						x => println!("wat. got {:?}, expected 1/0",x)
					}}).last();
				y.push(oblw::Run {v: 0, ct: 5000});
			}
			oblw::v2b(oblw::rld(y))
		},
		None => { oblw::v2b(vec!()) }
	}
}
