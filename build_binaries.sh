
cargo build --release
cp ./target/release/holier-than-thou ./holier

cargo build --release --target x86_64-pc-windows-gnu
cp ./target/x86_64-pc-windows-gnu/release/holier-than-thou.exe ./holier.exe

