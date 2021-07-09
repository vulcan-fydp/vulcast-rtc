#!/bin/bash
# build cmake portion only

set -euo pipefail

source env.sh

cmake . -Bbuild \
	-DCMAKE_EXPORT_COMPILE_COMMANDS=1 \
	-DLIBWEBRTC_INCLUDE_PATH:PATH="$LIBWEBRTC_INCLUDE_PATH" \
	-DLIBWEBRTC_BINARY_PATH:PATH="$LIBWEBRTC_BINARY_PATH" 

cmake --build build 
