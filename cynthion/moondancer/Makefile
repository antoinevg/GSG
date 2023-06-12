# - helpful -------------------------------------------------------------------

TARGET=../target/riscv32imac-unknown-none-elf/release

# logging=off:	 44344
# logging=debug:  9028
BIN=moondancer

# start:			  25624
# 1 queue, no buffer: 46176
#BIN=bulk_speed_test

# mpmc: 34720
# spsc: 35272
#BIN=bulk_speed_test_spsc

# 16 endpoint buffer: 25620
# with 2 endpoint buffer: 32812
#BIN=bulk_speed_test_bbq

#
#BIN=smollest

bins:
	cargo build --release --bins

size:
	cargo size --release --bin $(BIN) -- -A
	cargo size --release --bin $(BIN) -- -B

stack:
	@# cargo size --release --bin $(BIN) -- -A | grep stack # 12504
	cargo size --release --bin $(BIN) -- -A | grep stack # 41576 / 40240

headers:
	cargo readobj --release --bin $(BIN) -- --file-headers

disasm:
	@# riscv64-unknown-elf-objdump --disassemble --demangle $(TARGET)/$(BIN)
	@# cargo objdump --release --bin $(BIN) -- --disassemble --no-show-raw-insn --print-imm-hex
	cargo objdump --release --bin $(BIN) -- --disassemble

disasm-src:
	riscv64-unknown-elf-objdump --disassemble --demangle --source $(TARGET)/$(BIN)
	@# cargo objdump --release --bin $(BIN) -- --disassemble --source --no-show-raw-insn --print-imm-hex
	@# cargo objdump --release --bin $(BIN) -- --disassemble  --source

syms:
	cargo objdump --release --bin $(BIN) -- --syms

rodata:
	cargo objdump --release --bin $(BIN) -- --full-contents --section .rodata

data:
	cargo objdump --release --bin $(BIN) -- --full-contents --section .data

nm:
	@# cargo nm --release --bin $(BIN) -- --debug-syms --format=bsd --print-size --size-sort --radix=d
	@# cargo nm --release --bin $(BIN) -- --size-sort --radix=d | grep -v Lanon | grep -v 00000000
	cargo nm --release --bin $(BIN) -- --size-sort | grep -v Lanon | grep -v 00000000

elf-size-analyze: bins
	@# https://github.com/jedrzejboczar/elf-size-analyze
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable --alternating-colors --max-width 240 \
		--ram --rom --print-sections \
		$(TARGET)/$(BIN)

elf-ram: bins
	@# https://github.com/jedrzejboczar/elf-size-analyze
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable --alternating-colors --max-width 240 \
		--ram \
		$(TARGET)/$(BIN)

elf-rom: bins
	@# https://github.com/jedrzejboczar/elf-size-analyze
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable --alternating-colors --max-width 240 \
		--rom \
		$(TARGET)/$(BIN)

elf-sections: bins
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable \
		--print-sections \
		$(TARGET)/$(BIN)

elf-text: bins
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable \
		-S2 \
		$(TARGET)/$(BIN)
