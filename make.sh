ARGS="-O --lib -L ."
mkdir -p src
cd src
rustc $ARGS rtlsdr.rs
rustc $ARGS dsputils.rs
rustc $ARGS kpn.rs
rustc $ARGS bitfount.rs
rustc $ARGS unpackers.rs
rustc $ARGS instant.rs
rustc -O -L. amt.rs
rustc -O -L. temp.rs
mv amt temp ..
mv *so ..
cd ..
