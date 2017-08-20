extern crate openssl_src;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let artifacts = openssl_src::Build::new().build();
    artifacts.print_cargo_metadata();

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out_dir.join("include"))
        .unwrap()
        .write_all(artifacts.include_dir().to_str().unwrap().as_bytes())
        .unwrap();
    File::create(out_dir.join("lib"))
        .unwrap()
        .write_all(artifacts.lib_dir().to_str().unwrap().as_bytes())
        .unwrap();
    File::create(out_dir.join("target"))
        .unwrap()
        .write_all(env::var("TARGET").unwrap().as_bytes())
        .unwrap();
    File::create(out_dir.join("openssl-src-version"))
        .unwrap()
        .write_all(openssl_src::version().as_bytes())
        .unwrap();
}
