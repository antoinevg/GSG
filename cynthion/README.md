---

## Step 0: Set up environment

### Check out repo

    git clone https://github.com/antoinevg/gsg.git gsg.git


### Script dependencies

If you don't have these installed already you'll probably want to:

    # debian
    apt install curl expect picocom zsh

    # macos
    brew install curl expect picocom zsh


### Python

If you'd like to use `pyenv` to manage your Python environment you can install it with:

    # macos
    brew install pyenv

    # other
    curl https://pyenv.run | bash

You'll also need these to build Python versions:

    # debian
    apt install build-essential libssl-dev zlib1g-dev libbz2-dev \
        libreadline-dev libsqlite3-dev libncursesw5-dev xz-utils \
        tk-dev libxml2-dev libxmlsec1-dev libffi-dev liblzma-dev

    # macos
    brew install openssl readline sqlite3 xz zlib tcl-tk

Finally, you can setup an environment with:

    # install python
    pyenv install 3.11

    # create a new virtual environment
    pyenv virtualenv 3.11 gsg-cynthion

    # enable virtual environment for gsg.git/cynthion/ directory
    cd gsg.git/cynthion/
    pyenv local gsg-cynthion

    # upgrade pip to latest
    python -m pip install --upgrade pip


### Yosys Toolchain

Grab and install the latest toolchain from:

    https://github.com/YosysHQ/oss-cad-suite-build/releases/latest

Remember to mollify Gatekeeper if you're on macOS:

    oss-cad-suite/activate

Enable environment with:

    source <path-to>/oss-cad-suite/environment


### Rust

If you'd like to use `rustup` to manage your Rust environment you can install it with:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Install riscv target support:

    #rustup target add riscv32imac-unknown-none-elf
    rustup target add riscv32imc-unknown-none-elf  # use imc until dcache issues are resolved

    rustup component add llvm-tools-preview
    cargo install cargo-binutils

Optional: only needed if you want to use rust nightly

    rustup target add riscv32imac-unknown-none-elf --toolchain nightly
    rustup component add llvm-tools-preview --toolchain nightly
    cargo +nightly install cargo-binutils


### RiscV GNU Toolchain

This is needed to build litex-bios and any of the C examples:

    # macOS - https://github.com/riscv-software-src/homebrew-riscv
    brew tap riscv-software-src/riscv
    brew install riscv-gnu-toolchain

    # debian
    apt install gcc-riscv64-unknown-elf

If we can get rid of litex-bios it'll only be needed to build the C examples.


---

## Step 1: Build the Cynthion SoC gateware

The Cynthion SoC gateware can be found in the [`gsg.git/cynthion/lunasoc/`](lunasoc/) directory.

### 0. Activate Yosys

    source <path-to>/oss-cad-suite/environment

### 1. Install python requirements

    cd gsg.git/cynthion/
    python -m pip install -r requirements.txt

### 2. Test environment setup

    cd gsg.git/cynthion/lunasoc/
    python blinky_verilog.py

If all goes well your cynthion should start counting in binary on the fpga led's.

### 3. Build soc gateware

Build the bitstream with:

    python top_vexriscv.py

You can load the bitstream with `apollo` using:

    apollo configure build/top.bit

Finally, you can check if the soc is working with something like:

    # ubuntu
    picocom --imap lfcrlf -b 115200 /dev/ttyACM0

    # macos
    picocom --imap lfcrlf -b 115200 /dev/cu.usbserial-D00137

Hit enter after connecting and you should see a prompt like:

    BIOS>

Note: there are two variations of the Cynthion SoC

* `top_minerva.py` - built on [Minerva](https://github.com/minerva-cpu/minerva)
  - supports the `rv32i` or `rv32im` isa
* `top_vexriscv.py` - built on [VexRiscv](https://github.com/SpinalHDL/VexRiscv)
  - supports the `rv32imac` isa


---

## Step 2: Build the Cynthion SoC firmware

The Cynthion SoC firmware crate can be found in the [`gsg.git/cynthion/cynthion/`](cynthion/) directory.

### 0. Check UART configuration in `flash.sh`

Edit `gsg.git/cynthion/cynthion/.cargo/flash.sh` and make sure the `UART` variable is pointing at the right device file.


### 1. Try running a simple example

    cargo run --release --bin hello

You should see a chase sequence on the fpga led's and the console should be outputting something like:

    INFO    Peripherals initialized, entering main loop.
    INFO    left: 3
    DEBUG   right: 7
    INFO    left: 11
    ...


### 2. Try running the USB bulk speed test example

Make sure the USB host and sideband ports are both connected to the machine you will be performing the test on.

In one terminal, build and run the speed test firmware:

    cargo run --release --bin bulk_speed_test

In another terminal, run the host-side speed test script:

    python scripts/bulk_speed_test.py


---

## Step 3: Try running the moondancer-info script

In one terminal, build and run the moondancer firmware:

    cargo run --release --bin moondancer

In another terminal, run:

    python scripts/moondancer-info.py

You should see something like:

    Found a GreatFET One!
      Board ID: 0
      Firmware version: v2023.0.1
      Part ID: a0000a30604f5e
      Serial number: 000057cc67e6306f5357


---

## Step 4: Run a Facedancer example

In one terminal, build and run the moondancer firmware:

    cargo run --release --bin moondancer

In another terminal, run:

    python scripts/facedancer-ftdi-echo.py

In a third terminal, run:

    picocom --imap lfcrlf -b 115200 /dev/ttyUSB0


---

## Uninstall

### Python Environment

    pip uninstall -y -r <(pip freeze)
    pyenv uninstall 3.11/envs/gsg-cynthion
    pyenv uninstall 3.11
