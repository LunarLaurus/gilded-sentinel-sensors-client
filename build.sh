#!/bin/bash
# Build script for Linux

echo "Building for Linux..."
cargo build --target x86_64-unknown-linux-gnu

echo "Building for Windows..."
# Ensure MinGW or the appropriate cross-compilation toolchain is installed
cargo build --target x86_64-pc-windows-gnu

echo "Build completed for all platforms."
