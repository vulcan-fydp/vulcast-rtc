#!/bin/bash
# build a compile db for clangd

set -euo pipefail

cmake . -Bbuild \
	-DCMAKE_EXPORT_COMPILE_COMMANDS=1 \
	-DLIBWEBRTC_INCLUDE_PATH:PATH="../deps/libwebrtc/include" \
	-DLIBWEBRTC_BINARY_PATH:PATH="../deps/libwebrtc/lib/x86_64-unknown-linux-gnu" 

cmake --build build
