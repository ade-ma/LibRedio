extern crate kpn;
extern crate serialize;

use kpn::Packet;

#[deriving(Encodable,Decodable)]
pub enum DataTypes {
	temperature,
	relativeHumidity
}

#[deriving(Encodable,Decodable)]
pub struct Reading {
	Lower: ~[uint],
	SensorType: uint,
	SensorConstant: uint,
	SensorReading: f64,
	ReadingTimestamp: f64
}
