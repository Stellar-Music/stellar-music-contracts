Write-Host "🔨 Building optimized Soroban WASM binary..." -ForegroundColor Cyan
cargo build --target wasm32-unknown-unknown --release

$wasmPath = "target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm"
if (Test-Path $wasmPath) {
    $bytes = (Get-Item $wasmPath).Length
    Write-Host "✅ Contract WASM build successful!" -ForegroundColor Green
    Write-Host "📦 Binary path: $wasmPath"
    Write-Host "📊 Binary size: $bytes bytes"
} else {
    Write-Host "❌ Error: WASM binary not found at $wasmPath" -ForegroundColor Red
    exit 1
}
