extern crate openssl_src;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let (lib_dir, include_dir) = openssl_src::Build::new().build();
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ssl");
    println!("cargo:rustc-link-lib=static=crypto");
    println!("cargo:include={}", include_dir.display());
}
