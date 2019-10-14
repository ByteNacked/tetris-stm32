use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufWriter, Write, BufReader, BufRead, Read};
use std::path::Path;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    let path = Path::new("test.txt");
    let mut file = BufReader::new(File::open(&path).unwrap());
     for line in file.lines() {
        println!("{}", line.unwrap());
    }


    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=memory.x");
}
