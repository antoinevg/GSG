#!/usr/bin/env zsh

#set -ex

UART=/dev/cu.usbmodem22301

NAME=$(basename $1)

# Create bin file
cargo objcopy --release -- -Obinary $1.bin

echo "Flashing: $1.bin"

# Program vexriscv
lxterm --kernel $1.bin --kernel-adr 0x00008000 --speed 115200 $UART
