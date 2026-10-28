#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown --lib

cp target/wasm32-unknown-unknown/release/nanocode.wasm packages/js/nanocode.wasm
