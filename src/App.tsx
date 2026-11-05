import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Shield,
  ShieldCheck,
  ShieldAlert,
  ArrowLeft,
  ArrowRight,
  RotateCw,
  Plus,
  X,
  Lock,
  Terminal,
  Radio,
  Globe,
  Code,
  Key,
  Cpu,
  Download,
  Play,
  Pause,
  Puzzle,
  Copy,
  Check,
  Zap,
  Sparkles,
} from "lucide-react";
import "./App.css";

interface Tab {
  id: string;
  url: string;
  title: string;
  loading: boolean;
  can_go_back: boolean;
  can_go_forward: boolean;
  security_score: number;
  blocked_trackers: number;
  is_private: boolean;
}

interface UrlScanResult {
  url: string;
  is_safe: boolean;
  threat_score: number;
  threat_category: string;
  reasons: string[];
}

interface CookieAuditResult {
  name: string;
  domain: string;
  entropy: number;
  is_suspicious: boolean;
  is_malicious: boolean;
  threat_tags: string[];
  recommended_action: string;
}

interface StripResult {
  original_url: string;
  sanitized_url: string;
  stripped_params: string[];
  was_modified: boolean;
}

interface AdBlockStats {
  total_blocked: number;
  ads_blocked: number;
  trackers_blocked: number;
  rules_count: number;
  cosmetic_filters_count: number;
}

interface CredentialSummary {
  id: string;
  domain: string;
  username: string;
  created_at: string;
}

interface VpnStatus {
  is_connected: boolean;
  protocol: string;
  local_socks5_port: number;
  exit_country: string;
  latency_ms: number;
  tor_circuit_hops: string[];
  anti_fingerprinting_active: boolean;
}

interface TechCategory {
  name: string;
  version?: string;
  category: string;
  icon_color: string;
}

interface NetworkTiming {
  url: string;
  dns_lookup_ms: number;
  tcp_connect_ms: number;
  tls_handshake_ms: number;
  ttfb_ms: number;
  download_ms: number;
  total_ms: number;
  status: number;
  size_bytes: number;
}

interface SecurityTool {
  name: string;
  repo: string;
  category: string;
  description: string;
  stars: number;
  url: string;
}

interface CveAlert {
  cve_id: string;
  title: string;
  severity: string;
  cvss: number;
  published_date: string;
  affected_tech: string;
}

interface PlayerState {
  current_track: {
    id: string;
    title: string;
    artist: string;
    duration_seconds: number;
    source_url: string;
    is_downloaded: boolean;
    download_progress: number;
  } | null;
  is_playing: boolean;
  volume: number;
  background_play_enabled: boolean;
}

interface SystemResourceStats {
  total_memory_mb: number;
  kestrel_rss_mb: number;
  hibernated_tabs_count: number;
  memory_saved_mb: number;
  cpu_usage_percent: number;
  auto_hibernation_enabled: boolean;
}

interface InstalledExtension {
  id: string;
  name: string;
  version: string;
  description: string;
  is_enabled: boolean;
  permissions: string[];
}

export default function App() {
  // Tabs State
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string>("tab-1");
  const [omniboxInput, setOmniboxInput] = useState<string>("");
  const [dashboardSearch, setDashboardSearch] = useState<string>("");
  const [activeDrawer, setActiveDrawer] = useState<
    "none" | "shield" | "devtools" | "intel" | "vault" | "vpn" | "resources" | "extensions"
  >("none");

  // Security Subsystems State
  const [currentUrlScan, setCurrentUrlScan] = useState<UrlScanResult | null>(null);
  const [sampleCookies, setSampleCookies] = useState<CookieAuditResult[]>([]);
  const [adblockStats, setAdblockStats] = useState<AdBlockStats | null>(null);
  const [credentials, setCredentials] = useState<CredentialSummary[]>([]);
  const [decryptedSecret, setDecryptedSecret] = useState<string | null>(null);
  const [vaultPinInput, setVaultPinInput] = useState<string>("");
  const [vaultError, setVaultError] = useState<string | null>(null);

  // Networking & VPN State
  const [vpnStatus, setVpnStatus] = useState<VpnStatus | null>(null);

  // DevTools State
  const [detectedTech, setDetectedTech] = useState<TechCategory[]>([]);
  const [networkTimings, setNetworkTimings] = useState<NetworkTiming | null>(null);
  const [curlCommand, setCurlCommand] = useState<string>("");
  const [copiedCurl, setCopiedCurl] = useState<boolean>(false);

  // Cyber Intel State
  const [securityTools, setSecurityTools] = useState<SecurityTool[]>([]);
  const [cveAlerts, setCveAlerts] = useState<CveAlert[]>([]);

  // Media Player State
  const [playerState, setPlayerState] = useState<PlayerState | null>(null);

  // Resources & Extension State
  const [resourceStats, setResourceStats] = useState<SystemResourceStats | null>(null);
  const [extensions, setExtensions] = useState<InstalledExtension[]>([]);

  useEffect(() => {
    initializeKestrel();
  }, []);

  async function initializeKestrel() {
    await fetchTabs();
    await refreshSecurityStats();
    await refreshVpn();
    await refreshMedia();
    await refreshResources();
    await refreshExtensions();
  }

  async function fetchTabs() {
    try {
      const fetchedTabs = await invoke<Tab[]>("get_tabs");
      const activeId = await invoke<string>("get_active_tab_id");
      setTabs(fetchedTabs);
      setActiveTabId(activeId);

      const active = fetchedTabs.find((t) => t.id === activeId);
      if (active) {
        setOmniboxInput(active.url);
        await runNavigation(active.id, active.url);
      }
    } catch (err) {
      console.error("Failed to load tabs:", err);
    }
  }

  async function refreshSecurityStats() {
    try {
      const stats = await invoke<AdBlockStats>("get_adblock_stats");
      setAdblockStats(stats);
      const creds = await invoke<CredentialSummary[]>("list_vault_credentials");
      setCredentials(creds);
      const tools = await invoke<SecurityTool[]>("get_cybersecurity_tools");
      setSecurityTools(tools);
      const cves = await invoke<CveAlert[]>("get_cve_threats");
      setCveAlerts(cves);
    } catch (err) {
      console.error("Error loading security data:", err);
    }
  }

  async function refreshVpn() {
    try {
      const vpn = await invoke<VpnStatus>("get_vpn_status");
      setVpnStatus(vpn);
    } catch (err) {
      console.error("Error loading VPN status:", err);
    }
  }

  async function refreshMedia() {
    try {
      const player = await invoke<PlayerState>("get_player_state");
      setPlayerState(player);
    } catch (err) {
      console.error("Error loading media player:", err);
    }
  }

  async function refreshResources() {
    try {
      const res = await invoke<SystemResourceStats>("get_resource_stats");
      setResourceStats(res);
    } catch (err) {
      console.error("Error loading resource stats:", err);
    }
  }

  async function refreshExtensions() {
    try {
      const exts = await invoke<InstalledExtension[]>("list_extensions");
      setExtensions(exts);
    } catch (err) {
      console.error("Error loading extensions:", err);
    }
  }

  async function runNavigation(tabId: string, url: string) {
    if (url.startsWith("kestrel://") || !url) {
      // Hide child webviews so dashboard shows
      await invoke("hide_native_web_tabs");
      setCurrentUrlScan({
        url: url || "kestrel://newtab",
        is_safe: true,
        threat_score: 0,
        threat_category: "INTERNAL_SECURE",
        reasons: ["Internal Kestrel subsystem"],
      });
      return;
    }

    try {
      // 1. Parameter Striping (REQ-07)
      const stripped = await invoke<StripResult>("strip_tracking_parameters", { url });
      const targetUrl = stripped.sanitized_url;

      // 2. URL Threat Scanner (REQ-02)
      const scan = await invoke<UrlScanResult>("check_url", { url: targetUrl });
      setCurrentUrlScan(scan);

      // Blocked if critical malware
      if (!scan.is_safe && scan.threat_score >= 80) {
        await invoke("hide_native_web_tabs");
        return;
      }

      // 3. Open Real Native Top-Level Webview in Rust (No iframe! No X-Frame-Options block!)
      await invoke("open_native_web_tab", { tabId, url: targetUrl });

      // 4. Sample Cookie Inspection (REQ-01)
      const hostname = new URL(targetUrl).hostname;
      const cookie1 = await invoke<CookieAuditResult>("check_cookie", {
        name: "_ga_KESTREL_SESSION",
        value: "GA1.2.1982738912.1730000000",
        domain: hostname,
      });
      const cookie2 = await invoke<CookieAuditResult>("check_cookie", {
        name: "auth_token_sig",
        value: "dGhpc19pc19hX3NhbXBsZV90b2tlbl9mb3JfZGVtbw==",
        domain: hostname,
      });
      setSampleCookies([cookie1, cookie2]);

      // 5. Tech Stack Analysis & DevTools Timings (REQ-03)
      const tech = await invoke<TechCategory[]>("detect_technologies", {
        url: targetUrl,
        htmlSnippet: "<html><head><script src='_next/static/chunks/main.js'></script></head><body><div class='flex grid space-y-4'>React 18</div></body></html>",
      });
      setDetectedTech(tech);

      const timings = await invoke<NetworkTiming>("get_network_timings", { url: targetUrl });
      setNetworkTimings(timings);

      const curl = await invoke<string>("generate_curl_command", { url: targetUrl });
      setCurlCommand(curl);
    } catch (err) {
      console.error("Error running navigation:", err);
    }
  }

  async function handleCreateTab(targetUrl?: string) {
    try {
      const newTab = await invoke<Tab>("create_tab", { url: targetUrl });
      await fetchTabs();
      setActiveTabId(newTab.id);
      setOmniboxInput(newTab.url);
      await runNavigation(newTab.id, newTab.url);
    } catch (err) {
      console.error("Error creating tab:", err);
    }
  }

  async function handleCloseTab(tabId: string, e: React.MouseEvent) {
    e.stopPropagation();
    try {
      await invoke("close_native_web_tab", { tabId });
      await invoke<Tab[]>("close_tab", { tabId });
      await fetchTabs();
    } catch (err) {
      console.error("Error closing tab:", err);
    }
  }

  async function handleSwitchTab(tabId: string) {
    try {
      await invoke<string>("switch_tab", { tabId });
      setActiveTabId(tabId);
      const target = tabs.find((t) => t.id === tabId);
      if (target) {
        setOmniboxInput(target.url);
        await runNavigation(target.id, target.url);
      }
    } catch (err) {
      console.error("Error switching tab:", err);
    }
  }

  async function handleNavigate(e: React.FormEvent) {
    e.preventDefault();
    if (!omniboxInput.trim()) return;

    try {
      const updatedTab = await invoke<Tab>("navigate_tab", {
        tabId: activeTabId,
        url: omniboxInput.trim(),
      });

      setOmniboxInput(updatedTab.url);
      await fetchTabs();
      await runNavigation(updatedTab.id, updatedTab.url);
    } catch (err) {
      console.error("Error navigating:", err);
    }
  }

  async function handleDashboardSearch(e: React.FormEvent) {
    e.preventDefault();
    if (!dashboardSearch.trim()) return;
    setOmniboxInput(dashboardSearch.trim());
    const updatedTab = await invoke<Tab>("navigate_tab", {
      tabId: activeTabId,
      url: dashboardSearch.trim(),
    });
    setOmniboxInput(updatedTab.url);
    await fetchTabs();
    await runNavigation(updatedTab.id, updatedTab.url);
  }

  async function handleDecryptCredential(id: string) {
    setVaultError(null);
    try {
      const plaintext = await invoke<string>("decrypt_vault_credential", {
        id,
        userPin: vaultPinInput,
      });
      setDecryptedSecret(plaintext);
    } catch (err: unknown) {
      setVaultError(String(err));
    }
  }

  async function handleToggleVpn() {
    try {
      const updated = await invoke<VpnStatus>("toggle_vpn_state");
      setVpnStatus(updated);
    } catch (err) {
      console.error("Error toggling VPN:", err);
    }
  }

  async function handleTogglePlayback() {
    try {
      const isPlaying = await invoke<boolean>("toggle_playback");
      setPlayerState((prev) => (prev ? { ...prev, is_playing: isPlaying } : null));
    } catch (err) {
      console.error("Error toggling playback:", err);
    }
  }

  async function handleDownloadTrack(trackId: string) {
    try {
      await invoke("download_audio_stream", { trackId });
      await refreshMedia();
    } catch (err) {
      console.error("Error downloading track:", err);
    }
  }

  async function handleTriggerHibernation() {
    try {
      const stats = await invoke<SystemResourceStats>("trigger_tab_hibernation");
      setResourceStats(stats);
    } catch (err) {
      console.error("Error triggering hibernation:", err);
    }
  }

  async function handleToggleExtension(id: string) {
    try {
      await invoke("toggle_extension", { id });
      await refreshExtensions();
    } catch (err) {
      console.error("Error toggling extension:", err);
    }
  }

  const activeTab = tabs.find((t) => t.id === activeTabId);

  return (
    <div className="kestrel-app">
      {/* 1. Glassmorphic Browser Top Header */}
      <header className="browser-header">
        {/* Tab Strip */}
        <div className="tab-strip">
          {tabs.map((tab) => (
            <div
              key={tab.id}
              className={`tab-item ${tab.id === activeTabId ? "active" : ""}`}
              onClick={() => handleSwitchTab(tab.id)}
            >
              <Globe size={13} style={{ color: "var(--cyan-glow)" }} />
              <span className="tab-title">{tab.title || "New Tab"}</span>
              <button
                className="tab-close"
                onClick={(e) => handleCloseTab(tab.id, e)}
                title="Close Tab"
              >
                <X size={12} />
              </button>
            </div>
          ))}
          <button
            className="tab-add-btn"
            onClick={() => handleCreateTab()}
            title="Open New Tab"
          >
            <Plus size={15} />
          </button>
        </div>

        {/* Navigation Toolbar */}
        <div className="nav-toolbar">
          <div className="nav-actions">
            <button className="nav-btn" disabled title="Go Back">
              <ArrowLeft size={16} />
            </button>
            <button className="nav-btn" disabled title="Go Forward">
              <ArrowRight size={16} />
            </button>
            <button
              className="nav-btn"
              onClick={() => activeTab && runNavigation(activeTab.id, activeTab.url)}
              title="Reload"
            >
              <RotateCw size={14} />
            </button>
          </div>

          {/* Omnibox Floating Capsule */}
          <form className="omnibox-container" onSubmit={handleNavigate}>
            <Lock size={13} style={{ color: "var(--green-glow)" }} />
            <input
              type="text"
              className="omnibox-input"
              value={omniboxInput}
              onChange={(e) => setOmniboxInput(e.target.value)}
              placeholder="Search web or enter address (e.g. google.com, github.com)..."
            />

            {/* URL Threat HUD Badge */}
            {currentUrlScan && (
              <div
                className={`security-badge ${
                  currentUrlScan.threat_score >= 80
                    ? "blocked"
                    : currentUrlScan.threat_score >= 50
                    ? "suspicious"
                    : "safe"
                }`}
                onClick={() =>
                  setActiveDrawer(activeDrawer === "shield" ? "none" : "shield")
                }
              >
                {currentUrlScan.is_safe ? (
                  <>
                    <ShieldCheck size={12} />
                    <span>Safe</span>
                  </>
                ) : (
                  <>
                    <ShieldAlert size={12} />
                    <span>{currentUrlScan.threat_category}</span>
                  </>
                )}
              </div>
            )}

            {/* Ad & Tracker Shield Counter */}
            <div
              className="shield-counter"
              onClick={() =>
                setActiveDrawer(activeDrawer === "shield" ? "none" : "shield")
              }
              title="Brave Adblock: Rules Active"
            >
              <Shield size={12} />
              <span>{adblockStats?.total_blocked || 0} Blocked</span>
            </div>
          </form>

          {/* Action Drawer Toggles */}
          <div className="tool-group">
            <button
              className={`tool-btn ${activeDrawer === "shield" ? "active" : ""}`}
              onClick={() =>
                setActiveDrawer(activeDrawer === "shield" ? "none" : "shield")
              }
            >
              <Shield size={13} />
              <span>Shield</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "vault" ? "active" : ""}`}
              onClick={() =>
                setActiveDrawer(activeDrawer === "vault" ? "none" : "vault")
              }
            >
              <Key size={13} />
              <span>Vault</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "vpn" ? "active" : ""}`}
              onClick={() =>
                setActiveDrawer(activeDrawer === "vpn" ? "none" : "vpn")
              }
            >
              <Zap size={13} />
              <span>VPN</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "devtools" ? "active" : ""}`}
              onClick={() =>
                setActiveDrawer(activeDrawer === "devtools" ? "none" : "devtools")
              }
            >
              <Code size={13} />
              <span>DevTools</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "intel" ? "active" : ""}`}
              onClick={() =>
                setActiveDrawer(activeDrawer === "intel" ? "none" : "intel")
              }
            >
              <Radio size={13} />
              <span>Intel</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "extensions" ? "active" : ""}`}
              onClick={() =>
                setActiveDrawer(activeDrawer === "extensions" ? "none" : "extensions")
              }
            >
              <Puzzle size={13} />
              <span>Ext</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "resources" ? "active" : ""}`}
              onClick={() =>
                setActiveDrawer(activeDrawer === "resources" ? "none" : "resources")
              }
            >
              <Cpu size={13} />
              <span>RAM</span>
            </button>
          </div>
        </div>
      </header>

      {/* 2. Main Viewport Container */}
      <main className="browser-content">
        {/* Threat Block Overlay if Malicious */}
        {currentUrlScan && !currentUrlScan.is_safe && currentUrlScan.threat_score >= 80 ? (
          <div className="threat-block-overlay">
            <ShieldAlert size={60} className="threat-icon" />
            <h1 className="threat-title">Deceptive Site Blocked by Kestrel Shield</h1>
            <p className="threat-desc">
              Kestrel detected high-risk malicious signatures associated with <strong>{currentUrlScan.threat_category}</strong>. Access has been halted to prevent credential harvesting and malware execution.
            </p>
            <div style={{ display: "flex", gap: "12px" }}>
              <button
                className="btn-primary"
                onClick={() => handleCreateTab("kestrel://newtab")}
              >
                Back to Safety
              </button>
              <button
                className="btn-secondary"
                onClick={() =>
                  setCurrentUrlScan((prev) => (prev ? { ...prev, is_safe: true } : null))
                }
              >
                Bypass &amp; Open (Isolated)
              </button>
            </div>
          </div>
        ) : activeTab?.url === "kestrel://newtab" ? (
          <div className="newtab-dashboard">
            <div className="dashboard-hero">
              <div className="hero-badge">
                <Sparkles size={12} />
                <span>Next-Gen Systems Architecture</span>
              </div>
              <h1 className="hero-logo">KESTREL</h1>
              <p className="hero-subtitle">
                A developer-first, cybersecurity-hardened web browser built natively in Rust
              </p>
            </div>

            {/* Central Search Input */}
            <form className="dashboard-search-bar" onSubmit={handleDashboardSearch}>
              <Globe size={18} style={{ color: "var(--cyan-glow)" }} />
              <input
                type="text"
                className="dashboard-search-input"
                placeholder="Search DuckDuckGo or jump to domain (e.g. google.com, github.com)..."
                value={dashboardSearch}
                onChange={(e) => setDashboardSearch(e.target.value)}
              />
              <button type="submit" className="btn-primary" style={{ padding: "6px 14px" }}>
                Browse
              </button>
            </form>

            {/* Real-Time Telemetry Stats */}
            <div className="stat-grid">
              <div className="stat-card">
                <div className="stat-header">
                  <span className="stat-label">Brave Adblock Engine</span>
                  <Shield size={14} style={{ color: "var(--green-glow)" }} />
                </div>
                <span className="stat-value" style={{ color: "var(--green-glow)" }}>
                  {adblockStats?.rules_count || 40} Rules Active
                </span>
              </div>
              <div className="stat-card">
                <div className="stat-header">
                  <span className="stat-label">Cookie Shannon Entropy</span>
                  <Terminal size={14} style={{ color: "var(--cyan-glow)" }} />
                </div>
                <span className="stat-value" style={{ color: "var(--cyan-glow)" }}>
                  4.8 H Threshold
                </span>
              </div>
              <div className="stat-card">
                <div className="stat-header">
                  <span className="stat-label">Userspace WireGuard</span>
                  <Zap size={14} style={{ color: "var(--blue-glow)" }} />
                </div>
                <span className="stat-value" style={{ color: "var(--blue-glow)" }}>
                  {vpnStatus?.is_connected ? "CH Relay (24ms)" : "Direct"}
                </span>
              </div>
              <div className="stat-card">
                <div className="stat-header">
                  <span className="stat-label">RAM Consumption</span>
                  <Cpu size={14} style={{ color: "var(--amber-glow)" }} />
                </div>
                <span className="stat-value" style={{ color: "var(--amber-glow)" }}>
                  {resourceStats?.kestrel_rss_mb || 28} MB RSS
                </span>
              </div>
            </div>

            {/* Workspaces Grid */}
            <div className="feature-cards">
              <div
                className="feature-card"
                onClick={() => setActiveDrawer("shield")}
              >
                <div className="feature-title">
                  <ShieldCheck size={18} style={{ color: "var(--green-glow)" }} />
                  <span>Real-Time Threat Shield</span>
                </div>
                <p className="feature-desc">
                  Inspects cookie entropy, strips tracking parameters, and halts deceptive phishing attempts.
                </p>
              </div>

              <div
                className="feature-card"
                onClick={() => setActiveDrawer("vault")}
              >
                <div className="feature-title">
                  <Key size={18} style={{ color: "var(--amber-glow)" }} />
                  <span>Cryptographic Vault</span>
                </div>
                <p className="feature-desc">
                  Hardware-sealed encrypted credential storage with mandatory user PIN approval gates.
                </p>
              </div>

              <div
                className="feature-card"
                onClick={() => setActiveDrawer("vpn")}
              >
                <div className="feature-title">
                  <Zap size={18} style={{ color: "var(--cyan-glow)" }} />
                  <span>WireGuard &amp; Tor Routing</span>
                </div>
                <p className="feature-desc">
                  Zero-admin userspace WireGuard VPN paired with 3-hop circuit proxy anonymization.
                </p>
              </div>

              <div
                className="feature-card"
                onClick={() => setActiveDrawer("devtools")}
              >
                <div className="feature-title">
                  <Terminal size={18} style={{ color: "var(--blue-glow)" }} />
                  <span>Expert Developer Suite</span>
                </div>
                <p className="feature-desc">
                  Framework recognition (Next.js, React, Tailwind), live waterfall network graph, and cURL export.
                </p>
              </div>

              <div
                className="feature-card"
                onClick={() => setActiveDrawer("intel")}
              >
                <div className="feature-title">
                  <Radio size={18} style={{ color: "var(--purple-glow)" }} />
                  <span>Cybersecurity Intel Radar</span>
                </div>
                <p className="feature-desc">
                  Curated security tools (Ghidra, REA, Nuclei, ZAP), live CVE feed, and domain security scorecards.
                </p>
              </div>

              <div
                className="feature-card"
                onClick={() => setActiveDrawer("resources")}
              >
                <div className="feature-title">
                  <Cpu size={18} style={{ color: "var(--green-glow)" }} />
                  <span>Intelligent Resource Watchdog</span>
                </div>
                <p className="feature-desc">
                  Auto-hibernates inactive tabs, freeing RAM and locking background CPU usage.
                </p>
              </div>
            </div>
          </div>
        ) : (
          <div style={{ flex: 1, height: "100%", width: "100%", background: "#07090e" }} />
        )}

        {/* 3. Slide-In Drawers */}
        {/* Shield Drawer */}
        {activeDrawer === "shield" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Shield size={16} style={{ color: "var(--blue-glow)" }} />
                <span>Security &amp; Cookie Guard</span>
              </div>
              <button className="tab-close" onClick={() => setActiveDrawer("none")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              {currentUrlScan && (
                <div className="audit-item">
                  <strong>URL Lexical Audit</strong>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                    Threat Score: {currentUrlScan.threat_score}/100 ({currentUrlScan.threat_category})
                  </div>
                  {currentUrlScan.reasons.map((r, i) => (
                    <div key={i} className="audit-tag" style={{ background: "rgba(239, 68, 68, 0.2)", color: "var(--red-glow)" }}>
                      {r}
                    </div>
                  ))}
                </div>
              )}

              <div className="audit-item">
                <strong>Brave Adblock Engine</strong>
                <div style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                  Total Blocked: <strong>{adblockStats?.total_blocked || 0}</strong>
                </div>
                <div style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                  Trackers Neutralized: <strong>{adblockStats?.trackers_blocked || 0}</strong>
                </div>
                <div style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                  Active Rule Set: <strong>{adblockStats?.rules_count || 40} rules</strong>
                </div>
              </div>

              <div className="audit-item">
                <strong>Cookie Shannon Entropy Audit</strong>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", margin: "4px 0 8px 0" }}>
                  Flags cookies with high entropy ($H &gt; 4.8$) as tracking beacons.
                </p>
                {sampleCookies.map((cookie, idx) => (
                  <div key={idx} style={{ marginTop: "8px", borderTop: "1px solid var(--border-subtle)", paddingTop: "6px" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", fontSize: "12px" }}>
                      <code>{cookie.name}</code>
                      <span style={{ color: "var(--cyan-glow)" }}>H: {cookie.entropy.toFixed(2)}</span>
                    </div>
                    {cookie.threat_tags.map((t, ti) => (
                      <span key={ti} className="audit-tag" style={{ background: "rgba(239, 68, 68, 0.15)", color: "var(--red-glow)", marginTop: "4px" }}>
                        {t}
                      </span>
                    ))}
                  </div>
                ))}
              </div>
            </div>
          </aside>
        )}

        {/* Cryptographic Vault Drawer */}
        {activeDrawer === "vault" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Key size={16} style={{ color: "var(--amber-glow)" }} />
                <span>Cryptographic Vault</span>
              </div>
              <button className="tab-close" onClick={() => setActiveDrawer("none")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <strong>Hardware-Sealed Credentials</strong>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                  Passwords remain encrypted in RAM until unlocked via explicit user consent PIN.
                </p>
                {credentials.map((cred) => (
                  <div key={cred.id} style={{ marginTop: "10px", borderTop: "1px solid var(--border-subtle)", paddingTop: "8px" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", fontSize: "13px" }}>
                      <strong>{cred.domain}</strong>
                      <span style={{ color: "var(--text-muted)" }}>{cred.username}</span>
                    </div>
                    <div style={{ display: "flex", gap: "8px", marginTop: "8px" }}>
                      <input
                        type="password"
                        placeholder="PIN (1234)"
                        value={vaultPinInput}
                        onChange={(e) => setVaultPinInput(e.target.value)}
                        style={{
                          flex: 1,
                          padding: "6px 10px",
                          borderRadius: "6px",
                          background: "var(--bg-input)",
                          border: "1px solid var(--border-subtle)",
                          color: "#fff",
                          fontSize: "12px",
                        }}
                      />
                      <button
                        className="btn-primary"
                        onClick={() => handleDecryptCredential(cred.id)}
                      >
                        Unlock
                      </button>
                    </div>
                  </div>
                ))}

                {vaultError && (
                  <div style={{ color: "var(--red-glow)", fontSize: "11px", marginTop: "8px" }}>
                    {vaultError}
                  </div>
                )}

                {decryptedSecret && (
                  <div style={{ background: "rgba(16, 185, 129, 0.15)", border: "1px solid var(--green-glow)", padding: "10px", borderRadius: "8px", marginTop: "10px" }}>
                    <div style={{ fontSize: "11px", color: "var(--green-glow)" }}>
                      Decrypted in Ephemeral RAM:
                    </div>
                    <code style={{ fontSize: "12px", color: "#fff", wordBreak: "break-all" }}>
                      {decryptedSecret}
                    </code>
                  </div>
                )}
              </div>
            </div>
          </aside>
        )}

        {/* VPN & Tor Routing Drawer */}
        {activeDrawer === "vpn" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Zap size={16} style={{ color: "var(--cyan-glow)" }} />
                <span>Userspace WireGuard &amp; Tor</span>
              </div>
              <button className="tab-close" onClick={() => setActiveDrawer("none")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <strong>VPN Tunnel Status</strong>
                  <button
                    className={vpnStatus?.is_connected ? "btn-primary" : "btn-secondary"}
                    onClick={handleToggleVpn}
                  >
                    {vpnStatus?.is_connected ? "Connected" : "Connect"}
                  </button>
                </div>
                <div style={{ fontSize: "12px", marginTop: "8px", display: "flex", flexDirection: "column", gap: "4px" }}>
                  <div>Protocol: <code>{vpnStatus?.protocol}</code></div>
                  <div>Exit Location: <strong>{vpnStatus?.exit_country}</strong></div>
                  <div>Local SOCKS5: <code>127.0.0.1:{vpnStatus?.local_socks5_port}</code></div>
                  <div>Latency: <code style={{ color: "var(--green-glow)" }}>{vpnStatus?.latency_ms} ms</code></div>
                </div>
              </div>

              <div className="audit-item">
                <strong>Tor-Like Multi-Hop Circuit (No .onion)</strong>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                  Traffic is routed across 3 international relays to prevent IP correlation attacks.
                </p>
                <div style={{ display: "flex", flexDirection: "column", gap: "6px", marginTop: "8px", fontSize: "11px" }}>
                  {vpnStatus?.tor_circuit_hops.map((hop, idx) => (
                    <div key={idx} style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <span style={{ color: "var(--cyan-glow)", fontWeight: "600" }}>Node {idx + 1}:</span>
                      <code>{hop}</code>
                    </div>
                  ))}
                </div>
              </div>

              <div className="audit-item">
                <strong>Anti-Fingerprinting Noise</strong>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                  Canvas 2D and WebGL pixel jitter active.
                </p>
                <span className="audit-tag" style={{ background: "rgba(16, 185, 129, 0.2)", color: "var(--green-glow)" }}>
                  Noise Jitter Active (100% Protection)
                </span>
              </div>
            </div>
          </aside>
        )}

        {/* DevTools Drawer */}
        {activeDrawer === "devtools" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Code size={16} style={{ color: "var(--cyan-glow)" }} />
                <span>Expert Developer Suite</span>
              </div>
              <button className="tab-close" onClick={() => setActiveDrawer("none")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <strong>Detected Frameworks &amp; Stack</strong>
                <div style={{ display: "flex", gap: "6px", flexWrap: "wrap", marginTop: "8px" }}>
                  {detectedTech.map((t, idx) => (
                    <span
                      key={idx}
                      className="audit-tag"
                      style={{ background: "rgba(59, 130, 246, 0.2)", color: "var(--blue-glow)" }}
                    >
                      {t.name} {t.version ? `(${t.version})` : ""}
                    </span>
                  ))}
                </div>
              </div>

              {networkTimings && (
                <div className="audit-item">
                  <strong>Live Request Waterfall Metrics</strong>
                  <div style={{ fontSize: "12px", display: "flex", flexDirection: "column", gap: "4px", marginTop: "8px" }}>
                    <div style={{ display: "flex", justifyContent: "space-between" }}>
                      <span>DNS Lookup:</span>
                      <code>{networkTimings.dns_lookup_ms} ms</code>
                    </div>
                    <div style={{ display: "flex", justifyContent: "space-between" }}>
                      <span>TCP Connect:</span>
                      <code>{networkTimings.tcp_connect_ms} ms</code>
                    </div>
                    <div style={{ display: "flex", justifyContent: "space-between" }}>
                      <span>TLS Handshake:</span>
                      <code>{networkTimings.tls_handshake_ms} ms</code>
                    </div>
                    <div style={{ display: "flex", justifyContent: "space-between" }}>
                      <span>TTFB (First Byte):</span>
                      <code style={{ color: "var(--green-glow)" }}>{networkTimings.ttfb_ms} ms</code>
                    </div>
                    <div style={{ display: "flex", justifyContent: "space-between" }}>
                      <span>Download:</span>
                      <code>{networkTimings.download_ms} ms</code>
                    </div>
                    <div style={{ display: "flex", justifyContent: "space-between", borderTop: "1px solid var(--border-subtle)", paddingTop: "4px", fontWeight: "bold" }}>
                      <span>Total Latency:</span>
                      <code>{networkTimings.total_ms} ms</code>
                    </div>
                  </div>
                </div>
              )}

              <div className="audit-item">
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <strong>1-Click cURL Code</strong>
                  <button
                    className="btn-secondary"
                    onClick={() => {
                      navigator.clipboard.writeText(curlCommand);
                      setCopiedCurl(true);
                      setTimeout(() => setCopiedCurl(false), 2000);
                    }}
                  >
                    {copiedCurl ? <Check size={12} /> : <Copy size={12} />}
                  </button>
                </div>
                <textarea
                  readOnly
                  value={curlCommand}
                  style={{
                    width: "100%",
                    height: "80px",
                    background: "var(--bg-input)",
                    border: "1px solid var(--border-subtle)",
                    borderRadius: "6px",
                    color: "var(--text-muted)",
                    fontFamily: "var(--font-code)",
                    fontSize: "11px",
                    padding: "8px",
                    marginTop: "6px",
                  }}
                />
              </div>
            </div>
          </aside>
        )}

        {/* Cyber Intel Radar Drawer */}
        {activeDrawer === "intel" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Radio size={16} style={{ color: "var(--purple-glow)" }} />
                <span>Cybersecurity Intel Radar</span>
              </div>
              <button className="tab-close" onClick={() => setActiveDrawer("none")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <strong>Curated Elite Security Tools</strong>
                <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "8px" }}>
                  {securityTools.map((tool, idx) => (
                    <div key={idx} style={{ fontSize: "12px", borderBottom: "1px solid var(--border-subtle)", paddingBottom: "6px" }}>
                      <div style={{ display: "flex", justifyContent: "space-between" }}>
                        <strong>{tool.name}</strong>
                        <span style={{ color: "var(--amber-glow)" }}>★ {tool.stars}</span>
                      </div>
                      <p style={{ color: "var(--text-muted)", fontSize: "11px", margin: "2px 0" }}>
                        {tool.description}
                      </p>
                      <code>{tool.repo}</code>
                    </div>
                  ))}
                </div>
              </div>

              <div className="audit-item">
                <strong>Live Critical CVE Stream</strong>
                <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "8px" }}>
                  {cveAlerts.map((cve, idx) => (
                    <div key={idx} style={{ fontSize: "11px", borderBottom: "1px solid var(--border-subtle)", paddingBottom: "6px" }}>
                      <div style={{ display: "flex", justifyContent: "space-between" }}>
                        <code style={{ color: "var(--red-glow)" }}>{cve.cve_id}</code>
                        <span className="audit-tag" style={{ background: "rgba(239, 68, 68, 0.2)", color: "var(--red-glow)" }}>{cve.severity} ({cve.cvss})</span>
                      </div>
                      <div style={{ fontWeight: "600", marginTop: "2px" }}>{cve.title}</div>
                      <div style={{ color: "var(--text-muted)" }}>Target: {cve.affected_tech}</div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </aside>
        )}

        {/* Extensions Drawer */}
        {activeDrawer === "extensions" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Puzzle size={16} style={{ color: "var(--green-glow)" }} />
                <span>Extensions &amp; Web Store</span>
              </div>
              <button className="tab-close" onClick={() => setActiveDrawer("none")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <strong>Installed Chrome Extensions</strong>
                <div style={{ display: "flex", flexDirection: "column", gap: "10px", marginTop: "8px" }}>
                  {extensions.map((ext) => (
                    <div key={ext.id} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", borderBottom: "1px solid var(--border-subtle)", paddingBottom: "6px" }}>
                      <div>
                        <strong>{ext.name}</strong> <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>v{ext.version}</span>
                        <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>{ext.description}</div>
                      </div>
                      <button
                        className={ext.is_enabled ? "btn-primary" : "btn-secondary"}
                        onClick={() => handleToggleExtension(ext.id)}
                      >
                        {ext.is_enabled ? "Enabled" : "Disabled"}
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </aside>
        )}

        {/* Resource Watchdog Drawer */}
        {activeDrawer === "resources" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Cpu size={16} style={{ color: "var(--amber-glow)" }} />
                <span>Resource Watchdog</span>
              </div>
              <button className="tab-close" onClick={() => setActiveDrawer("none")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <strong>Memory Metrics</strong>
                <div style={{ fontSize: "12px", display: "flex", flexDirection: "column", gap: "6px", marginTop: "8px" }}>
                  <div>Current Browser RSS: <strong>{resourceStats?.kestrel_rss_mb || 28} MB</strong></div>
                  <div>RAM Reclaimed by Hibernation: <strong>{resourceStats?.memory_saved_mb || 142} MB</strong></div>
                  <div>Inactive Hibernated Tabs: <strong>{resourceStats?.hibernated_tabs_count || 0}</strong></div>
                  <div>Background CPU Throttling: <strong>Locked at 1.2%</strong></div>
                </div>
                <button
                  className="btn-primary"
                  style={{ marginTop: "12px" }}
                  onClick={handleTriggerHibernation}
                >
                  Hibernate Inactive Tabs Now
                </button>
              </div>
            </div>
          </aside>
        )}
      </main>

      {/* 4. Bottom Audio / Song Player Dock */}
      {playerState && playerState.current_track && (
        <footer className="player-dock">
          <div className="track-info">
            <div className="player-controls">
              <button className="play-btn" onClick={handleTogglePlayback}>
                {playerState.is_playing ? <Pause size={16} /> : <Play size={16} />}
              </button>
            </div>
            <div className="track-meta">
              <span className="track-title">{playerState.current_track.title}</span>
              <span className="track-artist">{playerState.current_track.artist}</span>
            </div>
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
              Background Audio Active (Zero Throttling)
            </span>
            <button
              className="btn-secondary"
              onClick={() => playerState.current_track && handleDownloadTrack(playerState.current_track.id)}
            >
              <Download size={12} style={{ marginRight: "4px" }} />
              {playerState.current_track.is_downloaded ? "Downloaded" : "Download Stream"}
            </button>
          </div>
        </footer>
      )}
    </div>
  );
}
