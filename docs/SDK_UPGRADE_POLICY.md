# Stellar Music Protocol — Soroban SDK & Protocol Version Upgrade Policy

## 1. Current Environment Baseline

- **Soroban Rust SDK**: `v22.0.6`
- **Target Architecture**: `wasm32-unknown-unknown`
- **Rust Toolchain**: `stable` (edition 2021)
- **Stellar Network Protocol**: Protocol 21 / Protocol 22 compatible

---

## 2. Compatibility Verification Protocol

Before upgrading `soroban-sdk` or `soroban-env-host` versions:

1. **Dependency Audit**: Verify new SDK releases against RustSec advisories using `cargo deny check`.
2. **Deterministic Test Validation**: All 34 contract tests must pass with zero warnings (`cargo test`).
3. **Binary Size Budget Assertion**:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   bash scripts/check-wasm-size.sh
   ```
   *Ceiling constraint*: The resulting WASM binary **must remain under 51,200 bytes (50KB)**.
4. **Snapshot Regressions Check**: Verify that `test_snapshots/` JSON artifacts reflect expected host function invocations without unbudgeted instruction inflation.

---

## 3. On-Chain Upgrade Lifecycle

Once a new WASM binary is certified:
1. Administrator installs new bytecode on-chain via `stellar contract install`.
2. Multi-sig council approves migration and invokes `upgrade(admin, new_wasm_hash)`.
3. In-memory state and persistent storage (`TrackInfo`, `PassInfo`, `SplitAgreement`) carry forward seamlessly without data migration scripts.
