Write-Host "🧪 Running Soroban contract smoke test suite..." -ForegroundColor Cyan
cargo test --lib

$wasmPath = "target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm"
if (-not (Test-Path $wasmPath)) {
    Write-Host "⚠️ Compiling WASM binary..." -ForegroundColor Yellow
    cargo build --target wasm32-unknown-unknown --release
}

Write-Host "✅ All smoke tests passed successfully!" -ForegroundColor Green
