# Stellar Music Protocol — End-to-End Sequence Flow Specifications

This document outlines the end-to-end cryptographic and financial sequence flows governing **Stellar Music**.

---

## 1. Flow 1: Music Pass Purchase & Streaming Session Flow

```mermaid
sequenceDiagram
    autonumber
    actor Listener as 🎧 Listener
    participant UI as 💻 React Frontend
    participant Wallet as 👛 Freighter Wallet
    participant Soroban as ⭐ Soroban Contract
    participant Backend as ⚙️ Backend API
    participant Storage as 📦 Media Storage / CDN

    Listener->>UI: Clicks "Purchase Music Pass"
    UI->>Wallet: Requests signature for `purchase_pass(track_id, token)`
    Wallet-->>Listener: Displays transaction parameters & token cost
    Listener->>Wallet: Confirms and signs transaction
    Wallet->>Soroban: Submits contract call to Stellar Network
    Soroban->>Soroban: Validates price >= MIN_PASS_PRICE, transfers SAC tokens to artist
    Soroban->>Soroban: Writes PassInfo record to storage, increments total_passes
    Soroban-->>UI: Transaction confirmed on-chain (Tx Hash + PassInfo)

    UI->>Backend: POST /api/streaming/token { trackId, walletAddress, txHash }
    Backend->>Soroban: Verifies on-chain pass validity via `has_valid_pass`
    Backend->>Backend: Issues HMAC-SHA256 streaming token (TTL: 3600s)
    Backend-->>UI: Returns { token, expiresAt }

    UI->>Storage: GET /api/streaming/stream/:id (Headers: Range: bytes=0-1048576, Bearer token)
    Storage->>Storage: Validates HMAC signature in O(1) time
    Storage-->>UI: 206 Partial Content (Audio Byte Chunks)
    UI->>Listener: Seamless low-latency audio playback starts
```

---

## 2. Flow 2: Multi-Party Split Proposal, Signing & On-Chain Locking

```mermaid
sequenceDiagram
    autonumber
    actor Artist as 🎤 Primary Artist
    actor Producer as 🎹 Producer
    actor Writer as ✍️ Songwriter
    participant UI as 💻 Split Studio UI
    participant Soroban as ⭐ Soroban Contract
    participant Backend as ⚙️ Backend DB

    Artist->>UI: Configures split basis points (5000 bps, 3000 bps, 2000 bps = 10000 bps)
    UI->>UI: Generates canonical JSON & computes deterministic SHA-256 agreement hash
    Artist->>Soroban: Calls `create_split_agreement(track_id, hash, recipients)`
    Soroban->>Soroban: Validates sum == 10000 bps, creates DRAFT agreement
    Soroban-->>UI: Agreement version created on-chain

    UI->>Backend: Syncs split proposal state for collaborator notifications
    Backend-->>Producer: Real-time SSE alert: "Split agreement proposal pending signature"
    Backend-->>Writer: Real-time SSE alert: "Split agreement proposal pending signature"

    Producer->>UI: Inspects canonical agreement hash and contributor roles
    Producer->>Soroban: Calls `approve_split_agreement(track_id, version, hash)`
    Soroban->>Soroban: Verifies hash match & contributor identity, adds approval

    Writer->>UI: Inspects and approves terms
    Writer->>Soroban: Calls `approve_split_agreement(track_id, version, hash)`
    Soroban->>Soroban: All required recipients have approved (3/3)
    Soroban->>Soroban: Sets `is_locked = true` and updates `ActiveLockedSplit`
    Soroban-->>UI: Emits `(split, locked)` event
```

---

## 3. Flow 3: Automated Revenue Settlement & 0-Stroop Distribution

```mermaid
sequenceDiagram
    autonumber
    participant Engine as 🤖 Automated Settlement Worker
    participant Soroban as ⭐ Soroban Contract
    participant SAC as 🪙 Stellar Asset Contract
    actor Artist as 🎤 Artist (50%)
    actor Producer as 🎹 Producer (30%)
    actor Writer as ✍️ Songwriter (20%)
    participant SSE as 📡 SSE Live Stream

    Engine->>Soroban: Calls `execute_split_settlement(track_id, gross_amount, token)`
    Soroban->>Soroban: Verifies circuit breaker is NOT paused
    Soroban->>Soroban: Retrieves ActiveLockedSplit agreement
    Soroban->>Soroban: Computes integer stroop shares: (gross * bps) / 10000
    Soroban->>Soroban: Assigns remainder dust to primary artist (index 0)

    par Atomic Token Transfers
        Soroban->>SAC: transfer(caller, artist, artist_stroops)
        SAC-->>Artist: Account credited
    and
        Soroban->>SAC: transfer(caller, producer, producer_stroops)
        SAC-->>Producer: Account credited
    and
        Soroban->>SAC: transfer(caller, writer, writer_stroops)
        SAC-->>Writer: Account credited
    end

    Soroban->>Soroban: Records SettlementReceipt, increments settlement_id
    Soroban-->>Engine: Returns SettlementReceipt
    Engine->>SSE: Broadcasts `SETTLEMENT_EXECUTED` event to connected dashboards
```
