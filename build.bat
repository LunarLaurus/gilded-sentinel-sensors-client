@echo off
REM Build script for Windows
echo Building for Windows...
cargo build --target x86_64-pc-windows-msvc
echo Build completed for all platforms.