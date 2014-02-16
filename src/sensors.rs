extern crate readings;
extern crate msgpack;
extern crate kpn;
extern crate extra;
extern crate native;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::net::udp::UdpSocket;
use std::io::{Listener, Acceptor};

use msgpack::{Array, Unsigned, Double, Value, String};
use extra::time;
use kpn::{Token, Chip, Packet, SourceConf, Dbl, Break, Run, Dur};
use std::comm::{Port, Chan};

// recursive encoding of a Token to a msgpack Value
pub fn tokenToValue(U: Token) -> Value {
	match U {
		Packet(p) => Array(p.move_iter().map(|x| tokenToValue(x)).to_owned_vec()),
		Dbl(x) => Double(x),
		Chip(x) => Unsigned(x as u64),
		Break(s) => String(s.into_bytes()),
		Dur(~t,d) => Array(~[tokenToValue(t), tokenToValue(Dbl(d))]),
		Run(~t,d) => Array(~[tokenToValue(t), tokenToValue(Chip(d))]),
	}
}

pub fn udpTokenSink(U: Port<Token>, S: SourceConf) {
	let mut sock = UdpSocket::bind(SocketAddr{ip:Ipv4Addr(127,0,0,1), port:9998}).unwrap();
	loop {
		let v = tokenToValue(U.recv());
		sock.sendto(msgpack::to_msgpack(&v), SocketAddr{ip:Ipv4Addr(127,0,0,1), port:9999});
	}
}

pub fn tempA(U: Port<Token>, V: Chan<Token>, S: SourceConf) {
	loop {
		match U.recv() {
			Packet(p) => {
				let now = time::get_time();
				let l = p.clone().move_iter().filter_map(|x| match x { Chip(c) => Some(c), _ => None }).to_owned_vec();
				V.send(Packet(~[Packet(p.clone()), Chip(0), Chip(l[0]+l[1]), Dbl(l[2] as f64 / 10f64), Dbl(now.sec as f64 + now.nsec as f64 * 1e-9)]));
				V.send(Packet(~[Packet(p.clone()), Chip(1), Chip(l[0]+l[1]), Dbl(l[3] as f64), Dbl(now.sec as f64 + now.nsec as f64 * 1e-9)]));
			},
			x => println!("{:?}", x),
		}
	}
}
