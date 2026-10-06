# 🎵 Stellar Music — Smart Contracts (`stellar-music-contracts`)

Smart contract financial foundation for **Stellar Music**, built on **Soroban (Rust SDK v22)** for the **Stellar Testnet**.

## 📌 Architectural Responsibility

* **Smart contracts enforce financial rules.**
* **Direct peer-to-peer / escrow payment settlement**: Funds flow directly from the listener to the artist (or into future locked revenue pools). The backend never holds custody or arbitrary authority over user funds.
* **On-chain Access Tokenization**: Music Passes are recorded on-chain with immutable timestamps and ledger sequences.
* **Level 2 & Level 3 Extensibility**: Includes foundational hooks for multi-party split agreements and automated settlement pools.

```
       [ Listener Wallet ]
                │
         (Signs Payment)
                ▼
   ┌───────────────────────────┐
   │    Soroban Smart Contract │
   │  - register_track()       │
   │  - purchase_pass()        │──────► [ Direct Transfer to Artist ]
   │  - has_valid_pass()       │
   │  - register_split() (L2)  │
   └─────────────┬─────────────┘
                 │ (Emits PassPurchased Event & State)
                 ▼
         [ Stellar Ledger ]
```

---

## 🛠️ Contract Interface

### Core Methods (Level 1: Access & Payment)

| Method | Parameters | Returns | Description |
| :--- | :--- | :--- | :--- |
| `initialize` | `admin: Address` | `Result<(), Error>` | Initializes contract instance with administrator. |
| `register_track` | `artist: Address`, `pass_price: i128`, `metadata_uri: String` | `Result<u64, Error>` | Registers a new track. Emits `track::reg` event. Requires artist signature. |
| `update_track` | `artist: Address`, `track_id: u64`, `pass_price: i128`, `is_active: bool` | `Result<(), Error>` | Updates track availability or pricing. Requires artist signature. |
| `purchase_pass` | `listener: Address`, `track_id: u64`, `token: Address` | `Result<PassInfo, Error>` | Purchases Music Pass. Transfers tokens directly to artist. Mints pass record. |
| `has_valid_pass` | `listener: Address`, `track_id: u64` | `bool` | Returns `true` if listener holds a valid Music Pass for the track. |
| `get_track` | `track_id: u64` | `Option<TrackInfo>` | Fetches metadata and pricing for a track. |
| `get_pass` | `listener: Address`, `track_id: u64` | `Option<PassInfo>` | Fetches pass purchase receipt and timestamp. |
| `get_track_count`| *None* | `u64` | Returns total registered tracks. |
| `get_total_passes`| *None* | `u64` | Returns total platform passes issued. |

### Level 2 & 3 Extensible Foundation Hooks

| Method | Parameters | Description |
| :--- | :--- | :--- |
| `register_split_agreement` | `artist: Address`, `track_id: u64`, `recipients: Vec<SplitRecipient>` | Registers multi-party revenue shares totaling exactly 10,000 basis points (100.00%). |
| `get_split_agreement` | `track_id: u64` | Fetches locked or draft split agreement for track collaborators. |

---

## 🧪 Testing

The contract test suite verifies full financial mechanics, mock token transfers, access state queries, and protection mechanisms.

Run the test suite:

```bash
cargo test
```

### Verified Test Cases:
1. `test_contract_initialization`: Verifies initial zero-state and admin setup.
2. `test_register_track`: Verifies track registration, metadata URIs, and pricing.
3. `test_purchase_pass_and_payment_transfer`: Validates complete token deduction from listener and direct crediting to artist.
4. `test_duplicate_purchase_prevention`: Rejects accidental or duplicate pass purchases.
5. `test_cannot_purchase_inactive_track`: Protects against buying unlisted/deactivated tracks.
6. `test_level2_split_agreement_foundation`: Validates 100% basis-point split calculations for future multi-party settlement.

---

## 📦 Building the WebAssembly Target

To compile the optimized WebAssembly contract for deployment to Stellar Testnet:

```bash
cargo build --target wasm32-unknown-unknown --release
```

Compiled artifact:
```text
target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm
```

---

## 🚀 Deployment Walkthrough (Stellar Testnet)

### 1. Configure Stellar CLI Testnet Identity
```bash
stellar keys generate --global alice --network testnet
stellar keys fund alice --network testnet
```

### 2. Deploy Contract WASM to Testnet
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_music_contracts.wasm \
  --source alice \
  --network testnet
```

### 3. Initialize Contract
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- initialize \
  --admin alice
```

---

## 🗺️ Roadmap to Levels 2 & 3

* **Level 1 (Current)**: Direct Music Pass access, artist payment settlement, and pass validation.
* **Level 2 (Agreement)**: Multi-party signature collection for revenue splits (`register_split_agreement`), locking split versioning on-chain.
* **Level 3 (Settlement)**: Automated streaming revenue pools distributing payouts according to locked basis points directly to collaborator wallets in real time.
