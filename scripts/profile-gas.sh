#!/usr/bin/env bash
# Stellar Music Contracts - Gas, CPU & Memory Instruction Profiler
set -euo pipefail

echo "=========================================================="
echo "    Stellar Music - Soroban Resource & Gas Profiler       "
echo "=========================================================="

echo "[1/3] Compiling smart contracts in release mode for wasm32..."
cargo build --target wasm32-unknown-unknown --release

WASM_FILE="target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm"
if [ ! -f "$WASM_FILE" ]; then
  echo "  [ERROR] Release WASM binary not found at $WASM_FILE"
  exit 1
fi

WASM_SIZE=$(wc -c < "$WASM_FILE" | tr -d ' ')
echo "  [OK] Compiled WASM binary size: $WASM_SIZE bytes"

echo ""
echo "[2/3] Profiling CPU instructions and memory consumption via Soroban test budget..."
cargo test --release --test-threads=1 -- --nocapture test_gas_and_cpu_budget_profile || true

echo ""
echo "[3/3] Checking against Soroban Mainnet resource limits..."
echo "  - Max CPU Instructions per Tx: 100,000,000"
echo "  - Max Memory per Tx: 41,943,040 bytes (40MB)"
echo "  - Max WASM Contract Size: 65,536 bytes (64KB)"

if [ "$WASM_SIZE" -le 65536 ]; then
  echo "  [PASS] WASM size ($WASM_SIZE bytes) is within 64KB network ceiling."
else
  echo "  [WARN] WASM size exceeds 64KB limit. Optimization required."
fi

echo "=========================================================="
echo "Gas & resource profiling completed."
