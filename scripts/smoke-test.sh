#!/usr/bin/env bash
set -euo pipefail

echo "🧪 Running Soroban contract smoke test suite..."
cargo test --lib -- --nocapture

echo "📦 Checking WASM binary status..."
if [ ! -f "target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm" ]; then
  echo "⚠️ Compiling WASM binary..."
  cargo build --target wasm32-unknown-unknown --release
fi

echo "✅ All smoke tests passed successfully!"
