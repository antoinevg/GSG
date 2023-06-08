## Dependencies

Add one or more of:

    rustup target add riscv32i-unknown-none-elf
    rustup target add riscv32imc-unknown-none-elf
    rustup target add riscv32imac-unknown-none-elf

## Select softcore

LunaSoC and the MoonDancer firmware supports two different soft-cores:

* Minerva
  - Supports the rv32i instruction set
* VexRiscv
  - Supports the rv32imac instruction set

### Minerva

Build gateware in `lunasoc/`:

    make top_minerva
    make load

Edit `moondancer/Cargo.toml`:

    [features]
    default = [
        "minerva",
    ]

Edit `moondancer/.cargo/config.toml`:

    [build]
    target = "riscv32i-unknown-none-elf"


### VexRiscv

Build gateware in `lunasoc/`:

    make top_vexriscv
    make load

Edit `moondancer/Cargo.toml`:

    [features]
    default = [
        "vexriscv",
    ]

Edit `moondancer/.cargo/config.toml`:

    [build]
    target = "riscv32imac-unknown-none-elf"


---

## Examples

### `bulk_in_speed_test.rs`

```
TEST_DATA_SIZE     = 1 * 1024 * 1024 = 1 048 576
TEST_TRANSFER_SIZE = 16 * 1024       = 16 384

1.06496MB = 512 * 2080


# iterators
INFO    | bulk_in_speed_test| Running speed test...
INFO    | bulk_in_speed_test| Exchanged 1.06496MB total at 3.8956449586736497MB/s.

# real data - vexriscv
INFO    | bulk_in_speed_test| Running speed test...
INFO    | bulk_in_speed_test| Exchanged 1.06496MB total at 4.074605743611125MB/s.

# real data - minerva
INFO    | bulk_in_speed_test| Running speed test...
INFO    | bulk_in_speed_test| Exchanged 1.06496MB total at 3.0601644558657592MB/s.

# computed data - vexriscv
INFO    | bulk_in_speed_test| Running speed test...
INFO    | bulk_in_speed_test| Exchanged 1.06496MB total at 5.060653974345321MB/s.

# computed data - minerva
INFO    | bulk_in_speed_test| Running speed test...
INFO    | bulk_in_speed_test| Exchanged 1.06496MB total at 3.227189566564362MB/s.
```
