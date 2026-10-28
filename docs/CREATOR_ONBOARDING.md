# Stellar Music Protocol — Creator & Artist Onboarding Manual

## 1. Overview for Music Creators

Stellar Music provides recording artists, producers, and independent labels with a direct, non-custodial pipeline to publish, monetize, and settle music royalties instantly on the Stellar blockchain.

---

## 2. Audio & Media Asset Specifications

### Lossless Master Audio
- **Accepted Formats**: FLAC, WAV, or high-bitrate MP3 (320 kbps).
- **Sample Rate**: 44.1 kHz (CD standard) or 48.0 kHz.
- **Bit Depth**: 16-bit or 24-bit.
- **Channels**: 2-channel Stereo.
- **True Peak Ceiling**: $-1.0\text{ dBTP}$ recommended to prevent lossy transcoding inter-sample clipping.

### Cover Artwork
- **Aspect Ratio**: 1:1 Square.
- **Minimum Resolution**: $1400 \times 1400\text{ px}$ (Optimal: $3000 \times 3000\text{ px}$).
- **File Format**: WebP, PNG, or high-quality JPEG.

---

## 3. Step-by-Step Publishing Flow

1. **Connect Stellar Wallet**: Log in with Freighter Wallet or create an instant test account.
2. **Register Track**: Submit title, genre, pass price in XLM/USDC, and audio master.
3. **Configure Revenue Splits in Split Studio**:
   - Add all co-writers, vocalists, and producers by their ed25519 Stellar public keys (`G...`).
   - Assign percentage cuts ensuring total equals exactly **100.00%** (10,000 basis points).
   - Generate canonical terms hash and submit `create_split_agreement`.
4. **Collect Signatures**: Share the track link with collaborators. When each contributor signs, the agreement automatically locks.
5. **Instant Streaming Royalties**: Every time a listener purchases a pass, payments route directly to collaborator wallets in seconds.
