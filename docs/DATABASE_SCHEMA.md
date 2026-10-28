# Stellar Music Protocol — Database Schema & Entity Relationship Specification

## 1. Entity-Relationship Diagram (ERD)

```mermaid
erDiagram
    ARTIST ||--o{ TRACK : creates
    TRACK ||--o{ MUSIC_PASS : receives
    TRACK ||--o{ SPLIT_AGREEMENT : governs
    SPLIT_AGREEMENT ||--|{ SPLIT_CONTRIBUTOR : defines
    SPLIT_AGREEMENT ||--o{ SPLIT_SIGNATURE : receives
    SPLIT_AGREEMENT ||--o{ SETTLEMENT : settles
    REVENUE_POOL ||--o{ SETTLEMENT : funds
    SETTLEMENT ||--|{ SETTLEMENT_RECIPIENT : distributes

    ARTIST {
        string id PK
        string wallet_address UK
        string name
        string bio
        string avatar_url
        string created_at
    }

    TRACK {
        string id PK
        string artist_id FK
        string title
        string genre
        int duration_seconds
        float pass_price_xlm
        string audio_url
        string cover_image_url
        int play_count
        boolean is_active
        string created_at
    }

    MUSIC_PASS {
        string id PK
        string track_id FK
        string wallet_address
        float payment_amount
        string tx_hash UK
        string status
        string purchased_at
    }

    SPLIT_AGREEMENT {
        string id PK
        string track_id FK
        int version
        string agreement_hash
        string status
        string created_at
        string locked_at
    }

    SPLIT_CONTRIBUTOR {
        string id PK
        string agreement_id FK
        string wallet_address
        string role
        int share_basis_points
        float percentage
    }

    SPLIT_SIGNATURE {
        string id PK
        string agreement_id FK
        string wallet_address
        string signature_hash
        string signed_at
    }

    SETTLEMENT {
        string id PK
        string pool_id FK
        string track_id FK
        string agreement_id FK
        float gross_amount
        string asset
        string tx_hash UK
        string settled_at
    }

    SETTLEMENT_RECIPIENT {
        string id PK
        string settlement_id FK
        string wallet_address
        string role
        float expected_amount
        float actual_amount
        string status
    }

    REVENUE_POOL {
        string id PK
        string asset
        float total_amount
        float allocated_amount
        float unallocated_amount
        string status
    }
```

---

## 2. Relational Integrity & Performance Indexes

| Table | Index Name | Columns | Purpose |
| :--- | :--- | :--- | :--- |
| `artists` | `idx_artists_wallet` | `wallet_address` | $O(1)$ Artist profile resolution |
| `tracks` | `idx_tracks_artist` | `artist_id` | Catalog queries by creator |
| `music_passes` | `idx_passes_track_wallet` | `track_id, wallet_address` | Instant pass holding lookup |
| `music_passes` | `idx_passes_tx_hash` | `tx_hash` (Unique) | Anti-replay duplicate protection |
| `split_agreements` | `idx_splits_track_ver` | `track_id, version` | Versioned terms inspection |
| `settlements` | `idx_settlements_track` | `track_id` | Lifetime earnings reporting |
| `settlements` | `idx_settlements_tx_hash` | `tx_hash` (Unique) | On-chain settlement receipt linkage |
| `settlement_recipients`| `idx_recipients_wallet` | `wallet_address` | Collaborator aggregate balance queries |
