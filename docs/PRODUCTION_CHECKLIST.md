# Stellar Music Protocol — Mainnet Readiness & Production Checklist

This checklist documents the mandatory engineering gates and operational validations required prior to deploying `StellarMusicContract` and associated services to Stellar Public Mainnet.

---

## 1. Smart Contract Verification Gates

- [ ] **WASM Bytecode Optimization**:
  - Compiled with `cargo build --target wasm32-unknown-unknown --release`
  - Stripped and optimized with `soroban contract optimize`
  - Final size verified below strict budget threshold (`< 50 KB`, actual: 40.9 KB)
- [ ] **Automated Test Suite**:
  - Full test suite passes: `cargo test` (34 passing unit, invariant, and integration tests)
  - 0-stroop precision loss invariant verified across boundary cases
  - Boundary fuzzing test passes without unhandled panics
- [ ] **Storage Rent & TTL Management**:
  - Instance storage TTL bump logic implemented and tested
  - Contract state entry sizes strictly modeled within protocol maximums
  - Automated archival restoration procedures documented and tested
- [ ] **Access Control & Permissions**:
  - Multi-signature administration account configured with 3-of-5 threshold
  - No development or test keys hardcoded in contract binaries or deployment scripts
  - Circuit breaker emergency pause tested with unauthorized caller rejection

---

## 2. Infrastructure & RPC Connectivity

- [ ] **Primary & Failover RPC Nodes**:
  - Redundant Horizon and Soroban RPC providers configured (e.g., Stellar Development Foundation, QuickNode, Blockdaemon)
  - Automatic round-robin health probe and circuit breaking verified
  - Rate limiting handling (HTTP 429 backoff with exponential jitter) enabled
- [ ] **PostgreSQL Database Readiness**:
  - Production connection pooling configured with `max_connections` sized to worker pool
  - Automated WAL archiving and Point-In-Time Recovery (PITR) verified
  - Read-replica read/write splitting operational for analytical endpoints
- [ ] **Secret Management**:
  - No secrets stored in git repository (`scripts/audit-secrets.sh` returns exit code 0)
  - Cloud secret managers (AWS Secrets Manager / Vault) utilized for production private keys
  - Dynamic API key rotation schedule defined and automated

---

## 3. Frontend & Client Integrity

- [ ] **Client Build & Bundle Budget**:
  - Production build succeeds cleanly: `npm run build`
  - TypeScript strict mode typecheck clean: `npm run typecheck`
  - Static asset gzip compression enabled with immutable caching headers
- [ ] **Accessibility & Web Standards**:
  - WCAG 2.1 AA compliance verified with high-contrast focus rings and screen reader attributes
  - Reduced motion preferences honored via `prefers-reduced-motion` media queries
  - Lighthouse performance score > 90 across Mobile and Desktop
- [ ] **Wallet & StrKey Safety**:
  - Strict StrKey validation before dispatching any payment or contract invocation
  - Freighter, xBull, and Albedo wallet connectors tested on mainnet simulation
  - User-facing error notifications explain transaction failures in non-technical language

---

## 4. Operational Monitoring & Disaster Recovery

- [ ] **Real-Time Telemetry & Alerts**:
  - Prometheus / Grafana metrics tracking active streams, settlement latency, and RPC errors
  - PagerDuty / Opsgenie alert rules for contract event anomalies and Horizon disconnections
  - Sentry error reporting configured with client PII scrubbing
- [ ] **Runbooks & Incident Response**:
  - Multi-sig key replacement runbook tested in rehearsal
  - Contract upgrade procedure rehearsed on Stellar Testnet
  - Automated deployment verification script (`scripts/verify-deployment.sh`) executed post-launch
