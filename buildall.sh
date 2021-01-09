# npm install
npm run build
find ./build/ -iname *.map -delete
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release