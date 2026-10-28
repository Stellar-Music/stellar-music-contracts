# Stellar Music Protocol — Governance & Multi-Signature Specification

## 1. Principles of Protocol Governance

The **Stellar Music Protocol** transitions administrative authority from a single lead developer during early testnet phases to a decentralized, multi-signature community council for Mainnet operations.

---

## 2. Multi-Signature Key Architecture (Mainnet Standard)

- **Admin Account**: 3-of-5 threshold multi-signature account.
- **Key Holders**: Composed of independent music artists, core protocol developers, and Stellar ecosystem maintainers.
- **Weight Requirements**:
  - Master Weight: 0 (Direct master key permanently revoked post-deployment)
  - Signer Weights: 1 per keyholder
  - Low Threshold (Queries & Metadata): 1
  - Medium Threshold (Pause / Unpause Circuit Breaker): 2
  - High Threshold (Contract Upgrade / Admin Key Transfer): 3

---

## 3. Protocol Upgrade Policy & Timelock Delay

1. **Non-Emergency Upgrades**: Must undergo a **48-hour timelock delay** after on-chain multi-signature threshold is achieved, giving users and node operators time to audit proposed WASM bytecode hashes.
2. **Emergency Circuit Breaker**: The `set_paused` function can execute immediately with 2-of-5 signer approval to protect funds against active exploits.

---

## 4. Stellar Music Improvement Proposals (SMIPs)

Protocol parameter adjustments (e.g. `MAX_PLATFORM_FEE_BPS`, storage TTL parameters) must follow the SMIP process:
1. **Idea / Discussion**: GitHub Discussion thread with technical specification.
2. **Draft SMIP**: Pull request against the protocol repository with test harness additions.
3. **Review & Voting**: 7-day community review period followed by Multi-Sig Council on-chain signature.
