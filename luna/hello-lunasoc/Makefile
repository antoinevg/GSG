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

LUNASOC_PAC := ../lunasoc-pac

# - gateware ------------------------------------------------------------------

top:
	PATH=$(PATH):$(YOSYS_BIN) AR=$(AR) python top.py

load:
	python configure_ecp5.py build/top.bit

load2:
	PATH=$(PATH):$(YOSYS_BIN) openFPGALoader --board ulx3s build/top.bit


# - cgen ----------------------------------------------------------------------

blinky.elf: #soc.ld resources.h
	$(CC) $(CFLAGS) $(LDFLAGS) start.S blinky.c -o $@

blinky.bin: blinky.elf
	$(OBJCOPY) -O binary $< $@

flash: blinky.bin
	lxterm --kernel ./blinky.bin --kernel-adr 0x40000000 --speed 115200 $(UART)

timer.elf: timer.c timer.S soc.ld resources.h
	$(CC) $(CFLAGS) $(LDFLAGS) timer.S timer.c -o $@

timer.bin: timer.elf
	$(OBJCOPY) -O binary $< $@

flash_timer: timer.bin
	lxterm --kernel ./timer.bin --kernel-adr 0x40000000 --speed 115200 $(UART)



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
