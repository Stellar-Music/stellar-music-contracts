# Stellar Music Smart Contracts — Formal Security Specification

## 1. Security Architecture & Threat Model

The `StellarMusicContract` is engineered under defensive security standards for financial settlement on the Stellar Network using Soroban Rust SDK v22.

---

## 2. Threat Vector Mitigations

| Threat Vector | Risk Severity | Protocol Mitigation Strategy |
| :--- | :--- | :--- |
| **Reentrancy Attacks** | High | Soroban execution environment does not allow recursive contract callbacks. External token transfers (`token::Client::transfer`) execute atomically at completion of function state modifications. |
| **Arithmetic Overflows** | High | Contract compiled with `overflow-checks = true`. All financial amounts are typed as 128-bit signed integers (`i128`), and basis points math uses integer division with bounds checking: `(gross * bps) / 10000`. |
| **Front-Running / MEV** | Medium | Stellar Consensus Protocol (SCP) processes transactions in federated validator rounds with deterministic sequencing, mitigating Ethereum-style gas auction front-running. |
| **Signature Replay** | High | Split agreement terms approvals require matching canonical `agreement_hash` generated over canonical JSON containing track ID, version, and sorted contributors. Mismatched terms fail with `HashMismatch`. |
| **Double-Spending / Duplicate Minting** | High | Listener passes are indexed by composite key `DataKey::Pass(listener, track_id)`. Re-purchase attempts fail immediately with `PassAlreadyPurchased`. |
| **Unbounded Loop Gas Exhaustion** | Medium | Multi-recipient split agreements enforce a strict `MAX_RECIPIENTS` cap (25 collaborators). Loop executions over recipient vectors are strictly $O(1)$ bounded. |
| **Storage Expiration / State Loss** | Medium | Contract implements `extend_ttl` using Soroban persistent and instance storage lifecycle APIs (`STORAGE_TTL_THRESHOLD = 100,000`, `STORAGE_TTL_EXTEND = 500,000`). |

---

## 3. Authorization Boundaries

```
[Contract Invocations]
       │
       ├── Admin Only ──────────► initialize(admin)
       │                         set_admin(new_admin)
       │                         set_platform_fee(treasury, fee_bps)
       │                         set_paused(paused)
       │                         upgrade(new_wasm_hash)
       │
       ├── Artist Only ─────────► register_track(price, metadata_uri)
       │                         update_track(price, is_active)
       │                         archive_track(track_id)
       │                         update_metadata_uri(metadata_uri)
       │                         create_split_agreement(hash, recipients)
       │
       ├── Collaborator Only ───► approve_split_agreement(version, hash)
       │
       └── Public / Listener ───► purchase_pass(track_id, token)
                                 execute_split_settlement(track_id, gross, token)
                                 has_valid_pass(listener, track_id)
                                 calculate_split_allocations(track_id, gross)
```

---

## 4. Emergency Circuit Breaker Runbook

1. **Trigger Condition**: Malicious contract invocation pattern, token contract anomaly, or critical vulnerability discovery.
2. **Action**: Authorized administrator executes `set_paused(admin, true)`.
3. **Effect**:
   - `purchase_pass` calls immediately abort with `Error::ContractPaused`.
   - `execute_split_settlement` calls abort with `Error::ContractPaused`.
   - Read-only queries (`has_valid_pass`, `get_track`, `get_split_agreement`) remain operational.
4. **Resolution**:
   - Deploy patched WASM bytecode via `stellar contract install`.
   - Admin executes `upgrade(admin, new_wasm_hash)`.
   - Admin verifies patched state and executes `set_paused(admin, false)`.
