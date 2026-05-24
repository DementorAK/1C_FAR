#!/bin/bash
set -e

echo "Building far1c for far2l/far2m..."
cargo build --release --no-default-features --features far2

echo "Installing plugin (requires sudo)..."
sudo mkdir -p /usr/lib/far2l/Plugins/far1c
sudo cp target/release/libfar1c.so /usr/lib/far2l/Plugins/far1c/far1c.far-plug-wide

sudo mkdir -p /usr/share/far2l/Plugins/far1c/plug
sudo cp far1c_en.lng far1c_ru.lng /usr/share/far2l/Plugins/far1c/plug/

echo "Installation complete! Please restart far2l."
