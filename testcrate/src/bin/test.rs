extern crate testcrate;

fn main() {
    unsafe {
        println!("{:#x}", testcrate::OpenSSL_version_num());
    }
}
