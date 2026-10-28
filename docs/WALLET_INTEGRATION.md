# Stellar Music Protocol — Web3 Wallet Integration & StrKey Validation Guide

## 1. Supported Non-Custodial Wallets

Stellar Music supports non-custodial browser extensions and standalone key management:

1. **Freighter Wallet (Recommended)**: Official browser extension from Stellar Development Foundation.
2. **xBull Wallet**: Multi-chain, privacy-focused Stellar extension.
3. **Albedo**: Web-based instant signing without browser extension requirements.
4. **Instant Testnet Generator**: Built-in developer keypair derivation tool for rapid hackathon testing and demo flows.

---

## 2. Stellar Public Key Validation (StrKey RFC)

Stellar public keys are encoded using base32 RFC 4648 with a CRC-16 checksum:

- **Prefix**: Always starts with uppercase `G` (ed25519 public key identifier).
- **Length**: Exactly 56 characters.
- **Alphabet**: Base32 characters `[A-Z2-7]`.
- **Validation Regular Expression**: `^G[A-Z2-7]{55}$`.

Frontend and backend validators utilize `@stellar/stellar-sdk` `StrKey.isValidEd25519PublicKey(address)` to prevent misdirected funds prior to transaction creation.

---

## 3. Freighter Wallet Connection Protocol

```typescript
import { isConnected, getPublicKey, signTransaction } from '@stellar/freighter-api';

export async function connectFreighter(): Promise<string> {
  const connected = await isConnected();
  if (!connected) {
    throw new Error('WALLET_NOT_INSTALLED: Please install Freighter Wallet extension.');
  }

  const publicKey = await getPublicKey();
  if (!publicKey || !publicKey.startsWith('G')) {
    throw new Error('UNAUTHORIZED_KEY: No authorized public key returned from Freighter.');
  }

  return publicKey;
}
```

---

## 4. Testnet Friendbot Instant Keypair Faucet

For judges and reviewers without a pre-existing wallet, the application provides an integrated credential generator:

```typescript
import { Keypair } from '@stellar/stellar-sdk';

export async function createFundedTestAccount(): Promise<{ publicKey: string; secret: string }> {
  const pair = Keypair.random();
  // Request 10,000 testnet XLM from Stellar Friendbot
  await fetch(`https://friendbot.stellar.org?addr=${pair.publicKey()}`);
  return {
    publicKey: pair.publicKey(),
    secret: pair.secret(),
  };
}
```
