#!/bin/bash
set -e

echo "Building far1c for far2l/far2m..."
cargo build --release --no-default-features --features far2

echo "Packaging release..."
mkdir -p target/release/far2
cp target/release/libfar1c.so target/release/far2/far1c.far-plug-wide
cp dist/* target/release/far2/

echo "Build complete. Run copy_to_far2l.sh to install on Far2l."
