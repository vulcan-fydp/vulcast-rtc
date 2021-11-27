# vulcast-rtc
Native Rust library for realtime communication with Relay, handling:
- Controller input over WebRTC DataChannel
- Audio/video encoding over WebRTC transport (post-MVP)

## What
- C++ library `vulcast-rtc` encapsulating a WebRTC client using MediaSoup
- Rust sys crate `vulcast-rtc-sys` exposing FFI declarations from `vulcast-rtc`
- Rust crate `vulcast-rtc` exposing high-level abstractions for `vulcast-rtc-sys`

## Build 
### Native
I don't recommend building natively on the Raspberry Pi itself. I tried myself
and it never finished.
```
cargo build
```

### Cross-compiling
Only cross-compilation for armv7-unknown-linux-gnueabihf is supported.

1. Install cross-compiler toolchain for arm-linux-gnueabihf  

You **must** use a toolchain that has a relatively old version of glibc (<2.32). 

e.g. You can use 
 Raspberry Pi GCC Cross-Compiler Toolchains (Buster) gcc 10.3.0 from 
https://github.com/abhiTronix/raspberry-pi-cross-compilers. Just extract it somewhere for now.

2. Install Rust cross-compiler toolchain for armv7-unknown-linux-gnueabihf
```bash
# install rust cross toolchain
rustup target add armv7-unknown-linux-gnueabihf
```

3. Build a project using `vulcast-rtc`

Replace `/path/to/cross-pi-gcc-10.3.0-2` with where you extracted the toolchain to.
```bash
# add cross-compiler toolchain to PATH
export PATH=/mnt/scratch/cross-pi-gcc-10.3.0-2/bin:$PATH
# set linker for cargo to use for armv7-unknown-linux-gnueabihf target
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER='/path/to/cross-pi-gcc-10.3.0-2/bin/arm-linux-gnueabihf-gcc'
# set paths to C++ standard library and platform defines for bindgen
export BINDGEN_EXTRA_CLANG_ARGS='-I/path/to/cross-pi-gcc-10.3.0-2/arm-linux-gnueabihf/include/c++/10.3.0 \
	 -I/path/to/cross-pi-gcc-10.3.0-2/arm-linux-gnueabihf/include/c++/10.3.0/arm-linux-gnueabihf/'
cargo build --example echo --target=armv7-unknown-linux-gnueabihf
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

### version `GLIBCXX_3.4.29' not found
Your cross-compiler toolchain uses an incompatible version of glibc. Try using one of these: 
https://github.com/abhiTronix/raspberry-pi-cross-compilers