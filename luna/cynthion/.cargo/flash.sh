#!/usr/bin/env zsh

# configuration
UART=/dev/cu.usbmodem22401
BASE_MEM=0x10000000
BITSTREAM=../lunasoc/build/top.bit

# create bin file
NAME=$(basename $1)
cargo objcopy --release --bin $NAME -- -Obinary $1.bin

# lxterm command
LXTERM="lxterm --kernel $1.bin --kernel-adr $BASE_MEM --speed 115200 $UART"

# configure cynthion fpga with soc bitstream
echo "Configuring fpga: $BITSTREAM"
apollo configure $BITSTREAM 2>/dev/null

# flash firmware to soc
echo "Flashing: $1.bin"
expect -c "spawn $LXTERM; send \nserialboot\n; interact"
