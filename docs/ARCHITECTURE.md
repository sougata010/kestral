# Kestrel Browser — System Architecture Specification

## 1. Architectural Philosophy: The Native Rust Systems Model

Modern native browsers (Brave, Chrome, Edge) are built using a hybrid systems approach:
- **Systems Core (Rust)**: High-throughput network interception, Brave adblocking engine (`adblock-rs`), local ML inference, cryptographic memory safety, packet routing, and tab lifecycle supervision.
- **Rendering Engine**: OS-native hardware-accelerated Chromium (via Microsoft WebView2 on Windows / WebKit on Linux/macOS).
- **User Interface & DevTools**: GPU-accelerated HTML/CSS/TypeScript rendered by the system compositor and compiled directly into the single Rust `.exe` binary.

Kestrel achieves native C/Rust throughput where it matters while retaining modern UI flexibility and extension compatibility.

```mermaid
graph TD
    UI["Browser Chrome UI (TypeScript / React)"]
    DevTools["Kestrel Advanced DevTools & Cyber Intel"]
    Media["Background Audio & Downloader Service"]
    
    subgraph "Kestrel Core (Rust / Tauri v2)"
        IPC["Typed IPC Router (Rust Commands)"]
        SessionMgr["Multi-Profile & Session Partition Manager"]
        TabMgr["Tab Lifecycle & Resource Manager (WebviewBuilder)"]
    end
    
    subgraph "Native Rust Security Subsystem"
        AdBlock["adblock crate (Brave Engine)"]
        URLScanner["Local ML Inference & Bloom Filter Engine"]
        CookieAnalyzer["Entropy & Threat Payload Parser"]
        CryptoVault["libsodium AEAD + OS Hardware Vault"]
        ProxyRouter["WireGuard SOCKS5 / Multi-Hop Proxy Chain"]
    end
    
    subgraph "Render / Web Content Layer"
        WCV1["Tab 1: Isolated Webview"]
        WCV2["Tab 2: Isolated Webview"]
        CDP["Chrome DevTools Protocol (CDP Hook)"]
    end

    UI --> IPC
    DevTools --> CDP
    IPC --> SessionMgr
    IPC --> TabMgr
    SessionMgr --> ProxyRouter
    SessionMgr --> AdBlock
    SessionMgr --> URLScanner
    SessionMgr --> CookieAnalyzer
    TabMgr --> WCV1
    TabMgr --> WCV2
    CryptoVault --> IPC
```

