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
        if target.contains("pc-windows-gnu") {
            configure.arg(&format!("--prefix={}", sanitize_sh(&install_dir)));
        } else {
            configure.arg(&format!("--prefix={}", install_dir.display()));
        }

        configure
            // No shared objects, we just want static libraries
            .arg("no-dso")
            .arg("no-shared")

            // Should be off by default on OpenSSL 1.1.0, but let's be extra sure
            .arg("no-ssl3")

            // No need to build tests, we won't run them anyway
            .arg("no-unit-test")

            // Nothing related to zlib please
            .arg("no-comp")
            .arg("no-zlib")
            .arg("no-zlib-dynamic")

            // This actually fails to compile on musl (it needs linux/version.h
            // right now) but we don't actually need this most of the time. This
            // is intended for super-configurable backends and whatnot
            // apparently but the whole point of this script is to produce a
            // "portable" implementation of OpenSSL, so shouldn't be any harm in
            // turning this off.
            .arg("no-engine");

        if target.contains("msvc") {
            // On MSVC we need nasm.exe to compile the assembly files, but let's
            // just pessimistically assume for now that's not available.
            configure.arg("no-asm");
        } else {
            // If we're *not* on MSVC then we can optimize our build a bit by
            // avoiding building the CLI tools. Unfortunately though on MSVC if
            // we pass this option the build breaks oddly...
            configure.arg("no-stdio");
        }

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
            "x86_64-pc-windows-msvc" => "VC-WIN64A",
            "i686-pc-windows-msvc" => "VC-WIN64A",
            _ => panic!("don't know how to configure OpenSSL for {}", target),
        };

        configure.arg(os);

        // If we're not on MSVC we configure cross compilers and cross tools and
        // whatnot. Note that this doesn't happen on MSVC b/c things are pretty
        // different there and this isn't needed most of the time anyway.
        if !target.contains("msvc") {
            let mut gcc = gcc::Build::new();
            gcc.target(target)
                .host(host)
                .warnings(false)
                .opt_level(2);
            let compiler = gcc.get_compiler();
            configure.env("CC", compiler.path());
            let path = compiler.path().to_str().unwrap();

            // Infer ar/ranlib tools from cross compilers if the it looks like
            // we're doing something like `foo-gcc` route that to `foo-ranlib`
            // as well.
            if path.ends_with("-gcc") && !target.contains("unknown-linux-musl") {
                let path = &path[..path.len() - 4];
                configure.env("RANLIB", format!("{}-ranlib", path));
                configure.env("AR", format!("{}-ar", path));
            }

            // Make sure we pass extra flags like `-ffunction-sections` and
            // other things like ARM codegen flags.
            for arg in compiler.args() {
                configure.arg(arg);
            }

            // Not really sure why, but on Android specifically the
            // CROSS_SYSROOT variable needs to be set. The build system will
            // pass `--sysroot=$(CROSS_SYSROOT)` so we need to make sure that's
            // set to something. By default we infer it as next to the `bin`
            // directory containing the compiler itself.
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
        }

        // And finally, run the perl configure script!
        configure.current_dir(&inner_dir);
        self.run_command(configure, "configuring OpenSSL build");

        // On MSVC we use `nmake.exe` with a slightly different invocation, so
        // have that take a different path than the standard `make` below.
        if target.contains("msvc") {
            let mut build = gcc::windows_registry::find(target, "nmake.exe")
                .expect("failed to find nmake");
            build.current_dir(&inner_dir);
            self.run_command(build, "building OpenSSL");

            let mut install = gcc::windows_registry::find(target, "nmake.exe")
                .expect("failed to find nmake");
            install.arg("install").current_dir(&inner_dir);
            self.run_command(install, "installing OpenSSL");
        } else {
            let mut depend = Command::new("make");
            depend.arg("depend").current_dir(&inner_dir);
            self.run_command(depend, "building OpenSSL dependencies");

            let mut build = Command::new("make");
            build.current_dir(&inner_dir);
            self.run_command(build, "building OpenSSL");

            let mut install = Command::new("make");
            install.arg("install").current_dir(&inner_dir);
            self.run_command(install, "installing OpenSSL");
        }

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

fn sanitize_sh(path: &Path) -> String {
    if !cfg!(windows) {
        return path.to_str().unwrap().to_string()
    }
    let path = path.to_str().unwrap().replace("\\", "/");
    return change_drive(&path).unwrap_or(path);

    fn change_drive(s: &str) -> Option<String> {
        let mut ch = s.chars();
        let drive = ch.next().unwrap_or('C');
        if ch.next() != Some(':') {
            return None
        }
        if ch.next() != Some('/') {
            return None
        }
        Some(format!("/{}/{}", drive, &s[drive.len_utf8() + 2..]))
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
