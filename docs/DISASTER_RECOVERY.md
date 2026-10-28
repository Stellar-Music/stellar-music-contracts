# Stellar Music Protocol — Disaster Recovery & Emergency Runbook

## 1. Incident Classification

| Severity Level | Definition | Response SLA | Action Required |
| :---: | :--- | :---: | :--- |
| **P0 — Critical** | Exploitable vulnerability in contract math, funds at risk, or active malicious drained attempt | $< 15\text{ mins}$ | Immediate contract pause, emergency patch deployment |
| **P1 — High** | Horizon RPC outage, database desync, or streaming gateway disruption | $< 1\text{ hour}$ | Trigger RPC failover pool, sync state from on-chain ledgers |
| **P2 — Moderate** | Non-financial UI bug, metadata rendering defect, or offline worker crash | $< 24\text{ hours}$ | Standard patch release and redeployment |

---

## 2. Emergency Pause Containment Procedure

If anomalous contract behavior or financial risk is detected:

```bash
# 1. Admin executes emergency pause via Stellar CLI
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_SECRET_KEY \
  --network testnet \
  -- set_paused \
  --admin $ADMIN_PUBLIC_KEY \
  --paused true
```

### Operational Impact
- `purchase_pass`: **BLOCKED** (`Error::ContractPaused`)
- `execute_split_settlement`: **BLOCKED** (`Error::ContractPaused`)
- Existing listener pass ownership and track metadata queries: **REMAINS ACTIVE**

---

## 3. Administrative Key Rotation Runbook

In the event of an administrator key leak or routine security rotation:

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $CURRENT_ADMIN_SECRET \
  --network testnet \
  -- set_admin \
  --current_admin $CURRENT_ADMIN_PUBLIC \
  --new_admin $NEW_COLD_STORAGE_ADMIN_PUBLIC
```

---

## 4. Hot-Patch WASM Bytecode Upgrade

```bash
# Step 1: Compile patched WASM binary
cargo build --target wasm32-unknown-unknown --release

# Step 2: Install patched WASM on Stellar network to obtain new bytecode hash
NEW_WASM_HASH=$(stellar contract install \
  --wasm target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm \
  --source $ADMIN_SECRET_KEY \
  --network testnet)

# Step 3: Upgrade existing contract instance in place
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_SECRET_KEY \
  --network testnet \
  -- upgrade \
  --admin $ADMIN_PUBLIC_KEY \
  --new_wasm_hash $NEW_WASM_HASH

# Step 4: Resume protocol operations
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_SECRET_KEY \
  --network testnet \
  -- set_paused \
  --admin $ADMIN_PUBLIC_KEY \
  --paused false
```
