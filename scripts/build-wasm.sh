#!/usr/bin/env bash
set -euo pipefail

echo "🔨 Building optimized Soroban WASM binary..."
cargo build --target wasm32-unknown-unknown --release

WASM_PATH="target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm"
if [ -f "$WASM_PATH" ]; then
  SIZE=$(wc -c <"$WASM_PATH" | tr -d ' ')
  echo "✅ Contract WASM build successful!"
  echo "📦 Binary path: $WASM_PATH"
  echo "📊 Binary size: $SIZE bytes"
else
  echo "❌ Error: WASM binary not found at $WASM_PATH"
  exit 1
fi
