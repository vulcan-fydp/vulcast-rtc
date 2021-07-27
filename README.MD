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
1. Install cross-compiler toolchain for arm-linux-gnueabihf
```bash
# install rust cross toolchain
rustup target add armv7-unknown-linux-gnueabihf

# install system cross linker (e.g. arm-linux-gnueabihf-gcc for Arch Linux)
```

Two host triples are supported, `armv7-unknown-linux-gnueabihf` and `x86_64-unknown-linux-gnu`.

1. Build `vulcast-rtc`
```bash
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
export BINDGEN_EXTRA_CLANG_ARGS='-I/usr/arm-linux-gnueabihf/include/c++/11.1.0/arm-linux-gnueabihf'
cargo build --target=armv7-unknown-linux-gnueabihf
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
