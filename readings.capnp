@0xc2d14490a65b0af6;

enum DataTypes {
	temperature @0;
	relativeHumidity @1;
}

struct Reading {
	lower @0: List(Bool); # raw output from sensor
	sensorType @1: DataTypes;
	sensorConstant @2: UInt64; # some sort of ID, should always be constant between readings from same sensor
	sensorReading @3: Float64; # SI units
	readingTimestamp @4: Float64; # seconds since epoch
}
