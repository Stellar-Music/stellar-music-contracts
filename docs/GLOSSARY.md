# Stellar Music Protocol — Glossary of Terms

A reference guide defining cryptographic, protocol, and music royalty terminology used across the Stellar Music ecosystem.

---

## 1. Stellar & Soroban Core Concepts

- **Stellar Asset Contract (SAC)**: A built-in Soroban smart contract interface (defined under SEP-41) that allows Stellar Classic assets (like native XLM and issued tokens like USDC) to be manipulated directly by Soroban smart contracts without wrapping or bridging.
- **Stroop**: The smallest atomic unit of XLM. 1 XLM = 10,000,000 stroops (7 decimal places). All mathematical operations in `StellarMusicContract` operate on raw integer stroops to eliminate floating-point rounding errors.
- **Ledger Sequence**: A monotonically increasing integer representing the block height of the Stellar distributed ledger. New ledgers close every ~5 seconds.
- **Instance Storage**: Soroban contract storage tied to the contract instance itself, holding shared operational parameters such as administrator address, paused status, and platform fee basis points.
- **Persistent Storage**: Soroban contract storage allocated per user or agreement, retained indefinitely provided storage rent is maintained via TTL extensions.
- **Time To Live (TTL)**: The lifespan in ledgers of a Soroban storage entry before it becomes archived. Our protocol automatically extends TTL during active transactions.
- **StrKey (Stellar Key Format)**: Base32-encoded public keys and contract identifiers with CRC checksums, beginning with `G` (ed25519 public account) or `C` (Soroban contract address).
- **Freighter**: The standard non-custodial browser extension wallet for signing transactions and interacting with Stellar smart contracts.

---

## 2. Music Royalty & Streaming Mechanics

- **Basis Points (BPS)**: A unit of measure equal to 1/100th of 1% (0.01%). In Stellar Music, 10,000 basis points equals 100% split allocation.
- **0-Stroop Precision Loss Invariant**: A formal protocol invariant guaranteeing that during multi-recipient royalty splits, zero stroops are discarded or lost to integer truncation; remainder stroops are deterministically allocated to the designated primary artist.
- **Split Agreement**: An on-chain cryptographic registry mapping a track identifier and content hash to a vector of recipient Stellar accounts with proportional basis point weights.
- **Music Pass**: An on-chain access token purchased by a fan granting streaming access, collector stems, or VIP tier privileges for a specified duration or track catalogue.
- **Master Rights**: The copyright ownership of a recorded sound recording (as opposed to the underlying composition/publishing copyright).
- **Settlement Window**: The elapsed time between a user playback event and the verifiable on-chain distribution of funds to artist wallets (typically 3–5 seconds on Stellar).

---

## 3. Protocol Architecture & Infrastructure

- **Horizon**: The RESTful HTTP API server that enables applications to interact with the Stellar Core consensus engine.
- **Soroban RPC**: The JSON-RPC endpoint through which dApps simulate, submit, and inspect Soroban smart contract invocations.
- **Circuit Breaker**: An emergency administration control (`set_paused`) that suspends financial state changes in the event of an identified security anomaly or network fork.
- **Server-Sent Events (SSE)**: A real-time unidirectional streaming HTTP protocol used by `stellar-music-backend` to broadcast instantaneous settlement confirmations to listening frontend clients.
- **IPFS (InterPlanetary File System)**: Content-addressed peer-to-peer decentralized storage used to store lossless audio stems and metadata immutably.
