target=$1
set -ex
cargo test --manifest-path testcrate/Cargo.toml --target $1 -vv
cargo test --manifest-path testcrate/Cargo.toml --target $1 -vv --release
if [ "$1" = "x86_64-unknown-linux-gnu" ] ; then
	cargo test --manifest-path testcrate/Cargo.toml --target $1 -vv --all-features
fi
