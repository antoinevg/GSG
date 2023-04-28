---

## Step 0: Set up environment

### General Dependencies

    # rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    # pyenv
    curl https://pyenv.run | bash


### Yosys Toolchain

Grab the latest toolchain from:

    https://github.com/YosysHQ/oss-cad-suite-build/releases/latest

Copy it into the `toolchain/` directory and:

    cd toolchain/
    tar xzf oss-cad-suite-*.tgz

    # Mollify gatekeeper if you're on macOS
    oss-cad-suite/activate

Enable environment with:

    source <path-to>/oss-cad-suite/environment


### Rust Dependencies

    rustup target add riscv32i-unknown-none-elf
    rustup component add llvm-tools-preview
    cargo install cargo-binutils


### Python Environment

    # install python
    pyenv install 3.11.3

    # create a new virtual environment
    pyenv virtualenv 3.11.3 gsg-cynthion

    # enable virtual environment for gsg.git/cynthion/ directory
    cd gsg.git/cynthion/
    pyenv local gsg-cynthion

    # upgrade pip to latest
    python -m pip install --upgrade pip

    # install package: luna
    python -m pip install "luna @ git+https://github.com/greatscottgadgets/luna@main"

    # because: https://github.com/python-poetry/poetry/issues/3514
    cd gsg.git/cynthion/
    python -m pip install -r requirements.txt


### RiscV toolchain

This is needed to build litex-bios:

    # macOS
    # riscv-gnu-toolchain - https://github.com/riscv-software-src/homebrew-riscv
    brew tap riscv-software-src/riscv
    brew install riscv-gnu-toolchain

    # debian/ubuntu
    TODO

If we can get rid of litex-bios it'll only be needed to build the C examples.


### Optional

To use rust nightly:

    rustup target add riscv32i-unknown-none-elf --toolchain nightly
    rustup component add llvm-tools-preview --toolchain nightly
    cargo +nightly install cargo-binutils

To mess around with C examples:



---

## Step 1: Build the Cynthion SoC gateware

The Cynthion SoC gateware can be found in the [`lunasoc/`](lunasoc/) directory:

    cd lunasoc/

### 0. Activate Yosys

    source <path-to>/oss-cad-suite/environment

### 1. Test environment setup

    python blinky_verilog.py

If all goes well your cynthion should show start counting in binary on the fpga led's.

### 2. Install requirements

    python -m pip install -r requirements.txt

### 3. Build soc gateware

    make top


Note: there are two variations of the Cynthion SoC

* `top_minerva.py` - built on [Minerva](https://github.com/minerva-cpu/minerva)
  - supports the `rv32i` or `rv32im` isa
* `top_vexriscv.py` - built on [VexRiscv](https://github.com/SpinalHDL/VexRiscv)
  - supports the `rv32imac` isa



---

## Uninstall

### Python Environment

    pip uninstall -y -r <(pip freeze)
    pyenv uninstall 3.11.3/envs/gsg-cynthion
    pyenv uninstall 3.11.3




========================================================================================================

---

## Environments


### Yosys Toolchain



### Vanilla Amaranth Environment

    pyenv activate gsg-amaranth

    pip install --upgrade 'amaranth[builtin-yosys]'

    # prefer
    cd toolchain/
    git clone https://github.com/amaranth-lang/amaranth.git amaranth.git
    cd amaranth.git
    python setup.py install

    cd toolchain/
    git clone https://github.com/amaranth-lang/amaranth-boards.git amaranth-boards.git
    cd amaranth-boards.git
    python setup.py install
    pip install markupsafe==2.0.1     # fix

    cd toolchain/
    git clone https://github.com/lambdaconcept/lambdasoc.git lambdasoc.git

    # test installation
    python -m amaranth_boards.icestick
    python -m amaranth_boards.ulx3s 85F


### Vanilla `luna.git` Environment

    See [WORKAROUNDS.md](WORKAROUNDS.md) for issues.

    pyenv activate gsg-cynthion

    cd toolchain/
    # git clone https://github.com/greatscottgadgets/luna.git luna.git
    git clone git@github.com:antoinevg/luna.git luna.git

    # Install luna using poetry
    pip3 install poetry
    rm poetry.lock
    poetry install

    # Install using requirements.txt (preferred ?)
    pip install -r requirements.txt
    python setup.py install

    # test installation - apollo
    ~/.pyenv/versions/gsg-cynthion/bin/apollo info

    # test installation - self test
    poetry run applets/interactive-test.py

    # test installation - blinky
    cd examples/blinky
    python blinky.py



---

## Run Luna Examples

    cd toolchain/luna.git/examples/soc/bios
    LUNA_PLATFORM=luna.gateware.platform.ulx3s:ULX3S_85F_Platform AR=riscv64-unknown-elf-ar make clean all


---

## `hello-r04/`

    python hello-r04/blinky.py


---

## `hello-uart/`

    picocom --imap lfcrlf -b 115200 /dev/cu.usbmodem22301


## References

* [Luna Documentation - Getting Started](https://luna.readthedocs.io/en/latest/getting_started.html)
