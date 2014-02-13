CC=rustc
CFLAGS=-O --crate-type=lib -L ./ -L ./lib -A unused-variable -A unused-imports
DEPS = hellomake.h
OBJ = hellomake.o hellofunc.o


OBJ = ./lib/librtlsdr*.rlib ./lib/libdsputils*.rlib ./lib/libkpn*.rlib ./lib/libbitfount*.rlib ./lib/libunpackers*.rlib ./lib/libinstant*.rlib

./lib/lib%.rlib: ./src/%.rs
	$(CC) $(CFLAGS) $<

all: $(OBJ)
	$(CC) -O -L ./ -L ./lib ./src/temps.rs
	mv *rlib lib
#rustc rtlsdr.rs
#rustc dsputils.rs
#rustc kpn.rs
#rustc bitfount.rs
#rustc unpackers.rs
#rustc instant.rs

