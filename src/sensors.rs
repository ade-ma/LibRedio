extern crate readings;
extern crate msgpack;
extern crate kpn;
extern crate extra;
extern crate native;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::net::tcp::TcpListener;
use std::io::{Listener, Acceptor};

use msgpack::{Array, Unsigned, Double};
use extra::time;
use kpn::{Token, Chip, Packet, SourceConf};
use std::comm::{Port, Chan};

pub fn tempA(U: Port<Token>, S: SourceConf) {
	let mut stream = TcpListener::bind(SocketAddr{ip:Ipv4Addr(127,0,0,1), port:9991});
	for mut c in stream.listen().incoming() {
	loop {
		match U.recv() {
			Packet(p) => {
					let now = time::get_time();
					let l = p.clone().move_iter().filter_map(|x| match x { Chip(c) => Some(c), _ => None }).to_owned_vec();
					let rt = (l.clone(), 0, l[0]+l[1], l[2] as f64 / 10f64, now.sec as f64 + now.nsec as f64 * 1e-9 ); // tuple serializes well enough
					let rh = (l.clone(), 0, l[0]+l[1], l[3] as f64, now.sec as f64 + now.nsec as f64 * 1e-9 );
					let msgt: ~[u8] = msgpack::to_msgpack(&rt);
					let msgh: ~[u8] = msgpack::to_msgpack(&rh);
					c.clone().write(msgt.slice_from(0));
					c.clone().write(msgh.slice_from(0));
			}
			x => println!("{:?}", x),
		}
	}
	}
}
