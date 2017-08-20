extern crate libc;

use libc::c_ulong;

extern {
    pub fn OpenSSL_version_num() -> c_ulong;
}
