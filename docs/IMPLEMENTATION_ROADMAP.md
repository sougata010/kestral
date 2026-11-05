# Kestrel Browser — Implementation & Engineering Roadmap

This document outlines the phased engineering roadmap for Kestrel:

- **Phase 1: Native Rust Core & Window Shell**: Tauri v2 scaffolding, window layout, and native tab lifecycle supervisor (`WebviewBuilder`).
- **Phase 2: Security & Network Interception**: Brave's `adblock` Rust crate, URL parameter stripping, and cookie entropy/payload inspector.
- **Phase 3: Threat ML & Cryptographic Vault**: Bloom filter + local ONNX ML URL classifier, `chacha20poly1305` + `keyring` credential vault with user consent prompts.
- **Phase 4: Developer Tools & Cyber Intel Radar**: CDP stream, Wappalyzer framework detector, live request/response topology graph, and CVE radar.
- **Phase 5: Privacy Networking & Media Engine**: Userspace WireGuard (`wireproxy`) integration, multi-hop SOCKS5 chaining, fingerprint jitter, and `yt-dlp` background audio player.
- **Phase 6: Resource Manager & Final Polish**: Tab hibernation, memory watchdog, and Windows installer generation.

