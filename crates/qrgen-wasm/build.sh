#!/bin/sh
# Builds the browser package into crates/qrgen-wasm/pkg (or $1): qrgen.js, qrgen_bg.wasm, qrgen.d.ts.
# Needs wasm-bindgen-cli matching the wasm-bindgen version in Cargo.lock; runs wasm-opt if installed.
# Built with WebAssembly SIMD (see .cargo/config.toml).
set -eu
cd "$(dirname "$0")/../.."
out="${1:-crates/qrgen-wasm/pkg}"

cargo build --release --locked -p qrgen-wasm --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir "$out" --out-name qrgen \
  target/wasm32-unknown-unknown/release/qrgen_wasm.wasm
if command -v wasm-opt >/dev/null; then
  wasm-opt -O3 --enable-simd --enable-bulk-memory --enable-nontrapping-float-to-int --enable-reference-types \
    --enable-multivalue --enable-sign-ext --enable-mutable-globals \
    "$out/qrgen_bg.wasm" -o "$out/qrgen_bg.wasm"
fi
ls -l "$out"
node crates/qrgen-wasm/smoke.mjs "$out"
