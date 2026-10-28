# Stellar Music Protocol — Maintainers & Contribution Policy

## 1. Core Maintainers Roster

| Name / GitHub Handle | Role | Primary Focus |
| :--- | :--- | :--- |
| **KingTaiwoDev** (`@KingTaiwoDev`) | Lead Maintainer | Core Protocol, Smart Contracts & Architecture |
| **Stellar-Music Core** (`@Stellar-Music`) | Protocol Council | Ecosystem Relations, Security & Infrastructure |

---

## 2. Commit Standards (Conventional Commits)

All commits across `stellar-music-contracts`, `stellar-music-backend`, and `stellar-music-frontend` must follow the Conventional Commits specification:

```
<type>(<scope>): <short imperative summary>

[optional detailed description explaining what and why]
```

### Approved Types:
- `feat`: New user-facing feature or contract function
- `fix`: Bug fix in contract logic, backend route, or frontend UI
- `test`: Addition or modification of unit, integration, or invariant tests
- `docs`: Documentation, architecture diagrams, or specifications
- `ci`: GitHub Actions workflows, build scripts, or deployment pipelines
- `chore`: Dependency updates, tooling configurations, or housekeeping
- `style`: Formatting, whitespace, or accessibility styling adjustments

---

## 3. Pull Request Review & Merge Standards

A pull request can only merge into `main` when:
1. **Automated CI Gates Pass**:
   - `cargo fmt --check` / `prettier --check .`
   - `cargo clippy -- -D warnings` (zero linter warnings)
   - `cargo test` / `npm test` (100% test pass rate)
   - WASM binary size verified $< 50\text{ KB}$ (`scripts/check-wasm-size.sh`)
2. **Deterministic Author Identity**: Signed commits authored by authorized maintainers.
3. **No Bundle Commits**: Atomic commits detailing specific changes.
