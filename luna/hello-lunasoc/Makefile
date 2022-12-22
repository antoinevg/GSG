SHELL := /bin/zsh

UART := /dev/cu.usbmodem22301
#UART := /dev/cu.usbmodem22401
#UART := /dev/cu.usbmodem22101

YOSYS_BIN := ~/GreatScott/gsg.git/luna/toolchain/oss-cad-suite/bin
AR=riscv64-unknown-elf-ar
CC=riscv64-unknown-elf-gcc
OBJCOPY=riscv64-unknown-elf-objcopy

CFLAGS	= -march=rv32im -mabi=ilp32 -g -Os
LDFLAGS = -Tsoc.ld -Triscv_application.ld  -nostdlib


# - gateware ------------------------------------------------------------------

top:
	PATH=$(PATH):$(YOSYS_BIN) AR=$(AR) python top.py

load:
	python configure_ecp5.py build/top.bit

load2:
	PATH=$(PATH):$(YOSYS_BIN) openFPGALoader --board ulx3s build/top.bit


# - cbindgen ------------------------------------------------------------------

blinky.elf: #soc.ld resources.h
	$(CC) $(CFLAGS) $(LDFLAGS) start.S blinky.c -o $@

blinky.bin: blinky.elf
	$(OBJCOPY) -O binary $< $@

flash: blinky.bin
	lxterm --kernel ./blinky.bin --kernel-adr 0x40000000 --speed 115200 $(UART)


# - svdgen --------------------------------------------------------------------

svd:
	cp build/soc/soc.svd lunasoc-pac/svd/lunasoc.svd
	rm -rf lunasoc-pac/src
	mkdir lunasoc-pac/src
	svd2rust -i lunasoc-pac/svd/lunasoc.svd -o build --target riscv
	mv build/lib.rs lunasoc-pac/src
	mv build/build.rs lunasoc-pac/
	mv build/device.x lunasoc-pac/
	form -i lunasoc-pac/src/lib.rs -o lunasoc-pac/src/
	cd lunasoc-pac && cargo fmt -- --emit files


# - helpers -------------------------------------------------------------------

console:
	picocom --imap lfcrlf -b 115200 $(UART)

console2:
	picocom --imap lfcrlf -b 115200 /dev/cu.usbserial-D00137

clean:
	-rm -rf build/
