# Kestrel Browser — Security & Privacy Deep Dive

## 1. Malicious Cookie & Storage Inspection (REQ-01)
- **Shannon Entropy Analysis**: Flags cookies with $H(X) > 4.8$ as potential steganographic trackers or opaque cross-domain tracking IDs.
- **Payload Signature Scanning**: Evaluates cookies for SQLi, XSS, and deserialization exploit vectors.
- **Tracking Radar**: Cross-references cookies against the DuckDuckGo Tracker Radar dataset.

## 2. Real-Time URL Threat & ML Intercept Engine (REQ-02)
- **Tier 1 (Instant Bloom Filter)**: Sub-millisecond lookup against known malicious/phishing URLs (PhishTank/URLhaus).
- **Tier 2 (Local ML Inference)**: Lexical URL feature extraction analyzed via quantized local model in < 5ms.
- **Tier 3 (Cloud Fallback)**: Google Safe Browsing / ExposedDB query for borderline confidence scores.

## 3. Brave Adblock Engine (REQ-05)
- Uses Brave's native `adblock` Rust crate for high-throughput rule matching.
- Network rule cancellation + CSS cosmetic hiding injection.

## 4. Cryptographic Vault (REQ-06)
- Master key derived via Argon2id.
- Encrypted with `ChaCha20-Poly1305` AEAD.
- Hardware-backed master key protection via OS credential store (`keyring`).
- Explicit user prompt / PIN validation before releasing credentials.

## 5. Userspace WireGuard VPN (REQ-08)
- Employs `wireproxy` (WireGuard userspace stack) on local loopback (`127.0.0.1:<PORT>`).
- Zero administrative / UAC elevation needed.

## 6. Tor-Like Circuit Privacy (REQ-12)
- Multi-hop SOCKS5 chaining for IP dissociation.
- Strict isolation from `.onion` hidden services.
- Canvas, WebGL, and AudioContext fingerprint noise injection.

