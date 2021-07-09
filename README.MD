# vulcast-rtc
Native Rust library for realtime communication with Relay, handling:
- Controller input over WebRTC DataChannel
- Audio/video encoding over WebRTC transport (post-MVP)

## What
- C++ library `vulcast-rtc` encapsulating a WebRTC client using MediaSoup
- Rust sys crate `vulcast-rtc-sys` exposing FFI declarations from `vulcast-rtc`
- Rust crate `vulcast-rtc` exposing high-level abstractions for `vulcast-rtc-sys`

## Build
### From prebuilt
Two host triples are prebuilt in debug mode, `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-gnu`.

1. Install cross-compiler toolchain for ARM64
```bash
# install rust cross toolchain
rustup target add aarch64-unknown-linux-gnu
# install system cross linker (e.g. for archlinux)
sudo pacman -S aarch64-linux-gnu-gcc
```
2. Build `vulcast-rtc`
```bash
cargo build --target=aarch64-unknown-linux-gnu
```
### From source
You will need:
- A 64-bit linux system with kernel >4.2
- At least 16 GB of RAM
- 20 GB of free disk space

1. [Install and build WebRTC for ARM64](https://webrtc.github.io/webrtc-org/native-code/development/) on M84 branch
(this will take several hours)
- You MUST use clang (i.e. `is_clang=true` for GN config)
- You MUST compile to a static library (no component builds)
- You will need to add a Debian sysroot for ARM64
2. Install cross-compiler toolchain for ARM64
```bash
# install rust cross toolchain
rustup target add aarch64-unknown-linux-gnu
# install system cross linker (e.g. for archlinux)
sudo pacman -S aarch64-linux-gnu-gcc
```
3. Build `vulcast-rtc`
```bash
export VULCAST_RTC_FROM_SOURCE=1
export LIBWEBRTC_INCLUDE_PATH=/path/to/webrtc/src
export LIBWEBRTC_BINARY_PATH=/path/to/webrtc/src/out/m84/obj
cargo build --target=aarch64-unknown-linux-gnu
```
