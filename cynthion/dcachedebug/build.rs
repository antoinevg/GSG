#![allow(dead_code, unused_imports, unused_mut, unused_variables)]

use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // TODO Tracking Issue: https://github.com/rust-lang/rust/issues/94039
    let target = match rustc_target() {
        Some(target) => target,
        None => return,
    };
    if target_has_atomic(&target) {
        println!("cargo:rustc-cfg=target_has_atomic");
    }

    // device.x
    /*File::create(out_dir.join("device.x"))
        .unwrap()
        .write_all(include_bytes!("device.x"))
        .unwrap();
    println!("cargo:rerun-if-changed=device.x");*/

    // memory.x
    println!("cargo:rerun-if-changed=memory.x");

    // asm.S / link.x
    //link_bare();
    //link_riscvrt();
    link_dcachedebug();
    //link_litex();

    // build.rs
    println!("cargo:rerun-if-changed=build.rs");
}

// - link.x -------------------------------------------------------------------

fn link_bare() {
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // link.x
    fs::write(
        out_dir.join("link.x"),
        include_bytes!("link-bare.x"),
    ).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=link-bare.x");
}

fn link_riscvrt() {
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // asm.S
    cc::Build::new()
        .file("asm.S")
        .compile("my_asm");
    println!("cargo:rerun-if-changed=asm.S");

    // link.x
    fs::write(
        out_dir.join("link.x"),
        include_bytes!("link-riscvrt.x"),
    ).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=link-riscvrt.x");
}

fn link_dcachedebug() {
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // asm.S
    cc::Build::new()
        .file("asm-dcachedebug.S")
        .compile("my_asm");
    println!("cargo:rerun-if-changed=asm-dcachedebug.S");

    // link.x
    fs::write(
        out_dir.join("link.x"),
        include_bytes!("link-dcachedebug.x"),
    ).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=link-dcachedebug.x");
}

fn link_litex() {
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // asm.S
    cc::Build::new()
        .file("asm-litex.S")
        .compile("my_asm");
    println!("cargo:rerun-if-changed=asm-litex.S");

    // link.x
    fs::write(
        out_dir.join("link.x"),
        include_bytes!("link-litex.x"),
    ).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=link-litex.x");
}

// - target -------------------------------------------------------------------

fn rustc_target() -> Option<String> {
    env::var("TARGET").ok()
}

fn target_has_atomic(target: &str) -> bool {
    match target {
        "riscv32imac-unknown-none-elf" => true,
        "riscv32i-unknown-none-elf" => false,
        _ => false,
    }
}
