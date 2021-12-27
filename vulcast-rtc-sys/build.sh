#!/bin/bash
# build a compile db for clangd

set -euo pipefail

cmake . -Bbuild \
	-DCMAKE_EXPORT_COMPILE_COMMANDS=1 \
	-DTARGET="x86_64-unknown-linux-gnu" \
	-DLIBWEBRTC_INCLUDE_PATH:PATH="/mnt/scratch/vulcast-rtc/vulcast-rtc-sys/deps/libwebrtc/include" \
	-DLIBWEBRTC_BINARY_PATH:PATH="/mnt/scratch/vulcast-rtc/vulcast-rtc-sys/deps/libwebrtc/lib/x86_64-unknown-linux-gnu" 

cmake --build build
