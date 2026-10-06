# 🎵 Stellar Music — Smart Contracts (`stellar-music-contracts`)

> **🚀 Live Web Application**: [https://stellar-music-app.netlify.app](https://stellar-music-app.netlify.app)

Smart contract financial foundation for **Stellar Music**, built on **Soroban (Rust SDK v22)** for the **Stellar Testnet**.

## 📌 Architectural Responsibility

* **Smart contracts enforce financial rules & agreement immutability.**
* **Level 1**: Direct peer-to-peer / escrow payment settlement (`purchase_pass`). The backend never holds custody or arbitrary authority over user funds.
* **Level 2 (NEW)**: Multi-party **Collaborator Revenue Split Agreements**:
  * Track artists register deterministic revenue split agreements with contributors.
  * Every contributor reviews and cryptographically approves the agreement with their wallet.
  * Enforces exact 10,000 basis points (100.00%) allocation.
  * Rejects duplicate recipients, invalid percentages, and unauthorized signatures.
  * Automatically transitions to **`LOCKED`** once all required contributors have signed.
  * Once locked, percentages and contributors are strictly immutable.
* **Level 3 Compatibility**: Exposes `get_active_locked_split(track_id)` for future automatic revenue settlement pools.

```
       [ Collaborator Wallets ]
                │
     (Cryptographic Approvals)
                ▼
   ┌──────────────────────────────────┐
   │     Soroban Smart Contract       │
   │  - register_track() (L1)         │
   │  - purchase_pass() (L1)          │
   │  - create_split_agreement() (L2) │
   │  - approve_split_agreement() (L2)│
   │  - get_active_locked_split() (L3)│
   └─────────────┬────────────────────┘
                 │ (Emits Lifecycle Events)
                 ▼
         [ Stellar Ledger ]
```

---

## 🛠️ Contract Interface

### Level 2: Collaborator Revenue Split Agreements

| Method | Parameters | Returns | Description |
| :--- | :--- | :--- | :--- |
| `create_split_agreement` | `artist: Address`, `track_id: u64`, `agreement_hash: BytesN<32>`, `recipients: Vec<SplitRecipient>` | `Result<u32, Error>` | Creates a new split agreement version. Verifies 10,000 basis points and no duplicates. Requires artist auth. |
| `approve_split_agreement` | `contributor: Address`, `track_id: u64`, `version: u32`, `agreement_hash: BytesN<32>` | `Result<bool, Error>` | Contributor cryptographically approves the exact terms. Locks agreement once all parties sign. |
| `get_split_agreement` | `track_id: u64`, `version: u32` | `Option<SplitAgreement>` | Queries agreement metadata, recipients, approvals, and lock status. |
| `get_active_locked_split` | `track_id: u64` | `Option<SplitAgreement>` | Queries the authoritative locked agreement ready for Level 3 settlement. |
| `get_latest_split_version`| `track_id: u64` | `u32` | Returns the latest agreement version counter for the track. |
| `is_agreement_locked` | `track_id: u64`, `version: u32` | `bool` | Returns `true` if agreement is locked. |

### Level 1: Core Access & Payments

| Method | Parameters | Returns | Description |
| :--- | :--- | :--- | :--- |
| `initialize` | `admin: Address` | `Result<(), Error>` | Initializes contract instance with administrator. |
| `register_track` | `artist: Address`, `pass_price: i128`, `metadata_uri: String` | `Result<u64, Error>` | Registers a new track. Emits `track::reg` event. Requires artist signature. |
| `update_track` | `artist: Address`, `track_id: u64`, `pass_price: i128`, `is_active: bool` | `Result<(), Error>` | Updates track availability or pricing. Requires artist signature. |
| `purchase_pass` | `listener: Address`, `track_id: u64`, `token: Address` | `Result<PassInfo, Error>` | Purchases Music Pass. Transfers tokens directly to artist. Mints pass record. |
| `has_valid_pass` | `listener: Address`, `track_id: u64` | `bool` | Returns `true` if listener holds a valid Music Pass for the track. |

---

## 🧪 Testing

The contract test suite verifies full financial mechanics, multi-party agreement state machine, access control, and replay protection:

```bash
cargo test
```

### Verified Test Cases:
1. `test_contract_initialization`: Verifies initial zero-state and admin setup.
2. `test_register_track`: Verifies track registration, metadata URIs, and pricing.
3. `test_purchase_pass_and_payment_transfer`: Validates token deduction and pass issuance.
4. `test_level2_create_split_agreement_success`: Verifies creation of multi-contributor split agreements.
5. `test_level2_reject_invalid_basis_points`: Rejects totals not equal to 10,000 basis points.
6. `test_level2_reject_duplicate_recipient`: Rejects duplicate contributor wallet entries.
7. `test_level2_multi_party_signing_flow_and_lock`: Tests multi-party signing flow and automatic locking.
8. `test_level2_non_recipient_cannot_approve`: Rejects approvals from unlisted accounts.
9. `test_level2_hash_mismatch_rejection`: Protects against signing mismatched agreement terms.

---

## 📦 Building WebAssembly Target

To compile the optimized WebAssembly contract for Stellar Testnet:

```bash
cargo build --target wasm32-unknown-unknown --release
```

Compiled artifact:
```text
target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm
```
