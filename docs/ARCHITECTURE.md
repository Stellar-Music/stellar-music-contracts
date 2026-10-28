# Stellar Music Protocol — System Architecture Specification

## 1. Executive Summary

**Stellar Music** is a decentralized, streaming-native music monetization platform built on the Stellar Network and Soroban smart contracts. It replaces opaque, slow traditional streaming royalty distribution with direct, peer-to-peer artist patronage, non-custodial music passes, and real-time on-chain revenue settlement.

---

## 2. C4 Context Diagram (System In Environment)

```mermaid
graph TD
    classDef person fill:#0284c7,stroke:#0369a1,stroke-width:2px,color:#ffffff;
    classDef system fill:#059669,stroke:#047857,stroke-width:2px,color:#ffffff;
    classDef ext fill:#475569,stroke:#334155,stroke-width:2px,color:#ffffff;

    Listener["🎧 Listener / Fan<br>(Discovers tracks, buys passes)"]:::person
    Artist["🎤 Creator / Artist<br>(Uploads music, creates split agreements)"]:::person
    Collaborator["🎹 Co-Writer / Producer<br>(Approves on-chain splits, receives revenue)"]:::person

    StellarMusic["🎵 Stellar Music Protocol<br>(Web App, Backend Orchestration, Smart Contracts)"]:::system

    StellarNetwork["⭐ Stellar Network & Soroban<br>(Immutable ledgers, SAC tokens, split execution)"]:::ext
    IPFS["📦 IPFS Decentralized Storage<br>(Immutable audio chunks & album cover metadata)"]:::ext
    FreighterWallet["👛 Freighter / xBull Wallet<br>(Key management, transaction signing)"]:::ext

    Listener -->|Browses catalog, signs pass purchases| StellarMusic
    Artist -->|Registers tracks, configures royalty terms| StellarMusic
    Collaborator -->|Cryptographically signs terms, claims earnings| StellarMusic

    StellarMusic -->|Settles atomic payments, checks state| StellarNetwork
    StellarMusic -->|Retrieves encrypted audio streams| IPFS
    Listener -->|Signs transactions| FreighterWallet
    Artist -->|Signs transactions| FreighterWallet
    Collaborator -->|Signs split terms| FreighterWallet
```

---

## 3. C4 Container Diagram (Internal Component Topography)

```mermaid
graph TB
    subgraph Client["Web Frontend Container (React 19 + TypeScript + Vite)"]
        SPA["Single Page Application<br>• Tailwind CSS & Lucide Icons<br>• StrKey & ed25519 input validators<br>• Real-time SSE settlement dashboard<br>• Offline connectivity banner"]
        PlayerBar["Audio Engine & Mini Player<br>• Range request chunking buffer<br>• Playback rate controller (0.75x - 1.5x)<br>• Dynamic audio waveform visualization"]
        WalletModal["Wallet Connector<br>• Freighter API adapter<br>• Instant testnet keypair generator"]
    end

    subgraph Backend["Application & Stream Orchestrator (Node.js 20 + Express)"]
        API["REST & SSE Controller<br>• Helmet security headers<br>• CORS & Rate limiting<br>• Request ID correlation tracing"]
        StreamService["Streaming Token Engine<br>• HMAC-SHA256 signature issuer<br>• HTTP 206 Partial Content range parser<br>• Monotonic play counter"]
        FailoverPool["Horizon RPC Pool<br>• Primary & fallback node failover<br>• Automatic transaction verification"]
        Database[("PostgreSQL 16 / SQLite<br>• Splits & approval receipts<br>• Settlement ledgers<br>• Track & artist metadata")]
    end

    subgraph Blockchain["Stellar Network / Soroban Smart Contracts (Rust v22)"]
        ContractCore["StellarMusicContract<br>• Track registration registry<br>• Non-custodial music pass state<br>• 10,000 basis points split validation<br>• Emergency circuit breaker & upgradeability"]
        SettlementEngine["Settlement Engine<br>• 0-Stroop precision loss invariant<br>• Deterministic dust absorption<br>• Atomic SAC token batch payouts"]
    end

    SPA -->|REST API Requests & SSE Live Feeds| API
    PlayerBar -->|Authenticated Range Requests| StreamService
    API -->|Reads / Writes persistent records| Database
    API -->|Probes transaction proofs| FailoverPool
    FailoverPool -->|Queries state & ledgers| ContractCore
    WalletModal -->|Direct contract invocation| ContractCore
    ContractCore -->|Triggers atomic payouts| SettlementEngine
```

---

## 4. Key Architectural Guarantees

1. **Non-Custodial Escrow**: Neither the server nor the artists ever take unilateral custody of listener subscription funds. Smart contracts enforce deterministic payment routing.
2. **Deterministic Stroop Arithmetic**: 1 XLM = 10,000,000 stroops. Indivisible dust is absorbed by index 0, guaranteeing zero rounding leakage across any number of contributors.
3. **Stateless Low-Latency Streaming**: Streaming authorization utilizes short-lived HMAC-SHA256 tokens validated in $O(1)$ memory without per-chunk database roundtrips.
4. **Defensive RPC Resilience**: Dual-endpoint Horizon RPC pooling shields client and server operations from single-node rate limits or network dropouts.

