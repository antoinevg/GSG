SHELL := /bin/zsh

#UART := /dev/cu.usbmodem22301
UART := /dev/cu.usbmodem22401
#UART := /dev/cu.usbmodem22101

YOSYS_BIN := ~/GreatScott/gsg.git/luna/toolchain/oss-cad-suite/bin
AR=riscv64-unknown-elf-ar
CC=riscv64-unknown-elf-gcc
OBJCOPY=riscv64-unknown-elf-objcopy

CFLAGS	= -march=rv32i -mabi=ilp32 -g -Os -I./build/genc/
LDFLAGS = -Tsoc.ld -Triscv_application.ld  -nostdlib -L./build/genc/ -L./examples/

LUNASOC_PAC := ../lunasoc-pac

# - gateware ------------------------------------------------------------------

top:
	PATH=$(PATH):$(YOSYS_BIN) AR=$(AR) python top.py

load:
	python configure_ecp5.py build/top.bit

load2:
	PATH=$(PATH):$(YOSYS_BIN) openFPGALoader --board ulx3s build/top.bit


# - cgen ----------------------------------------------------------------------

build/examples/blinky.elf: #soc.ld resources.h
	mkdir -p build/examples/
	$(CC) $(CFLAGS) $(LDFLAGS) examples/blinky.S examples/blinky.c -o $@

build/examples/blinky.bin: build/examples/blinky.elf
	$(OBJCOPY) -O binary $< $@

blinky: build/examples/blinky.bin
	lxterm --kernel ./build/examples/blinky.bin --kernel-adr 0x40000000 --speed 115200 $(UART)

build/examples/timer.elf: #soc.ld resources.h
	mkdir -p build/examples/
	$(CC) $(CFLAGS) $(LDFLAGS) examples/timer.S examples/timer.c -o $@

build/examples/timer.bin: build/examples/timer.elf
	$(OBJCOPY) -O binary $< $@

timer: build/examples/timer.bin
	lxterm --kernel ./build/examples/timer.bin --kernel-adr 0x40000000 --speed 115200 $(UART)

build/examples/eptri.elf: #soc.ld resources.h
	mkdir -p build/examples/
	$(CC) $(CFLAGS) $(LDFLAGS) examples/eptri.S examples/eptri.c -o $@

build/examples/eptri.bin: build/examples/eptri.elf
	$(OBJCOPY) -O binary $< $@

eptri: build/examples/eptri.bin
	lxterm --kernel ./build/examples/eptri.bin --kernel-adr 0x40000000 --speed 115200 $(UART)


# - gensvd --------------------------------------------------------------------

rs-blinky:
	@echo "TODO"


# - helpers -------------------------------------------------------------------

console:
	picocom --imap lfcrlf -b 115200 $(UART)

console2:
	picocom --imap lfcrlf -b 115200 /dev/cu.usbserial-D00137

clean:
	-rm -rf build/ resources.h soc.ld
