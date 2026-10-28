#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
for target in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build --locked --release --lib --target "$target"
done
mkdir -p artifacts/apple/headers artifacts/apple/simulator artifacts/apple/macos
cp include/nanocode.h artifacts/apple/headers/
printf 'module CNanoCode { header "nanocode.h" export * }\n' > artifacts/apple/headers/module.modulemap
lipo -create target/aarch64-apple-ios-sim/release/libnanocode.a target/x86_64-apple-ios/release/libnanocode.a -output artifacts/apple/simulator/libnanocode.a
lipo -create target/aarch64-apple-darwin/release/libnanocode.a target/x86_64-apple-darwin/release/libnanocode.a -output artifacts/apple/macos/libnanocode.a
# Build into a fresh directory, then replace only the generated framework.
output="artifacts/apple/build-$(date +%s)"
mkdir -p "$output"
xcodebuild -create-xcframework -library target/aarch64-apple-ios/release/libnanocode.a -headers artifacts/apple/headers -library artifacts/apple/simulator/libnanocode.a -headers artifacts/apple/headers -library artifacts/apple/macos/libnanocode.a -headers artifacts/apple/headers -output "$output/CNanoCode.xcframework"
python3 scripts/package-apple.py "$output/CNanoCode.xcframework"
