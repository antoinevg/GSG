#
# This file is part of LUNA.
#

TARGET		= blinky
BAUDRATE	= 115200
SERIALPORT ?= /dev/ttyACM0
UART := /dev/cu.usbmodem22301

CROSS  ?= riscv64-unknown-elf-

AR		= $(CROSS)ar
CC		= $(CROSS)gcc
OBJCOPY = $(CROSS)objcopy

CFLAGS	= -march=rv32i -mabi=ilp32 -g -Os
LDFLAGS = -Tsoc.ld -Triscv_application.ld  -nostdlib

SOC = bios_example.py
SOURCES = \
	start.S \
	$(TARGET).c


# By default, build our binary.
all: $(TARGET).bin


#
# Generated files.
#

soc.ld: $(SOC)
	python ./$(SOC) --generate-ld-script > $@

resources.h: $(SOC)
	python ./$(SOC) --generate-c-header > $@


#
# Firmware binary.
#

$(TARGET).elf: $(SOURCES) soc.ld resources.h
	$(CC) $(CFLAGS) $(LDFLAGS) $(SOURCES) -o $@

$(TARGET).bin: $(TARGET).elf
	$(OBJCOPY) -O binary $< $@


#
# Virtual/command targets.
#

.PHONY: clean program

clean:
	rm -rf $(TARGET).elf $(TARGET).bin soc.ld resources.h build


# Load our SoC onto the FPGA...
configure: $(SOC)
	AR=$(AR) python ./$(SOC) --keep-files

# Flash the FPGA's ROM with our SoC.
flash-soc: $(SOC)
	python ./$(SOC) --flash

# Program the SoC with our application.
program: $(TARGET).bin
	lxterm --kernel ./blinky.bin --kernel-adr 0x00008000 --speed 115200 $(UART)

	@#echo -e "\nserialboot" | script -q /dev/null -c \
	@#	"flterm --speed $(BAUDRATE) --kernel $< --kernel-addr $(shell python ./$(SOC) --get-fw-address) $(SERIALPORT)"

# console
console:
	picocom --imap lfcrlf -b 115200 $(UART)
