# Stellar Music Protocol — Bug Bounty & Vulnerability Disclosure Policy

## 1. Scope & Guidelines

Stellar Music values ethical security research. We invite security auditors and white-hat researchers to discover and report vulnerabilities responsibly.

---

## 2. In-Scope Targets

| Target Component | Repository / Address | Impact Area |
| :--- | :--- | :--- |
| **Soroban Smart Contract** | `CBUNX6PF4OOFGQ434IBVCLRBR6OBQOM5PTYUES7X6KSN2AIER6VMJK7K` | Pass purchases, split agreements, settlements |
| **Settlement Math Engine** | `src/lib.rs` / `src/services/settlement.service.ts` | 0-stroop precision, remainder allocation |
| **Streaming Token HMAC** | `src/services/streamingToken.ts` | Timing-safe verification, replay attacks |

---

## 3. Reward Tiers (USD Value Equivalent in XLM / USDC)

| Severity | Reward Range | Qualifying Vulnerability Examples |
| :---: | :---: | :--- |
| **Critical** | **$2,500 – $5,000** | Direct theft of user tokens, contract drain, split agreement unauthorized mutation, arbitrary balance minting. |
| **High** | **$1,000 – $2,500** | Bypassing required collaborator signatures, circuit breaker override, permanent contract freeze. |
| **Medium** | **$250 – $1,000** | Denial of service in settlement batches, unbounded gas loops, unhandled panic conditions. |
| **Low** | **$50 – $250** | Non-financial logic discrepancies, minor telemetry leaks, edge case validation omissions. |

---

## 4. Responsible Disclosure Process

1. **Submit Report**: Email full reproduction details to `KingTaiwoDev` (`328718370+KingTaiwoDev@users.noreply.github.com`).
2. **Acknowledgment**: You will receive an initial response within **24 hours**.
3. **Remediation SLA**: Critical issues patched within **72 hours**; public disclosure embargoed for **30 days** post-fix.
