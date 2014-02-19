CC=rustc
CFLAGS=-O --crate-type=lib -L ./lib -A unused-variable -A unused-imports

OBJ = ./lib/libmsgpack*.rlib ./lib/librtlsdr*.rlib ./lib/libdsputils*.rlib ./lib/libkpn*.rlib ./lib/libsensors*.rlib ./lib/libbitfount*.rlib ./lib/libinstant*.rlib ./lib/libsdl*.rlib ./lib/libvidsink*.rlib ./lib/libsdl2*.rlib ./lib/libvidsink2*.rlib 

all: $(OBJ)
	mkdir -p bin
	$(CC) -O -L ./lib ./src/temps.rs
	mv temps bin

./lib/libmsgpack*.rlib:
	mkdir -p lib
	make -C ../rust-msgpack/
	mv ../rust-msgpack/lib/libmsgpack* ./lib

./lib/libsdl2*.rlib:
	mkdir -p lib
	make -C ../rust-sdl2
	mv ../rust-sdl2/build/lib/libsdl2* ./lib

./lib/libsdl*.rlib:
	mkdir -p lib
	$(CC) $(CFLAGS) ../rust-sdl/src/sdl/lib.rs
	mv *rlib lib


./lib/lib%.rlib: ./src/%.rs
	mkdir -p lib
	$(CC) $(CFLAGS) $<
	mv -f *rlib lib

clean:
	rm -rf lib bin
