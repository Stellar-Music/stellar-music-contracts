# Stellar Music Protocol — Multi-Signature Key Management Specification

## 1. Multi-Signature Architecture Overview

To eliminate single-point-of-failure risks in production, the `StellarMusicContract` administrative privileges are anchored to a **Stellar Native Multi-Signature Account**.

Unlike EVM networks that require complex, expensive external smart contract wallets (e.g. Gnosis Safe), the Stellar Network implements multi-signature security natively at the protocol level with zero contract overhead.

---

## 2. Threshold & Signer Configuration

```
                         ┌────────────────────────────────────────┐
                         │   Stellar Music Multi-Sig Admin Root   │
                         │   GCMMDSL... (Master Weight = 0)       │
                         └───────────────────┬────────────────────┘
                                             │
      ┌──────────────┬──────────────┬────────┼──────────────┬──────────────┐
      ▼              ▼              ▼                       ▼              ▼
[ Signer 1 ]   [ Signer 2 ]   [ Signer 3 ]            [ Signer 4 ]   [ Signer 5 ]
(Weight = 1)   (Weight = 1)   (Weight = 1)            (Weight = 1)   (Weight = 1)
```

### Threshold Levels:
| Threshold Level | Value | Required Signatures | Controlled Operations |
| :---: | :---: | :---: | :--- |
| **Low** | `1` | 1 of 5 | Non-financial read queries, storage TTL extensions |
| **Medium** | `2` | 2 of 5 | Emergency circuit breaker pause (`set_paused`), fee config (`set_platform_fee`) |
| **High** | `3` | 3 of 5 | Contract bytecode upgrade (`upgrade`), admin key transfer (`set_admin`) |

---

## 3. Key Compromise & Replacement Procedure

If a signer key is compromised or a keyholder leaves the protocol:
1. Remaining keyholders coordinate to assemble a High-threshold transaction (3 signatures).
2. The transaction invokes `set_options` to remove the compromised signer (`weight: 0`) and register a replacement signer key (`weight: 1`).
3. Contract administrator state remains uninterrupted.
