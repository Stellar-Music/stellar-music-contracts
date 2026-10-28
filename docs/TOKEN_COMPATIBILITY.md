# Stellar Music Protocol — Cross-Contract Soroban Token Compatibility Matrix

## 1. Supported Token Standards

The `StellarMusicContract` interfaces with tokens via the official **Soroban Token Interface (SEP-41)**. Payments for music passes and automated revenue settlements support both Stellar Classic assets (wrapped via the Stellar Asset Contract) and native Soroban custom tokens.

---

## 2. Asset Compatibility Matrix

| Asset Symbol | Asset Type | Implementation Standard | Pass Purchase | Revenue Settlement |
| :--- | :--- | :--- | :---: | :---: |
| **XLM** | Native Stellar Asset | Native SAC Contract | ✅ Supported | ✅ Supported |
| **USDC** | Fiat-backed Stablecoin | Wrapped SAC (Circle Issuer) | ✅ Supported | ✅ Supported |
| **EURC** | Euro-backed Stablecoin | Wrapped SAC (Circle Issuer) | ✅ Supported | ✅ Supported |
| **Custom SEP-41**| Custom Smart Contract | Pure Soroban SEP-41 | ✅ Supported | ✅ Supported |

---

## 3. SEP-41 Interface Methods Utilized

The protocol interacts with token contracts via `soroban_sdk::token::Client`:

```rust
// Transfer tokens from listener to artist during pass acquisition
client.transfer(&listener, &track.artist, &track.pass_price);

// Transfer tokens atomically to each collaborator during settlement
client.transfer(&caller, &payout.recipient, &payout.amount);
```

### Required Token Capabilities
- `transfer(from: Address, to: Address, amount: i128)`
- `balance(id: Address) -> i128`

Tokens with arbitrary transfer fees (deflationary / fee-on-transfer) are supported by the integer stroop engine, though non-fee standard tokens (XLM, USDC) are recommended for exact 10,000 bps payouts.
