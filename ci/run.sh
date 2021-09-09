target=$1
set -ex

if [ "$1" = "aarch64-apple-darwin" ] ; then
	sudo xcode-select -s /Applications/Xcode_12.2.app/Contents/Developer/
	export SDKROOT=$(xcrun -sdk macosx11.0 --show-sdk-path)
	export MACOSX_DEPLOYMENT_TARGET=$(xcrun -sdk macosx11.0 --show-sdk-platform-version)
	export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=echo
fi

# Remove directories that are excluded by Cargo.toml
rm -rf openssl/boringssl
rm -rf openssl/fuzz
rm -rf openssl/krb5
rm -rf openssl/pyca-cryptography
rm -rf openssl/test
rm -rf openssl/wycheproof

cargo test --manifest-path testcrate/Cargo.toml --target $1 -vv
cargo test --manifest-path testcrate/Cargo.toml --target $1 -vv --release
if [ "$1" = "x86_64-unknown-linux-gnu" ] ; then
	cargo test --manifest-path testcrate/Cargo.toml --target $1 -vv --all-features
fi
