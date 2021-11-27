#!/bin/bash

export PATH=/mnt/scratch/cross-pi-gcc-10.3.0-2/bin:$PATH
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER='/mnt/scratch/cross-pi-gcc-10.3.0-2/bin/arm-linux-gnueabihf-gcc'
export BINDGEN_EXTRA_CLANG_ARGS='-I/mnt/scratch/cross-pi-gcc-10.3.0-2/arm-linux-gnueabihf/include/c++/10.3.0 \
	 -I/mnt/scratch/cross-pi-gcc-10.3.0-2/arm-linux-gnueabihf/include/c++/10.3.0/arm-linux-gnueabihf/'