CC=rustc
CFLAGS=-O --crate-type=lib -L ./ -L ./lib -A unused-variable -A unused-imports
DEPS = hellomake.h
OBJ = hellomake.o hellofunc.o


OBJ = ./lib/librtlsdr*.rlib ./lib/libdsputils*.rlib ./lib/libkpn*.rlib ./lib/libbitfount*.rlib ./lib/libinstant*.rlib

./lib/lib%.rlib: ./src/%.rs
	$(CC) $(CFLAGS) $<

all: $(OBJ)
	mkdir -p lib bin
	$(CC) -O -L ./ -L ./lib ./src/temps.rs
	-mv -f *rlib lib
	mv temps bin

clean:
	rm -rf lib bin
