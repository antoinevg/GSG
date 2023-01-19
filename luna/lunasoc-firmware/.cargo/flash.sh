#!/usr/bin/env zsh

# configuration
UART=/dev/cu.usbmodem22401
BASE_MEM=0x40000000

# create bin file
NAME=$(basename $1)
cargo objcopy --release --bin $NAME -- -Obinary $1.bin

# flash to soc
echo "Flashing: $1.bin"
lxterm --kernel $1.bin --kernel-adr $BASE_MEM --speed 115200 $UART
