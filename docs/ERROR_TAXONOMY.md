# Stellar Music Protocol — API Error Taxonomy & Standardized Envelopes

## 1. Unified Error Response Envelope

All API endpoints strictly format non-2xx responses using a predictable JSON envelope:

```json
{
  "error": "Descriptive message intended for client presentation",
  "code": "MACHINE_READABLE_ERROR_CODE",
  "requestId": "3a7b9c1d-1234-5678-90ab-cdef01234567",
  "timestamp": "2026-10-28T10:00:00.000Z",
  "details": [
    {
      "field": "share_basis_points",
      "message": "Total basis points must sum to exactly 10,000"
    }
  ]
}
```

---

## 2. Error Code Taxonomy

| HTTP Status | Error Code (`code`) | Trigger Description |
| :---: | :--- | :--- |
| **400** | `INVALID_INPUT` | Zod schema validation failed on request body or parameters |
| **400** | `INVALID_STRKEY` | Provided address is not a valid ed25519 Stellar public key (`G...`) |
| **400** | `INVALID_BASIS_POINTS` | Collaborator split percentages do not total 100.00% (10,000 bps) |
| **401** | `UNAUTHORIZED` | Request lacks required cryptographic signature or session header |
| **403** | `STREAM_TOKEN_EXPIRED` | Audio HMAC streaming token timestamp has elapsed |
| **403** | `PASS_REQUIRED` | Listener does not hold an active Music Pass for the requested track |
| **404** | `TRACK_NOT_FOUND` | Track entity does not exist in local database or on-chain storage |
| **404** | `AGREEMENT_NOT_FOUND` | Split agreement ID or version does not exist |
| **409** | `SPLIT_ALREADY_LOCKED` | Attempted modification of an immutable, fully signed split agreement |
| **409** | `DUPLICATE_PASS` | Listener already owns a valid pass for this track (prevent double-spend) |
| **416** | `RANGE_NOT_SATISFIABLE`| Requested byte range offsets exceed total audio file size |
| **502** | `HORIZON_RPC_ERROR` | Upstream Stellar Horizon node returned an unexpected network failure |
| **503** | `SERVICE_PAUSED` | Protocol circuit breaker is active; operations temporarily frozen |
