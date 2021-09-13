target=$1
testcrate_dir="$(pwd)/testcrate"
set -ex

if [ "$1" = "aarch64-apple-darwin" ] ; then
	sudo xcode-select -s /Applications/Xcode_12.2.app/Contents/Developer/
	export SDKROOT=$(xcrun -sdk macosx11.0 --show-sdk-path)
	export MACOSX_DEPLOYMENT_TARGET=$(xcrun -sdk macosx11.0 --show-sdk-platform-version)
	export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=echo
fi

cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $1 -vv
cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $1 -vv --release

if [ "$1" = "x86_64-unknown-linux-gnu" ] ; then
	cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $1 -vv --all-features

	# Ensure we don't rely on any files excluded by Cargo.toml
	rm -rf target/ci
	cargo package --allow-dirty --target-dir target/ci
	rm -f target/ci/package/*.crate
	cd target/ci/package/openssl-src-*
	cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $1 -vv
fi
