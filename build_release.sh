#!/bin/bash
set -e

echo "Start building Far1C for far2l..."
cargo build --release --no-default-features --features far2l

mkdir -p target/release/far2l
cp target/release/libfar1c.so target/release/far2l/far1c.far-plug-wide
cp dist/*.lng target/release/far2l/ || true

if [ -f "dist/copy_to_far2l.sh" ]; then
    cp dist/copy_to_far2l.sh target/release/far2l/
fi
echo "Build complete. Optional step: Run copy_to_far2l.sh to quick install Far1C on Far2l."

echo "Start building Far1C for far2m..."
export FAR1C_KEEP_BUILD_NUM=1
cargo build --release --no-default-features --features far2m

mkdir -p target/release/far2m
cp target/release/libfar1c.so target/release/far2m/far1c.far-plug-wide
cp dist/*.lng target/release/far2m/ || true

echo "Build complete."
