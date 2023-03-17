## Read

* [Luna Documentation - Getting Started](https://luna.readthedocs.io/en/latest/getting_started.html)


---

## Setup

### OS Dependencies

    # gtkwave
    brew install gtkwave

    # rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    # pyenv
    curl https://pyenv.run | bash

    # riscv-gnu-toolchain - https://github.com/riscv-software-src/homebrew-riscv
    brew tap riscv-software-src/riscv
    brew install riscv-gnu-toolchain

### Rust Dependencies

    rustup target add riscv32i-unknown-none-elf
    rustup target add riscv32i-unknown-none-elf --toolchain nightly

    rustup component add llvm-tools-preview
    rustup component add llvm-tools-preview --toolchain nightly

    cargo install cargo-binutils
    cargo +nightly install cargo-binutils

### Python Environments

    # x86_64/rosetta
    pyenv install pypy3.9-7.3.9
    pyenv virtualenv pypy3.9-7.3.9 gsg-amaranth
    pyenv virtualenv pypy3.9-7.3.9 gsg-luna
    pyenv local gsg-amaranth
    pyenv local gsg-luna

    # arm64
    pyenv install 3.11.1
    pyenv virtualenv 3.11.1 gsg-amaranth
    pyenv virtualenv 3.11.1 gsg-luna
    pyenv local gsg-amaranth
    pyenv local gsg-luna

    # upgrade pip to latest
    pip install --upgrade pip

    # install wheel
    pip install wheel

---

## Environments


### Yosys Toolchain

Links:

* https://github.com/YosysHQ/oss-cad-suite-build

Grab the latest toolchain from:

    https://github.com/YosysHQ/oss-cad-suite-build/releases/latest

Copy it into the `toolchain/` directory and:

    cd toolchain/
    tar xzf oss-cad-suite-darwin-arm64-*.tgz

    # Mollify gatekeeper
    oss-cad-suite/activate

Enable environment with:

    source toolchain/oss-cad-suite/environment



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

    pyenv activate gsg-luna

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
    ~/.pyenv/versions/gsg-luna/bin/apollo info

    # test installation - self test
    poetry run applets/interactive-test.py

    # test installation - blinky
    cd examples/blinky
    python blinky.py


---

## Uninstall

    brew uninstall gtkwave

    pip uninstall -y -r <(pip freeze)

    pyenv uninstall 3.11.1/envs/gsg-amaranth
    pyenv uninstall 3.11.1/envs/gsg-luna
    pyenv uninstall 3.11.1



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
