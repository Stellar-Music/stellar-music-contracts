# Stellar Music Protocol — Audio Streaming HMAC Token Cryptographic Specification

## 1. Overview & Threat Model

Streaming audio high-fidelity media requires strict access control without introducing database query bottlenecks on every chunk request.

Traditional database session checks on every 64KB audio chunk would saturate database connection pools. Stellar Music resolves this via **stateless, time-bounded HMAC-SHA256 streaming tokens**.

---

## 2. Token Format & Payload

A Stellar Music streaming token is an encoded string formatted as:

```text
base64url(trackId : walletAddress : expiresAt : hmacSignature)
```

| Parameter | Type | Description |
| :--- | :--- | :--- |
| `trackId` | `string` | Unique track identifier |
| `walletAddress` | `string` | Listener's ed25519 Stellar public key (`G...`) |
| `expiresAt` | `number` | Unix timestamp in seconds when authorization lapses |
| `hmacSignature` | `hex` | SHA-256 HMAC digest computed with the server secret |

---

## 3. Cryptographic Verification & Timing Attack Defense

Verification proceeds in four sequential validation gates:

1. **Format Validation**: Base64url decoded payload must split into exactly 4 colon-delimited components.
2. **Temporal Validation**: Current server time $\le \text{expiresAt}$. Expired tokens immediately return HTTP 403 `Streaming token expired`.
3. **Digest Recreation**: Recompute expected signature over `trackId:walletAddress:expiresAt` using the server HMAC secret key.
4. **Constant-Time Comparison**:
   ```javascript
   const valid = crypto.timingSafeEqual(
     Buffer.from(signature, 'hex'),
     Buffer.from(expectedSignature, 'hex')
   );
   ```
   *Rationale:* `crypto.timingSafeEqual` prevents execution time variance from leaking valid signature bytes to attackers.

---

## 4. Performance Profile

- **Token Generation**: $< 0.05\text{ms}$ per token.
- **Token Verification**: $< 0.02\text{ms}$ per request ($O(1)$ CPU, zero DB queries).
- **Throughput Capability**: Exceeds $20,000\text{ req/sec}$ per Node.js worker process.
