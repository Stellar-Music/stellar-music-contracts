# Contributing to Stellar Music Contracts

Thank you for your interest in contributing to **Stellar Music**! We welcome contributions to our Soroban smart contracts from developers of all backgrounds.

## Code of Conduct
Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all project interactions.

## Prerequisites
- **Rust Toolchain**: 1.80+ (`rustup target add wasm32-unknown-unknown`)
- **Stellar CLI**: `v22.0.0` or higher
- **Soroban SDK**: `v22.0.6`

## Development Workflow
1. Fork the repository and create your branch from `main`:
   ```bash
   git checkout -b feat/your-improvement-name
   ```
2. Build the contract locally:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```
3. Run the unit and integration tests:
   ```bash
   cargo test --lib
   ```
4. Verify code formatting and linting:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   ```
5. Commit with conventional commit messages:
   - `feat(contracts): ...`
   - `fix(contracts): ...`
   - `test(contracts): ...`
   - `docs(contracts): ...`

## Pull Request Guidelines
- Ensure all automated unit tests pass before submitting.
- Link your pull request to an existing open issue.
- Describe the exact financial invariant or security properties affected.

## Security Disclosures
If you discover a security vulnerability or exploit, please do not file a public issue. Refer to [SECURITY.md](SECURITY.md) for responsible disclosure instructions.
