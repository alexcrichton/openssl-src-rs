target=$1
set -ex
cargo test --manifest-path testcrate/Cargo.toml --target $1 -vv
