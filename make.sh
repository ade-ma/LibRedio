ARGS="-O --lib -L ."
rm bin/*
mkdir -p src bin
cd src
rustc $ARGS rtlsdr.rs
rustc $ARGS dsputils.rs
rustc $ARGS kpn.rs
rustc $ARGS bitfount.rs
rustc $ARGS unpackers.rs
rustc $ARGS instant.rs
rustc -O -L. temps.rs
mv temps ../bin
mv *so ../bin
cd ..
