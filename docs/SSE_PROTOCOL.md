# Stellar Music Protocol — Real-Time SSE Settlement Stream Protocol Specification

## 1. Protocol Architecture

To deliver live financial transparency to artists, contributors, and listeners, the Stellar Music backend exposes a **Server-Sent Events (SSE)** endpoint at `/api/settlements/stream` (with alias `/api/events`).

---

## 2. HTTP Connection & Headers

```http
GET /api/settlements/stream HTTP/1.1
Host: api.stellar-music.org
Accept: text/event-stream
```

### Response Headers
```http
HTTP/1.1 200 OK
Content-Type: text/event-stream; charset=utf-8
Cache-Control: no-cache, no-transform
Connection: keep-alive
X-Accel-Buffering: no
Access-Control-Allow-Origin: *
```

---

## 3. Keep-Alive Ping & Reverse Proxy Drop Defense

Many reverse proxies (Nginx, AWS ALB, Cloudflare) terminate idle HTTP streaming sockets after 30 to 60 seconds of inactivity. To prevent premature disconnects, the server transmits a periodic comment keepalive every **15 seconds**:

```
: ping - keep-alive heartbeat at 2026-10-28T10:00:00.000Z
```

---

## 4. Event Types & Schemas

### 4.1 `SETTLEMENT_EXECUTED`
Emitted immediately following successful on-chain revenue distribution:

```json
event: settlement_executed
data: {
  "type": "SETTLEMENT_EXECUTED",
  "settlement": {
    "id": "c1f7b029-7981-4ebc-a81d-669df0c3453b",
    "track_id": "1",
    "agreement_version": 1,
    "gross_amount": 50.0,
    "asset": "XLM",
    "tx_hash": "32408fc621838119d27a89f3fbd968201751582b7363943c97d226cdbe4249a6",
    "settled_at": "2026-10-28T10:00:00Z"
  },
  "recipients": [
    {
      "wallet_address": "GASTRALDRIFT7777777777777777777777777777777777777777",
      "role": "artist",
      "share_basis_points": 7000,
      "actual_amount": 35.0
    },
    {
      "wallet_address": "GBPRODUCER8888888888888888888888888888888888888888",
      "role": "producer",
      "share_basis_points": 3000,
      "actual_amount": 15.0
    }
  ]
}
```

---

## 5. Client Reconnection & Resiliency

Clients instantiate an `EventSource`:
```javascript
const es = new EventSource('/api/settlements/stream');
es.addEventListener('settlement_executed', (e) => {
  const data = JSON.parse(e.data);
  updateDashboard(data);
});
```
The server emits `retry: 3000`, directing clients to automatically reconnect after 3 seconds upon unexpected disconnects.
