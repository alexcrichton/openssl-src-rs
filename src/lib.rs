extern crate gcc;

use std::env;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("openssl")
}

pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
    host: Option<String>,
    cross_sysroot: Option<PathBuf>,
}

impl Build {
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| {
                PathBuf::from(s).join("openssl-build")
            }),
            target: env::var("TARGET").ok(),
            host: env::var("HOST").ok(),
            cross_sysroot: None,
        }
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    pub fn host(&mut self, host: &str) -> &mut Build {
        self.host = Some(host.to_string());
        self
    }

    pub fn build(&mut self) -> (PathBuf, PathBuf) {
        let target = &self.target.as_ref().expect("TARGET dir not set")[..];
        let host = &self.host.as_ref().expect("HOST dir not set")[..];
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let build_dir = out_dir.join("build");
        let install_dir = out_dir.join("install");
        let lib_dir = install_dir.join("usr/local/lib");
        let include_dir = install_dir.join("usr/local/include");

        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).unwrap();
        }
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).unwrap();
        }

        let inner_dir = build_dir.join("src");
        fs::create_dir_all(&inner_dir).unwrap();
        cp_r(&source_dir(), &inner_dir);

        let mut configure = Command::new("perl");
        configure.arg("./Configure");
        configure
            .arg("no-dso")
            .arg("no-ssl3")
            .arg("no-comp")
            .arg("no-unit-test")
            .arg("no-zlib")
            .arg("no-zlib-dynamic")
            .arg("no-shared");

        let os = match target {
            "aarch64-unknown-linux-gnu" => "linux-aarch64",
            "arm-unknown-linux-gnueabi" => "linux-armv4",
            "arm-unknown-linux-gnueabihf" => "linux-armv4",
            "armv7-unknown-linux-gnueabihf" => "linux-armv4",
            "i686-apple-darwin" => "darwin-i386-cc",
            "i686-unknown-freebsd" => "BSD-x86-elf",
            "i686-unknown-linux-gnu" => "linux-elf",
            "i686-unknown-linux-musl" => "linux-elf",
            "mips-unknown-linux-gnu" => "linux-mips32",
            "mips64-unknown-linux-gnuabi64" => "linux64-mips64",
            "mips64el-unknown-linux-gnuabi64" => "linux64-mips64",
            "mipsel-unknown-linux-gnu" => "linux-mips32",
            "powerpc-unknown-linux-gnu" => "linux-ppc",
            "powerpc64-unknown-linux-gnu" => "linux-ppc64",
            "powerpc64le-unknown-linux-gnu" => "linux-ppc64le",
            "s390x-unknown-linux-gnu" => "linux64-s390x",
            "x86_64-apple-darwin" => "darwin64-x86_64-cc",
            "x86_64-unknown-freebsd" => "BSD-x86_64",
            "x86_64-unknown-linux-gnu" => "linux-x86_64",
            "x86_64-unknown-linux-musl" => "linux-x86_64",
            "x86_64-unknown-netbsd" => "BSD-x86_64",
            "x86_64-pc-windows-gnu" => "mingw64",
            "i686-pc-windows-gnu" => "mingw",
            "arm-linux-androideabi" => "android-armeabi",
            "aarch64-linux-android" => "android64-aarch64",
            "i686-linux-android" => "android-x86",
            "x86_64-linux-android" => "android64",
            _ => panic!("don't know how to configure OpenSSL for {}", target),
        };

        configure.arg(os);

        let mut gcc = gcc::Build::new();
        gcc.target(target)
            .host(host)
            .warnings(false)
            .opt_level(2);
        let compiler = gcc.get_compiler();
        configure.env("CC", compiler.path());
        let path = compiler.path().to_str().unwrap();
        if path.ends_with("-gcc") {
            let path = &path[..path.len() - 4];
            configure.env("RANLIB", format!("{}-ranlib", path));
            configure.env("AR", format!("{}-ar", path));
        }
        for arg in compiler.args() {
            configure.arg(arg);
        }

        if target.contains("android") && self.cross_sysroot.is_none() {
            for path in env::split_paths(&env::var_os("PATH").unwrap()) {
                if !path.join(compiler.path()).exists() {
                    continue
                }
                let path = path.parent().unwrap(); // chop off 'bin'
                self.cross_sysroot = Some(path.join("sysroot"));
                break
            }
        }

        configure.current_dir(&inner_dir);
        self.run_command(configure, "configuring OpenSSL build");

        let mut depend = Command::new("make");
        depend.arg("depend").current_dir(&inner_dir);
        self.run_command(depend, "building OpenSSL dependencies");

        let mut build = Command::new("make");
        build.current_dir(&inner_dir);
        self.run_command(build, "building OpenSSL");

        let mut install = Command::new("make");
        install.arg("install").current_dir(&inner_dir)
            .arg(format!("DESTDIR={}", install_dir.display()));
        self.run_command(install, "installing OpenSSL");

        (lib_dir, include_dir)
    }

    fn run_command(&self, mut command: Command, desc: &str) {
        if let Some(ref path) = self.cross_sysroot {
            command.env("CROSS_SYSROOT", path);
        }
        let status = command.status().unwrap();
        if !status.success() {
            panic!("
    Error {}:
        Command: {:?}
        Exit status: {}
    ",
                desc,
                command,
                status);
        }
    }
}

fn cp_r(src: &Path, dst: &Path) {
    for f in fs::read_dir(src).unwrap() {
        let f = f.unwrap();
        let path = f.path();
        let name = path.file_name().unwrap();
        let dst = dst.join(name);
        if f.file_type().unwrap().is_dir() {
            fs::create_dir_all(&dst).unwrap();
            cp_r(&path, &dst);
        } else {
            let _ = fs::remove_file(&dst);
            fs::copy(&path, &dst).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::env;

    use super::Build;
    use self::tempdir::TempDir;

    #[test]
    fn build() {
        let td = TempDir::new("openssl-build").unwrap();
        let host = include_str!(concat!(env!("OUT_DIR"), "/target"));
        let target = env::var("TARGET_TO_TEST").unwrap_or(host.to_string());
        Build::new()
            .out_dir(td.path())
            .target(&target)
            .host(host)
            .build();
    }
}
