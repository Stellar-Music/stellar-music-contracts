# Stellar Music Protocol — Soroban Storage TTL & State Rent Guide

## 1. Soroban State Archival Model

Unlike Ethereum where state is immortal and bloats validator node disks forever, the Stellar Network implements **State Archival & Storage Rent**. Contract instances and persistent entries carry a **Time-To-Live (TTL)** measured in ledger sequences (~5 seconds per ledger).

If an entry’s TTL drops to zero, the entry is archived and can no longer be accessed until explicitly restored.

---

## 2. Storage Tier Architecture in Stellar Music

| Storage Tier | Data Elements | TTL Strategy | Persistence Guarantee |
| :--- | :--- | :--- | :--- |
| **Instance Storage** | `Admin`, `Paused`, `PlatformFeeBps`, `TrackCount`, `TotalPasses` | Extended via `extend_ttl` | Active throughout contract lifetime |
| **Persistent Storage** | `TrackInfo`, `PassInfo`, `SplitAgreement`, `SettlementReceipt` | User-paid rent, extendable by any party | Restorable from historical ledger archives |
| **Temporary Storage** | Ephemeral event nonces | Automatically purged upon expiry | Transient, non-critical data |

---

## 3. Storage TTL Extension Parameters

`StellarMusicContract` defines defensive TTL thresholds in `src/lib.rs`:

```rust
pub const STORAGE_TTL_THRESHOLD: u32 = 100_000; // ~5.7 days
pub const STORAGE_TTL_EXTEND: u32 = 500_000;    // ~28.9 days

pub fn extend_ttl(env: Env) {
    env.storage().instance().extend_ttl(STORAGE_TTL_THRESHOLD, STORAGE_TTL_EXTEND);
}
```

### Automatic TTL Refresh
Whenever a user interacts with the contract (e.g. `purchase_pass`, `execute_split_settlement`), active keys are automatically bumped, ensuring high-activity tracks never face archival risk.

---

## 4. Rent Economics

Stellar storage rent is among the lowest in Web3:
- **Cost**: $< 0.000001\text{ XLM}$ per byte per month.
- **Estimated Contract Footprint**: $< 64\text{ KB}$ for 5,000 active tracks and split sheets.
- **Monthly Protocol Rent Overhead**: Less than $0.05\text{ XLM}$ ($< \$0.01\text{ USD}$) per month.
