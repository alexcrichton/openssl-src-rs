extern crate libc;

use libc::c_ulong;

extern {
    pub fn OpenSSL_version_num() -> c_ulong;
}

#[test]
fn version_works() {
    unsafe {
        println!("{:#x}", OpenSSL_version_num());
        assert!(OpenSSL_version_num() > 0);
    }
}
