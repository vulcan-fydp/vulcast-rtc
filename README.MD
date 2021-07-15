# vulcast-rtc
Native Rust library for realtime communication with Relay, handling:
- Controller input over WebRTC DataChannel
- Audio/video encoding over WebRTC transport (post-MVP)

## What
- C++ library `vulcast-rtc` encapsulating a WebRTC client using MediaSoup
- Rust sys crate `vulcast-rtc-sys` exposing FFI declarations from `vulcast-rtc`
- Rust crate `vulcast-rtc` exposing high-level abstractions for `vulcast-rtc-sys`

## Build
```
cargo build
```

## Build (Cross-compiling)
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
# bindgen uses clang, so if your cross-compiler is gnu you need to add some include dirs
export BINDGEN_EXTRA_CLANG_ARGS='-I/usr/aarch64-linux-gnu/include/c++/11.1.0/aarch64-linux-gnu'
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
# install system cross compiler (e.g. for Arch Linux)
sudo pacman -S aarch64-linux-gnu-gcc lib32-glibc
```
3. Build `vulcast-rtc`
```bash
export VULCAST_RTC_FROM_SOURCE=1
export LIBWEBRTC_INCLUDE_PATH=/path/to/webrtc/src
export LIBWEBRTC_BINARY_PATH=/path/to/webrtc/src/out/m84/obj
# bindgen uses clang, so if your cross-compiler is gnu you need to add some include dirs
export BINDGEN_EXTRA_CLANG_ARGS='-I/usr/aarch64-linux-gnu/include/c++/11.1.0/aarch64-linux-gnu'
cargo build --target=aarch64-unknown-linux-gnu
```

## Sample
The `echo` example will produce a dummy video feed and a silent audio feed, and print consumed data.
```
cargo run --example echo -- --signal-addr <signal-addr> --token <token>
```

## Troubleshooting
### WebRTC check fail `0 == adm->Init()`
```
# Fatal error in: ../../media/engine/adm_helpers.cc, line 39
# last system error: 88
# Check failed: 0 == adm->Init() (0 vs. -1)
```
Install an audio server like PulseAudio.

### `SIGSEGV` in `pthread` when using TLS
Probably because we also link `pthread` as part of WebRTC and there is a conflict.

Try using `rustls` instead of `native-tls`. If that doesn't work then just turn TLS off.