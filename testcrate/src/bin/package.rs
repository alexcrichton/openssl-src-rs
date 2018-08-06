extern crate tar;
extern crate flate2;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use tar::Builder;

fn main() {
    let name = format!("openssl-src-bins-{}-{}",
                       read("openssl-src-version"),
                       read("target"));
    let out = File::create(format!("{}.tar.gz", name)).unwrap();
    let out = flate2::write::GzEncoder::new(out, flate2::Compression::best());
    let mut builder = Builder::new(out);
    builder.append_dir_all(format!("{}/include", name), read("include")).unwrap();
    builder.append_dir_all(format!("{}/lib", name), read("lib")).unwrap();
    builder.finish().unwrap();
}

fn read(path: &str) -> String {
    let out_dir = Path::new(env!("OUT_DIR"));
    let mut ret = String::new();
    File::open(out_dir.join(path)).unwrap().read_to_string(&mut ret).unwrap();
    return ret
}
