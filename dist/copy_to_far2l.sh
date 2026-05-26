#!/bin/bash
# copy_to_far2l.sh — Copies the built far1c plugin into far2l FHS directories.
# Run this AFTER building: cargo build --release --no-default-features --features far2
# Requires sudo.

set -e

DIST_DIR="$(dirname "$0")/../target/release/far2"

if [ ! -f "$DIST_DIR/far1c.far-plug-wide" ]; then
    echo "Error: '$DIST_DIR/far1c.far-plug-wide' not found."
    echo "Please build first: cargo build --release --no-default-features --features far2"
    exit 1
fi

echo "Installing far1c plugin for far2l..."

sudo mkdir -p /usr/lib/far2l/Plugins/far1c
sudo cp "$DIST_DIR/far1c.far-plug-wide" /usr/lib/far2l/Plugins/far1c/

sudo mkdir -p /usr/share/far2l/Plugins/far1c/plug
sudo cp "$DIST_DIR/far1c_en.lng" "$DIST_DIR/far1c_ru.lng" /usr/share/far2l/Plugins/far1c/plug/

echo "Installation complete! Please restart far2l."
