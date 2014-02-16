CC=rustc
CFLAGS=-O --crate-type=lib -L ./ -L ./lib -A unused-variable -A unused-imports

OBJ = ./lib/librtlsdr*.rlib ./lib/libdsputils*.rlib ./lib/libkpn*.rlib ./lib/libreading*.rlib ./lib/libsensors*.rlib ./lib/libbitfount*.rlib ./lib/libinstant*.rlib

./lib/lib%.rlib: ./src/%.rs
	$(CC) $(CFLAGS) $<

all: clean msgpack $(OBJ)
	mkdir -p lib bin
	$(CC) -O -L ./ -L ./lib ./src/temps.rs
	-mv -f *rlib lib
	mv temps bin

msgpack:
	make -C ../rust-msgpack/
	mv ../rust-msgpack/lib/libmsgpack* ./


clean:
	rm -rf lib bin
