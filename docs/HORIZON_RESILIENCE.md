# Stellar Music Protocol — Horizon RPC Failover & Network Resilience Specification

## 1. Resilience Philosophy

Decentralized applications cannot rely on a single RPC gateway. If a public Horizon endpoint encounters rate limits (HTTP 429) or maintenance downtime (HTTP 503), client operations and settlement listeners must not fail.

`stellar-music` implements an active-passive **Horizon RPC Failover Pool** across frontend and backend services.

---

## 2. Pool Configuration & Fallback Hierarchy

```
[Incoming Request / Transaction Probe]
                 │
                 ▼
      ┌───────────────────────┐
      │ Primary Horizon Node  │ ──► [Success] ──► Return Result
      │ (horizon-testnet)     │
      └──────────┬────────────┘
                 │ (Timeout / 503 / Network Error)
                 ▼
      ┌───────────────────────┐
      │ Fallback Horizon Node │ ──► [Success] ──► Return Result + Mark Primary Degraded
      │ (rpc-testnet)         │
      └──────────┬────────────┘
                 │ (Unreachable)
                 ▼
      ┌───────────────────────┐
      │ Standalone Local Node │ ──► Local Simulation Mode
      └───────────────────────┘
```

---

## 3. Failover Trigger Conditions

The RPC pool intercepts errors and initiates automatic failover under the following exact conditions:

1. **HTTP Status Codes**: `500 Internal Server Error`, `502 Bad Gateway`, `503 Service Unavailable`, `504 Gateway Timeout`.
2. **Network Exceptions**: `ECONNREFUSED`, `ETIMEDOUT`, `ENOTFOUND`, `FetchError`.
3. **Latency SLA Exceeded**: Request duration exceeding configurable threshold ($> 5,000\text{ms}$).

---

## 4. Cooldown & Auto-Recovery Protocol

- When an endpoint fails, it is placed in a **60-second cooldown period**.
- All subsequent calls immediately route to healthy endpoints without latency penalty.
- A background health check queries `GET /` of the degraded node every 30 seconds.
- Upon 3 consecutive successful health probes, the primary endpoint is restored to active status.
