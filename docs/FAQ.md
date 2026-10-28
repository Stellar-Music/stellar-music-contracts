# Stellar Music Protocol — Frequently Asked Questions (FAQ)

Comprehensive answers to key architectural, economic, and technical questions for hackathon judges, protocol developers, and recording artists.

---

## 1. Questions for Hackathon Judges & Reviewers

### Q1.1: Why did you choose Stellar and Soroban instead of an EVM L2 or Solana?
**Answer:** 
Music streaming micro-payments require high throughput, sub-cent transaction fees, and instant deterministic settlement. 
1. **Unrivaled Micro-Transaction Economics**: Stellar transaction fees cost < 0.00001 XLM (< $0.00001 USD), compared to $0.05–$0.50 on EVM L2s. Paying an artist $0.002 per second of stream time is economically infeasible when gas exceeds the payment value.
2. **Soroban Rust Safety**: Soroban smart contracts are compiled to WebAssembly from Rust, providing memory safety, strict zero-cost abstractions, and deterministic execution without EVM reentrancy vulnerabilities.
3. **Native Asset Interoperability**: Stellar Classic assets (USDC, XLM) interact seamlessly with Soroban contracts via SEP-41 Stellar Asset Contracts (SAC), eliminating fragile wrapped-token bridges.

### Q1.2: How do you prevent rounding loss when splitting micro-payments across multiple collaborators?
**Answer:**
We mathematically guarantee 0-stroop precision loss through an exact integer division remainder allocation algorithm:
```rust
let split_amount = (total_amount * bps) / 10000;
// Remainder stroops are allocated deterministically to the primary artist
```
This is verified by 10,000 randomized invariant test iterations in both Rust (`test_contract_stroop_loss_invariant`) and TypeScript (`tests/stroop_math_invariant.test.ts`).

### Q1.3: What happens if an RPC node drops or Stellar experiences network jitter?
**Answer:**
The backend implements an active Horizon RPC Failover Pool with exponential backoff, health probing, and automatic circuit breaking. If the primary node fails or throttles, the client automatically switches to backup endpoints without interrupting audio streaming or payment recording.

---

## 2. Questions for Web3 Developers & Integrators

### Q2.1: How do I run the full test suite across the entire project?
**Answer:**
The project features 253 automated tests across three repositories:
- **Contracts (34 tests)**: `cd stellar-music-contracts && cargo test`
- **Backend (89 tests)**: `cd stellar-music-backend && npm test`
- **Frontend (130 tests)**: `cd stellar-music-frontend && npm test`

### Q2.2: How does the contract handle Soroban storage TTL and state archival?
**Answer:**
The contract utilizes Soroban Persistent and Instance storage with automatic TTL extension. Every state-modifying call (`purchase_pass`, `record_stream`, `create_split_agreement`) extends the storage TTL by 100,000 ledgers, ensuring active accounts and agreements never enter the archived state.

### Q2.3: Can third-party apps integrate with the revenue split engine?
**Answer:**
Yes. Any dApp or platform can invoke `create_split_agreement` or query `get_split_agreement` by track ID. The split engine conforms to the SEP-41 token interface and accepts any SAC-compatible token (XLM, USDC, EURC).

---

## 3. Questions for Artists & Creators

### Q3.1: How soon do I receive my streaming royalties?
**Answer:**
Instantaneously. Unlike Web2 platforms (Spotify, Apple Music) that withhold royalties for 60 to 90 days and deduct 30% aggregator margins, Stellar Music settles payments to your Stellar wallet within 3 to 5 seconds of the listener pressing play.

### Q3.2: Can I share royalties automatically with my producer and featured artist?
**Answer:**
Yes. You can configure up to 10 split recipients with basis point precision (1 bp = 0.01%). When a fan streams your track or purchases a pass, Soroban executes an atomic multi-transfer transaction in a single ledger entry.

### Q3.3: Do my listeners need cryptocurrency to listen?
**Answer:**
Listeners can connect existing Stellar wallets (Freighter, Lobstr, xBull), or experience audio previews with gasless testnet passes sponsored by our platform relayer.
