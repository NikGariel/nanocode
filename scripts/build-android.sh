#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
: "${ANDROID_NDK_HOME:?Set ANDROID_NDK_HOME to NDK 28.2.13676358}"
case "$(uname -s)" in Darwin) host=darwin-x86_64;; Linux) host=linux-x86_64;; *) echo 'Use macOS or Linux'; exit 1;; esac
bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$host/bin"
for spec in 'aarch64-linux-android arm64-v8a aarch64-linux-android' 'armv7-linux-androideabi armeabi-v7a armv7a-linux-androideabi' 'x86_64-linux-android x86_64 x86_64-linux-android' 'i686-linux-android x86 i686-linux-android'; do
  set -- $spec
  target=$1; abi=$2; clang=$3
  key=$(printf '%s' "$target" | tr '[:lower:]-' '[:upper:]_')
  env "CARGO_TARGET_${key}_LINKER=$bin/${clang}23-clang" RUSTFLAGS='-C link-arg=-Wl,-z,max-page-size=16384' cargo build --locked --release --lib --target "$target"
  dest="packages/android/nanocode/src/main/rustLibs/$abi"
  mkdir -p "$dest"
  cp "target/$target/release/libnanocode.a" "$dest/"
done
