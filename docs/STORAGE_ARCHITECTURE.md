# Stellar Music Protocol — Decentralized Media & Storage Architecture

## 1. Storage Topology

**Stellar Music** decouples on-chain financial accounting from high-bandwidth media storage. Smart contracts store verified state, ownership, and IPFS Content Identifiers (CIDs), while audio binaries and rich visual art reside across decentralized content-addressed networks.

```
┌─────────────────────────────────┐
│     Stellar Soroban Contract    │
│  - Track ID, Artist, Price      │
│  - metadata_uri (IPFS CID)      │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│        IPFS / Filecoin          │
│  - Album artwork (PNG/WebP)     │
│  - Track metadata (JSON Schema) │
│  - Lossless Master Audio        │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│     Edge Streaming Gateway      │
│  - HMAC-SHA256 Token Validation │
│  - HTTP 206 Partial Content     │
│  - Immutable Byte Range Caching │
└─────────────────────────────────┘
```

---

## 2. Track Metadata Standard (Schema v1.1)

Track metadata is hosted as immutable JSON referenced by `metadata_uri`:

```json
{
  "schema": "StellarMusic::TrackMetadata::v1.1",
  "name": "Astral Drift",
  "artist": "Stellar Drift",
  "artist_wallet": "GASTRALDRIFT7777777777777777777777777777777777777777",
  "genre": "Synthwave",
  "duration_seconds": 214,
  "audio_cid": "ipfs://bafybeiaudioastraldrift2026lossless",
  "cover_art_cid": "ipfs://bafybeicoverartastraldrift2026hd",
  "mime_type": "audio/mpeg",
  "sample_rate_hz": 44100,
  "bitrate_kbps": 320,
  "created_at": "2026-10-28T00:00:00Z"
}
```

---

## 3. HTTP 206 Partial Content Range Parsing & Performance

To enable instant seeking, playback scrubbing, and bandwidth optimization, the streaming server implements RFC 7233 byte-range serving:

1. **Client Request**: `Range: bytes=0-1048576` (Requesting initial 1MB chunk).
2. **Server Response**:
   - Status: `206 Partial Content`
   - Header: `Content-Range: bytes 0-1048576/15728640`
   - Header: `Accept-Ranges: bytes`
   - Header: `Cache-Control: public, max-age=31536000, immutable`
3. **Latency Profile**: Initial Time-to-First-Byte (TTFB) $< 15\text{ms}$ on cached local edge; $< 120\text{ms}$ on cold IPFS gateway fetch.
