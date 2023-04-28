Rust Dependencies:

    rustup component add llvm-tools --toolchain nightly
    rustup component add rust-src --toolchain nightly-aarch64-apple-darwin
    cargo install cargo-binutils

Run:

    cargo run --release
