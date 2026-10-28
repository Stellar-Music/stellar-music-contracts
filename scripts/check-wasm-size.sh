#!/usr/bin/env bash
# Stellar Music Contracts - WASM Size Budget Enforcer
set -euo pipefail

MAX_BUDGET_BYTES=65536 # 64 KB Soroban network protocol budget ceiling
TARGET_WASM="target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm"

echo "=========================================================="
echo "    Stellar Music - WASM Size Budget Enforcer (<64KB)     "
echo "=========================================================="

if [ ! -f "$TARGET_WASM" ]; then
  echo "Building release wasm32 binary..."
  cargo build --target wasm32-unknown-unknown --release
fi

if [ ! -f "$TARGET_WASM" ]; then
  echo "[FAIL] Target WASM not found at $TARGET_WASM"
  exit 1
fi

# Attempt optimization if Soroban / Stellar CLI is available
if command -v stellar &> /dev/null; then
  echo "Optimizing WASM binary via stellar CLI..."
  stellar contract optimize --wasm "$TARGET_WASM" || true
elif command -v soroban &> /dev/null; then
  echo "Optimizing WASM binary via soroban CLI..."
  soroban contract optimize --wasm "$TARGET_WASM" || true
fi

WASM_SIZE=$(wc -c < "$TARGET_WASM" | tr -d ' ')
WASM_KB=$(echo "scale=2; $WASM_SIZE / 1024" | bc 2>/dev/null || awk "BEGIN {printf \"%.2f\", $WASM_SIZE/1024}")
PERCENT=$(echo "scale=1; ($WASM_SIZE * 100) / $MAX_BUDGET_BYTES" | bc 2>/dev/null || awk "BEGIN {printf \"%.1f\", ($WASM_SIZE * 100)/$MAX_BUDGET_BYTES}")

echo "WASM Binary:  $TARGET_WASM"
echo "Binary Size:  $WASM_SIZE bytes ($WASM_KB KB)"
echo "Max Budget:   $MAX_BUDGET_BYTES bytes (64.00 KB)"
echo "Budget Used:  ${PERCENT}%"

if [ "$WASM_SIZE" -gt "$MAX_BUDGET_BYTES" ]; then
  echo ""
  echo "❌ [VIOLATION] WASM size ($WASM_SIZE bytes) exceeds strict budget of $MAX_BUDGET_BYTES bytes!"
  exit 1
else
  echo ""
  echo "✅ [SUCCESS] WASM binary is well within budget ($PERCENT% consumed)."
  exit 0
fi
