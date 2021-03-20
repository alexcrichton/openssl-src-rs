# openssl-src

This crate contains the logic to build OpenSSL and is intended to be consumed by
the `openssl-sys` crate. You likely in theory aren't interacting with this too
much!

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Windows MSVC Assembly
Building the `windows-msvc` targets on Windows, the build process will
automatically detect whether [nasm](https://www.nasm.us/) is installed in PATH.
The assembly language routines will be enabled if `nasm.exe` is found in PATH (in
other words, the `no-asm` option will NOT be configured).  
You can disable the this by set the `OPENSSL_RUST_NO_NASM` environment variable to
a non-zero value. This environment variable does not take effects on non-windows
platforms.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in openssl-src by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
