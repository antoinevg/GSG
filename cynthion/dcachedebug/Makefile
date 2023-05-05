# - helpful -------------------------------------------------------------------

size:
	cargo size --release --bin firmware -- -A
	cargo size --release --bin firmware -- -B

stack:
	@# cargo size --release --bin firmware -- -A | grep stack # 12504
	cargo size --release --bin diag -- -A | grep stack # 41576 / 40240

headers:
	cargo readobj --release --bin firmware -- --file-headers

disasm:
	@# iscv64-unknown-elf-objdump --disassemble diag
	cargo objdump --release --bin firmware -- --disassemble	 --no-show-raw-insn --print-imm-hex

disasm-src:
	cargo objdump --release --bin firmware -- --disassemble	 --no-show-raw-insn --print-imm-hex --source

syms:
	cargo objdump --release --bin firmware -- --syms

rodata:
	cargo objdump --release --bin firmware -- --full-contents --section .rodata

data:
	cargo objdump --release --bin firmware -- --full-contents --section .data

nm:
	@# cargo nm --release --bin firmware -- --debug-syms --format=bsd --print-size --size-sort --radix=d
	cargo nm --release --bin firmware -- --size-sort --radix=d | grep -v Lanon | grep -v 00000000

elf-size-analyze:
	@# https://github.com/jedrzejboczar/elf-size-analyze
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable --alternating-colors --max-width 240 \
		--ram --rom --print-sections \
		../target/riscv32i-unknown-none-elf/release/firmware

elf-ram:
	@# https://github.com/jedrzejboczar/elf-size-analyze
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable --alternating-colors --max-width 240 \
		--ram \
		../target/riscv32i-unknown-none-elf/release/firmware

elf-rom:
	@# https://github.com/jedrzejboczar/elf-size-analyze
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable --alternating-colors --max-width 240 \
		--rom \
		../target/riscv32i-unknown-none-elf/release/firmware

elf-sections:
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable \
		--print-sections \
		../target/riscv32i-unknown-none-elf/release/firmware

elf-text:
	elf-size-analyze -t riscv64-unknown-elf- \
		--human-readable \
		-S2 \
		../target/riscv32i-unknown-none-elf/release/firmware


# .text	 + 2.4k
# .stack - 2.4k
