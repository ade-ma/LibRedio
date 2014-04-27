CC=rustc

CFLAGS=-O -L ./lib -A unused-variable -A unused-imports
OBJ = ./lib/librtlsdr*.rlib ./lib/libdsputils*.rlib ./lib/libkpn*.rlib ./lib/libsensors*.rlib ./lib/libbitfount*.rlib ./lib/libinstant*.rlib

ifeq ($(ARCH),arm)
CFLAGS+= --target arm-unknown-linux-gnueabihf -C linker=arm-linux-gnueabihf-gcc -C link-args=-Wl,-rpath-link,$(PWD)/lib/ -C target-feature=+vfp3,+v7,+neon,+vfp4
else
OBJ += ./lib/libsdl2*.rlib ./lib/libvidsink2*.rlib ./lib/libsndfile*.rlib ./lib/libwavio*.rlib
endif

all: $(OBJ)
	mkdir -p bin
	$(CC) $(CFLAGS) ./src/temps.rs
	mv temps bin

test:
	$(CC) $(CFLAGS) ./src/test.rs

./lib/libsndfile*.rlib:
	mkdir -p lib
	$(CC) $(CFLAGS) --crate-type=lib ./src/sndfile.rs
	mv -f *rlib lib

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
	$(CC) $(CFLAGS) --crate-type=lib $<
	mv -f *rlib lib

clean:
	rm -r lib/*rlib bin
