# 🎵 Stellar Music — Smart Contracts (`stellar-music-contracts`)

[![Live Application](https://img.shields.io/badge/Live%20App-stellar--music--app.netlify.app-00f2fe?style=for-the-badge)](https://stellar-music-app.netlify.app)
[![Stellar Testnet](https://img.shields.io/badge/Stellar-Testnet-blue?style=for-the-badge&logo=stellar)](https://stellar.expert/explorer/testnet)
[![Soroban SDK](https://img.shields.io/badge/Soroban%20SDK-v22.0.0-purple?style=for-the-badge)](https://stellar.org)
[![Tests](https://img.shields.io/badge/Contract%20Tests-12%2F12%20Passing-emerald?style=for-the-badge)](https://github.com/Stellar-Music/stellar-music-contracts)

The decentralized financial engine and settlement protocol for **Stellar Music**, built on **Soroban (Rust SDK v22)** for the **Stellar Network**.

---

## 📌 Protocol Overview

Stellar Music eliminates opaque royalty distribution and intermediary payment delays by embedding revenue agreements and payouts directly into Stellar smart contracts:

1. **Non-Custodial Financial Enforcement**: Neither artists, listeners, nor platform servers have unilateral custody of funds. Smart contract rules govern all transactions.
2. **Cryptographic Revenue Split Agreements**: Collaborators (producers, vocalists, songwriters) register and sign immutable split agreements on-chain. Exactly 10,000 basis points (100.00%) are allocated, verified, and locked.
3. **Automated Multi-Recipient Settlements**: Streaming revenue ingested into track pools is automatically distributed to all verified collaborator wallets in a single atomic transaction.
4. **Zero-Leak Integer Stroop Math**: 1 XLM = 10,000,000 stroops. Indivisible remainder dust is deterministically assigned to the primary collaborator (index 0), guaranteeing zero financial rounding leakage across any number of split participants.

```
                     ┌────────────────────────────────┐
                     │     Music Pass Streaming Fee   │
                     └───────────────┬────────────────┘
                                     │
                                     ▼
                     ┌────────────────────────────────┐
                     │    Track Revenue Allocation    │
                     └───────────────┬────────────────┘
                                     │
                       (Locked Split Agreement Check)
                                     ▼
     ┌────────────────────────────────────────────────────────────────┐
     │                Soroban Split Settlement Engine                 │
     │  - Verifies Agreement Status == LOCKED                         │
     │  - Computes Integer Basis Points (gross_stroops * bps / 10000) │
     │  - Allocates Indivisible Remainder Stroops to Primary Artist   │
     │  - Atomically Transfers XLM to Every Contributor Wallet        │
     │  - Emits SettlementReceipt Event & Ledger Audit Record         │
     └───────┬──────────────────────┬──────────────────────┬──────────┘
             │                      │                      │
             ▼                      ▼                      ▼
    [ Producer (40%) ]     [ Vocalist (35%) ]      [ Writer (25%) ]
```

---

## 🛠️ Smart Contract Specification

### Core Methods

| Method | Parameters | Returns | Description |
| :--- | :--- | :--- | :--- |
| `initialize` | `admin: Address` | `Result<(), Error>` | Initializes contract administrator. |
| `register_track` | `artist: Address`, `pass_price: i128`, `metadata_uri: String` | `Result<u64, Error>` | Registers track on-chain with pass price and metadata URI. Emits `track::reg`. |
| `update_track` | `artist: Address`, `track_id: u64`, `pass_price: i128`, `is_active: bool` | `Result<(), Error>` | Updates track metadata and active status. Requires artist auth. |
| `purchase_pass` | `listener: Address`, `track_id: u64`, `token: Address` | `Result<PassInfo, Error>` | Purchases access pass. Transfers payment tokens directly to artist account. |
| `create_split_agreement` | `artist: Address`, `track_id: u64`, `agreement_hash: BytesN<32>`, `recipients: Vec<SplitRecipient>` | `Result<u32, Error>` | Registers a new split agreement version. Validates 10,000 bps invariant and recipient uniqueness. |
| `approve_split_agreement` | `contributor: Address`, `track_id: u64`, `version: u32`, `agreement_hash: BytesN<32>` | `Result<bool, Error>` | Contributor cryptographically approves agreement terms. Automatically transitions status to `LOCKED` when all parties sign. |
| `get_split_agreement` | `track_id: u64`, `version: u32` | `Option<SplitAgreement>` | Queries full agreement metadata, version, terms, approvals, and lock state. |
| `get_active_locked_split` | `track_id: u64` | `Option<SplitAgreement>` | Fetches active locked agreement ready for settlement execution. |
| `calculate_split_allocations` | `track_id: u64`, `gross_amount: i128` | `Result<Vec<RecipientPayout>, Error>` | Pure dry-run calculation returning exact stroop allocations and dust assignment. |
| `execute_split_settlement` | `caller: Address`, `track_id: u64`, `gross_amount: i128`, `token: Address` | `Result<SettlementReceipt, Error>` | Executes atomic multi-recipient transfer on Stellar backed by locked agreement. Emits `settle::exec`. |
| `get_settlement` | `settlement_id: u64` | `Option<SettlementReceipt>` | Queries on-chain settlement receipt by global settlement index. |
| `get_settlement_count` | *None* | `u64` | Returns total lifetime settlements executed on the contract. |
| `get_track_settled_total` | `track_id: u64` | `i128` | Returns lifetime cumulative settled amount for a specific track. |

---

## 🔒 Financial Invariants & Security Architecture

1. **Exact 100.00% Allocation (`BasisPointsSumInvalid`)**: Every agreement must total exactly 10,000 basis points. Under-allocation or over-allocation is rejected at transaction submission.
2. **Duplicate Prevention (`DuplicateRecipient`)**: A wallet cannot be specified multiple times within the same split agreement.
3. **Lock Gatekeeper (`AgreementNotLocked`)**: Settlement execution is strictly prohibited unless the agreement has reached the `LOCKED` state through unanimous multi-party cryptographic approvals.
4. **Remainder Dust Zero-Leak Invariant**:
   $$\sum_{i=0}^{n-1} \text{actual\_amount}_i = \text{gross\_amount}$$
   $$\text{dust} = \text{gross\_amount} - \sum_{i=0}^{n-1} \left( \frac{\text{gross\_amount} \times \text{bps}_i}{10000} \right)$$
   $$\text{actual\_amount}_0 = \text{calculated\_amount}_0 + \text{dust}$$
5. **Replay & Hash Binding**: Every approval requires passing the agreement's SHA-256 hash. If terms are modified, previous signatures are rendered invalid.

---

## 📊 Data Structures

```rust
pub struct SplitRecipient {
    pub recipient: Address,
    pub share_basis_points: u32, // 10,000 bps = 100.00%
}

pub struct SplitAgreement {
    pub track_id: u64,
    pub version: u32,
    pub agreement_hash: BytesN<32>,
    pub recipients: Vec<SplitRecipient>,
    pub approvals: Vec<Address>,
    pub is_locked: bool,
    pub created_at: u64,
}

pub struct RecipientPayout {
    pub recipient: Address,
    pub share_basis_points: u32,
    pub amount: i128,
}

pub struct SettlementReceipt {
    pub settlement_id: u64,
    pub track_id: u64,
    pub agreement_version: u32,
    pub agreement_hash: BytesN<32>,
    pub gross_amount: i128,
    pub token: Address,
    pub payouts: Vec<RecipientPayout>,
    pub settled_at: u64,
}
```

---

## 🧪 Testing & Verification

The contract includes comprehensive test suites covering unit logic, multi-party signature flows, basis points validation, and multi-recipient settlement execution:

```bash
# Run contract unit tests
cargo test --lib

# Build optimized WASM binary for deployment
cargo build --target wasm32-unknown-unknown --release
```

All 12 automated test cases pass with zero failures:
* `test_initialize`
* `test_register_and_get_track`
* `test_purchase_pass`
* `test_duplicate_pass_prevention`
* `test_create_split_agreement_success`
* `test_split_agreement_basis_points_validation`
* `test_split_agreement_duplicate_recipient_rejected`
* `test_multi_party_split_approval_and_locking`
* `test_unauthorized_approval_rejected`
* `test_calculate_split_allocations_dust_remainder_invariant`
* `test_execute_split_settlement_locked_agreement`
* `test_execute_split_settlement_rejected_if_not_locked`

---

## 🌐 Network Configuration

* **Network**: Stellar Testnet
* **Soroban RPC**: `https://soroban-testnet.stellar.org`
* **Network Passphrase**: `Test SDF Network ; September 2015`
* **Testnet Explorer**: [Stellar Expert Explorer](https://stellar.expert/explorer/testnet)
* **Frontend Web Application**: [https://stellar-music-app.netlify.app](https://stellar-music-app.netlify.app)
