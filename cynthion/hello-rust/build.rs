use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // memory.x
    println!("cargo:rerun-if-changed=memory.x");

    // device.x
    File::create(out_dir.join("device.x"))
        .unwrap()
        .write_all(include_bytes!("device.x"))
        .unwrap();
    println!("cargo:rerun-if-changed=device.x");

    // link.rs
    println!("cargo:rustc-link-search={}", out_dir.display());

    // build.rs
    println!("cargo:rerun-if-changed=build.rs");
}
