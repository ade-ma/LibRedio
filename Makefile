CC=rustc
CFLAGS=-O --crate-type=lib -L ./ -L ./lib -A unused-variable -A unused-imports

OBJ = ./lib/librtlsdr*.rlib ./lib/libdsputils*.rlib ./lib/libkpn*.rlib ./lib/libsensors*.rlib ./lib/libbitfount*.rlib ./lib/libinstant*.rlib

all: ./lib/libmsgpack*.rlib  $(OBJ)
	mkdir -p lib bin
	$(CC) -O -L ./ -L ./lib ./src/temps.rs
	-mv -f *rlib lib
	mv temps bin

./lib/libmsgpack*.rlib:
	make -C ../rust-msgpack/
	mv ../rust-msgpack/lib/libmsgpack* ./

./lib/lib%.rlib: ./src/%.rs
	$(CC) $(CFLAGS) $<

clean:
	rm -rf lib bin
