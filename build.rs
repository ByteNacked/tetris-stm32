use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufWriter, Write, BufReader, BufRead, Read};
use std::path::Path;
use std::fs::read_to_string;
use std::fs::write;


fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=memory.x");
}
