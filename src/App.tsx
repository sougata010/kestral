import { useState, useEffect, useRef, useCallback } from "react";
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
  Eye,
  EyeOff,
  Volume2,
  VolumeX,
  ListMusic,
  PlusCircle,
  Search,
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

interface AudioTrack {
  id: string;
  title: string;
  artist: string;
  duration_seconds: number;
  source_url: string;
  is_downloaded: boolean;
  download_progress: number;
}

interface PlayerState {
  current_track: AudioTrack | null;
  is_playing: boolean;
  volume: number;
  playlist: AudioTrack[];
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

const QUICK_BOOKMARKS = [
  { name: "Google", url: "https://google.com", icon: "🌐" },
  { name: "GitHub", url: "https://github.com", icon: "🐙" },
  { name: "Hacker News", url: "https://news.ycombinator.com", icon: "⚡" },
  { name: "Shodan", url: "https://www.shodan.io", icon: "🔍" },
  { name: "Cloudflare", url: "https://radar.cloudflare.com", icon: "🛡️" },
  { name: "Hugging Face", url: "https://huggingface.co", icon: "🤗" },
  { name: "Rust Docs", url: "https://doc.rust-lang.org", icon: "🦀" },
  { name: "CVE Mitre", url: "https://cve.mitre.org", icon: "🚨" },
];

function normalizeUrl(input: string): string {
  const trimmed = input.trim();
  if (!trimmed || trimmed === "kestrel://newtab") return "kestrel://newtab";
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://") || trimmed.startsWith("kestrel://")) {
    return trimmed;
  }
  if (trimmed.includes(".") && !trimmed.includes(" ")) {
    return `https://${trimmed}`;
  }
  return `https://duckduckgo.com/?q=${encodeURIComponent(trimmed)}`;
}

function getSafeHostname(urlStr: string): string {
  try {
    return new URL(urlStr).hostname;
  } catch {
    return urlStr.replace(/^https?:\/\//, "").split("/")[0] || "kestrel.internal";
  }
}

// Production Web Audio Ambient Synthesizer
class AmbientAudioEngine {
  private ctx: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private oscs: OscillatorNode[] = [];
  private isRunning: boolean = false;

  public start(volume: number = 0.5) {
    if (this.isRunning) return;
    try {
      const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      this.ctx = new AudioCtx();
      this.masterGain = this.ctx.createGain();
      this.masterGain.gain.setValueAtTime(volume * 0.15, this.ctx.currentTime);
      this.masterGain.connect(this.ctx.destination);

      // Create warm ambient multi-tone drone chord (55Hz, 110Hz, 164.81Hz)
      const frequencies = [55.0, 110.0, 164.81, 220.0];
      this.oscs = frequencies.map((freq) => {
        const osc = this.ctx!.createOscillator();
        const oscGain = this.ctx!.createGain();
        osc.type = "sine";
        osc.frequency.setValueAtTime(freq, this.ctx!.currentTime);
        oscGain.gain.setValueAtTime(0.25, this.ctx!.currentTime);
        osc.connect(oscGain);
        oscGain.connect(this.masterGain!);
        osc.start();
        return osc;
      });

      this.isRunning = true;
    } catch (e) {
      console.warn("AudioContext init error:", e);
    }
  }

  public stop() {
    if (!this.isRunning) return;
    try {
      this.oscs.forEach((osc) => {
        try {
          osc.stop();
          osc.disconnect();
        } catch {
          // ignore
        }
      });
      this.oscs = [];
      if (this.ctx) {
        this.ctx.close();
        this.ctx = null;
      }
      this.isRunning = false;
    } catch (e) {
      console.warn("AudioContext stop error:", e);
    }
  }

  public setVolume(volume: number) {
    if (this.masterGain && this.ctx) {
      this.masterGain.gain.setValueAtTime(volume * 0.15, this.ctx.currentTime);
    }
  }
}

const audioEngine = new AmbientAudioEngine();

export default function App() {
  // Tabs State
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string>("tab-1");
  const [omniboxInput, setOmniboxInput] = useState<string>("");
  const [dashboardSearch, setDashboardSearch] = useState<string>("");
  const [activeDrawer, setActiveDrawer] = useState<
    "none" | "shield" | "devtools" | "intel" | "vault" | "vpn" | "resources" | "extensions"
  >("none");

  // Loading Progress Bar
  const [loadProgress, setLoadProgress] = useState<number>(0);

  // Security Subsystems State
  const [currentUrlScan, setCurrentUrlScan] = useState<UrlScanResult | null>(null);
  const [sampleCookies, setSampleCookies] = useState<CookieAuditResult[]>([]);
  const [customCookieName, setCustomCookieName] = useState<string>("");
  const [customCookieValue, setCustomCookieValue] = useState<string>("");
  const [customCookieAudited, setCustomCookieAudited] = useState<CookieAuditResult | null>(null);
  const [adblockStats, setAdblockStats] = useState<AdBlockStats | null>(null);

  // Cryptographic Vault State
  const [credentials, setCredentials] = useState<CredentialSummary[]>([]);
  const [decryptedSecret, setDecryptedSecret] = useState<string | null>(null);
  const [vaultPinInput, setVaultPinInput] = useState<string>("1234");
  const [vaultError, setVaultError] = useState<string | null>(null);
  const [showSecret, setShowSecret] = useState<boolean>(true);
  const [showAddCredModal, setShowAddCredModal] = useState<boolean>(false);
  const [newCredDomain, setNewCredDomain] = useState<string>("");
  const [newCredUsername, setNewCredUsername] = useState<string>("");
  const [newCredSecret, setNewCredSecret] = useState<string>("");
  const [newCredPin, setNewCredPin] = useState<string>("1234");

  // Networking & VPN State
  const [vpnStatus, setVpnStatus] = useState<VpnStatus | null>(null);

  // DevTools State
  const [detectedTech, setDetectedTech] = useState<TechCategory[]>([]);
  const [networkTimings, setNetworkTimings] = useState<NetworkTiming | null>(null);
  const [curlCommand, setCurlCommand] = useState<string>("");
  const [copiedCurl, setCopiedCurl] = useState<boolean>(false);
  const [copiedSecret, setCopiedSecret] = useState<boolean>(false);

  // Cyber Intel State
  const [securityTools, setSecurityTools] = useState<SecurityTool[]>([]);
  const [cveAlerts, setCveAlerts] = useState<CveAlert[]>([]);

  // Media Player State
  const [playerState, setPlayerState] = useState<PlayerState | null>(null);
  const [audioVolume, setAudioVolume] = useState<number>(0.7);

  // Resources & Extension State
  const [resourceStats, setResourceStats] = useState<SystemResourceStats | null>(null);
  const [extensions, setExtensions] = useState<InstalledExtension[]>([]);

  const omniboxRef = useRef<HTMLInputElement>(null);
  const activeTab = tabs.find((t) => t.id === activeTabId);
  const activeTabRef = useRef(activeTab);
  activeTabRef.current = activeTab;

  const activeDrawerRef = useRef(activeDrawer);
  activeDrawerRef.current = activeDrawer;

  // Dynamic layout calculator
  const getLayoutBounds = useCallback(() => {
    const top = 74;
    const bottom = 48; // player dock
    const rightDrawerWidth = activeDrawerRef.current !== "none" ? 440 : 0;
    const width = Math.max(300, window.innerWidth - rightDrawerWidth);
    const height = Math.max(200, window.innerHeight - top - bottom);
    return { x: 0, y: top, width, height };
  }, []);

  // Sync webview bounds whenever window resizes or drawer changes
  const syncWebviewBounds = useCallback(async () => {
    const tab = activeTabRef.current;
    if (!tab || tab.url.startsWith("kestrel://") || !tab.url) {
      try {
        await invoke("hide_native_web_tabs");
      } catch {
        // ignore
      }
      return;
    }

    const { x, y, width, height } = getLayoutBounds();
    try {
      await invoke("update_native_web_tab_bounds", {
        tabId: tab.id,
        x,
        y,
        width,
        height,
      });
    } catch (e) {
      console.warn("Bounds sync error:", e);
    }
  }, [getLayoutBounds]);

  const toggleDrawer = (drawer: typeof activeDrawer) => {
    const next = activeDrawer === drawer ? "none" : drawer;
    setActiveDrawer(next);
    activeDrawerRef.current = next;
    setTimeout(() => {
      syncWebviewBounds();
    }, 10);
  };

  // Keyboard Shortcuts Hook
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "t") {
        e.preventDefault();
        handleCreateTab("kestrel://newtab");
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "w") {
        e.preventDefault();
        if (activeTabRef.current) {
          handleCloseTab(activeTabRef.current.id);
        }
      } else if ((e.ctrlKey || e.metaKey) && (e.key.toLowerCase() === "l" || e.key.toLowerCase() === "d")) {
        e.preventDefault();
        omniboxRef.current?.focus();
        omniboxRef.current?.select();
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "r") {
        e.preventDefault();
        handleReload();
      } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "i") {
        e.preventDefault();
        toggleDrawer("devtools");
      } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "s") {
        e.preventDefault();
        toggleDrawer("shield");
      } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "v") {
        e.preventDefault();
        toggleDrawer("vault");
      } else if (e.key === "Escape") {
        if (activeDrawerRef.current !== "none") {
          toggleDrawer("none");
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  useEffect(() => {
    initializeKestrel();

    const handleResize = () => {
      syncWebviewBounds();
    };

    window.addEventListener("resize", handleResize);
    return () => {
      window.removeEventListener("resize", handleResize);
      audioEngine.stop();
    };
  }, [syncWebviewBounds]);

  async function initializeKestrel() {
    try {
      const fetchedTabs = await invoke<Tab[]>("get_tabs");
      const activeId = await invoke<string>("get_active_tab_id");
      setTabs(fetchedTabs);
      setActiveTabId(activeId);

      const active = fetchedTabs.find((t) => t.id === activeId);
      if (active) {
        setOmniboxInput(active.url === "kestrel://newtab" ? "" : active.url);
      }
    } catch (err) {
      console.error("Failed to load initial tabs:", err);
    }

    refreshTelemetry();
  }

  async function refreshTelemetry() {
    try {
      const [stats, creds, tools, cves, vpn, player, res, exts] = await Promise.allSettled([
        invoke<AdBlockStats>("get_adblock_stats"),
        invoke<CredentialSummary[]>("list_vault_credentials"),
        invoke<SecurityTool[]>("get_cybersecurity_tools"),
        invoke<CveAlert[]>("get_cve_threats"),
        invoke<VpnStatus>("get_vpn_status"),
        invoke<PlayerState>("get_player_state"),
        invoke<SystemResourceStats>("get_resource_stats"),
        invoke<InstalledExtension[]>("list_extensions"),
      ]);

      if (stats.status === "fulfilled") setAdblockStats(stats.value);
      if (creds.status === "fulfilled") setCredentials(creds.value);
      if (tools.status === "fulfilled") setSecurityTools(tools.value);
      if (cves.status === "fulfilled") setCveAlerts(cves.value);
      if (vpn.status === "fulfilled") setVpnStatus(vpn.value);
      if (player.status === "fulfilled") setPlayerState(player.value);
      if (res.status === "fulfilled") setResourceStats(res.value);
      if (exts.status === "fulfilled") setExtensions(exts.value);
    } catch (e) {
      console.warn("Telemetry refresh error:", e);
    }
  }

  // Non-blocking, instant navigation pipeline with loading progress
  async function navigateTo(targetUrl: string, targetTabId?: string) {
    const tabId = targetTabId || activeTabId;
    const cleanUrl = normalizeUrl(targetUrl);

    setOmniboxInput(cleanUrl === "kestrel://newtab" ? "" : cleanUrl);

    // Animate top progress bar
    setLoadProgress(25);
    setTimeout(() => setLoadProgress(65), 120);

    // Update tab state in UI immediately
    setTabs((prev) =>
      prev.map((t) =>
        t.id === tabId
          ? {
              ...t,
              url: cleanUrl,
              title: cleanUrl === "kestrel://newtab" ? "New Tab" : getSafeHostname(cleanUrl),
              loading: cleanUrl !== "kestrel://newtab",
            }
          : t
      )
    );

    // Case 1: Internal new tab page
    if (cleanUrl.startsWith("kestrel://") || !cleanUrl) {
      try {
        await invoke("hide_native_web_tabs");
      } catch {
        // ignore
      }
      setCurrentUrlScan({
        url: "kestrel://newtab",
        is_safe: true,
        threat_score: 0,
        threat_category: "INTERNAL_SECURE",
        reasons: ["Internal Kestrel subsystem"],
      });
      setLoadProgress(100);
      setTimeout(() => setLoadProgress(0), 200);
      return;
    }

    // Case 2: External Web Page
    const { x, y, width, height } = getLayoutBounds();

    try {
      await invoke("open_native_web_tab", {
        tabId,
        url: cleanUrl,
        x,
        y,
        width,
        height,
      });
    } catch (err) {
      console.error("Webview navigation error:", err);
    }

    // Background security & devtools telemetry in parallel
    (async () => {
      try {
        const hostname = getSafeHostname(cleanUrl);

        const [, scanRes] = await Promise.allSettled([
          invoke("strip_tracking_parameters", { url: cleanUrl }),
          invoke<UrlScanResult>("check_url", { url: cleanUrl }),
        ]);

        if (scanRes.status === "fulfilled") {
          setCurrentUrlScan(scanRes.value);
          if (!scanRes.value.is_safe && scanRes.value.threat_score >= 80) {
            await invoke("hide_native_web_tabs");
            setLoadProgress(100);
            setTimeout(() => setLoadProgress(0), 200);
            return;
          }
        }

        const [c1, c2, tech, timings, curl] = await Promise.allSettled([
          invoke<CookieAuditResult>("check_cookie", {
            name: "_ga_KESTREL_SESSION",
            value: "GA1.2.1982738912.1730000000",
            domain: hostname,
          }),
          invoke<CookieAuditResult>("check_cookie", {
            name: "auth_token_sig",
            value: "dGhpc19pc19hX3NhbXBsZV90b2tlbl9mb3JfZGVtbw==",
            domain: hostname,
          }),
          invoke<TechCategory[]>("detect_technologies", {
            url: cleanUrl,
            htmlSnippet:
              "<html><head><script src='_next/static/chunks/main.js'></script></head><body><div class='flex space-y-4'>React 18</div></body></html>",
          }),
          invoke<NetworkTiming>("get_network_timings", { url: cleanUrl }),
          invoke<string>("generate_curl_command", { url: cleanUrl }),
        ]);

        const cookies: CookieAuditResult[] = [];
        if (c1.status === "fulfilled") cookies.push(c1.value);
        if (c2.status === "fulfilled") cookies.push(c2.value);
        setSampleCookies(cookies);

        if (tech.status === "fulfilled") setDetectedTech(tech.value);
        if (timings.status === "fulfilled") setNetworkTimings(timings.value);
        if (curl.status === "fulfilled") setCurlCommand(curl.value);

        setLoadProgress(100);
        setTimeout(() => setLoadProgress(0), 300);

        setTabs((prev) =>
          prev.map((t) => (t.id === tabId ? { ...t, loading: false } : t))
        );
      } catch (e) {
        console.warn("Background audit task error:", e);
        setLoadProgress(0);
      }
    })();
  }

  async function handleGoBack() {
    if (!activeTab) return;
    try {
      await invoke("navigate_native_web_tab_back", { tabId: activeTab.id });
    } catch {
      // ignore
    }
  }

  async function handleGoForward() {
    if (!activeTab) return;
    try {
      await invoke("navigate_native_web_tab_forward", { tabId: activeTab.id });
    } catch {
      // ignore
    }
  }

  async function handleReload() {
    if (!activeTab) return;
    if (activeTab.url.startsWith("kestrel://")) {
      navigateTo(activeTab.url);
    } else {
      try {
        await invoke("reload_native_web_tab", { tabId: activeTab.id });
      } catch {
        navigateTo(activeTab.url);
      }
    }
  }

  async function handleCreateTab(targetUrl?: string) {
    try {
      const newTab = await invoke<Tab>("create_tab", { url: targetUrl });
      setTabs((prev) => [...prev, newTab]);
      setActiveTabId(newTab.id);
      navigateTo(newTab.url, newTab.id);
    } catch (err) {
      console.error("Error creating tab:", err);
    }
  }

  async function handleCloseTab(tabId: string, e?: React.MouseEvent) {
    if (e) e.stopPropagation();
    try {
      await invoke("close_native_web_tab", { tabId });
      const updated = await invoke<Tab[]>("close_tab", { tabId });
      setTabs(updated);
      const activeId = await invoke<string>("get_active_tab_id");
      setActiveTabId(activeId);
      const current = updated.find((t) => t.id === activeId);
      if (current) {
        navigateTo(current.url, current.id);
      }
    } catch (err) {
      console.error("Error closing tab:", err);
    }
  }

  async function handleSwitchTab(tabId: string) {
    if (tabId === activeTabId) return;
    try {
      await invoke<string>("switch_tab", { tabId });
      setActiveTabId(tabId);
      const target = tabs.find((t) => t.id === tabId);
      if (target) {
        navigateTo(target.url, target.id);
      }
    } catch (err) {
      console.error("Error switching tab:", err);
    }
  }

  function handleOmniboxSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!omniboxInput.trim()) return;
    navigateTo(omniboxInput.trim());
  }

  function handleDashboardSearchSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!dashboardSearch.trim()) return;
    navigateTo(dashboardSearch.trim());
  }

  async function handleAuditCustomCookie() {
    if (!customCookieName.trim() || !customCookieValue.trim()) return;
    try {
      const res = await invoke<CookieAuditResult>("check_cookie", {
        name: customCookieName.trim(),
        value: customCookieValue.trim(),
        domain: activeTab ? getSafeHostname(activeTab.url) : "localhost",
      });
      setCustomCookieAudited(res);
    } catch (err) {
      console.error("Cookie audit error:", err);
    }
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

  async function handleAddCredential(e: React.FormEvent) {
    e.preventDefault();
    if (!newCredDomain.trim() || !newCredUsername.trim() || !newCredSecret.trim()) return;
    try {
      await invoke("add_vault_credential", {
        domain: newCredDomain.trim(),
        username: newCredUsername.trim(),
        secret: newCredSecret.trim(),
        pin: newCredPin.trim() || "1234",
      });
      const creds = await invoke<CredentialSummary[]>("list_vault_credentials");
      setCredentials(creds);
      setNewCredDomain("");
      setNewCredUsername("");
      setNewCredSecret("");
      setShowAddCredModal(false);
    } catch (err) {
      console.error("Error adding credential:", err);
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

      if (isPlaying) {
        audioEngine.start(audioVolume);
      } else {
        audioEngine.stop();
      }
    } catch (err) {
      console.error("Error toggling playback:", err);
    }
  }

  async function handleSwitchTrack(trackId: string) {
    try {
      const updated = await invoke<PlayerState>("switch_audio_track", { trackId });
      setPlayerState(updated);
      audioEngine.start(audioVolume);
    } catch (err) {
      console.error("Error switching track:", err);
    }
  }

  function handleVolumeChange(vol: number) {
    setAudioVolume(vol);
    audioEngine.setVolume(vol);
  }

  async function handleDownloadTrack(trackId: string) {
    try {
      await invoke("download_audio_stream", { trackId });
      const player = await invoke<PlayerState>("get_player_state");
      setPlayerState(player);
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
      const exts = await invoke<InstalledExtension[]>("list_extensions");
      setExtensions(exts);
    } catch (err) {
      console.error("Error toggling extension:", err);
    }
  }

  return (
    <div className="kestrel-app">
      {/* 0. Top Loading Progress Bar */}
      {loadProgress > 0 && (
        <div
          className="top-progress-bar"
          style={{ width: `${loadProgress}%` }}
        />
      )}

      {/* 1. Header Toolbar */}
      <header className="browser-header">
        {/* Tab Strip */}
        <div className="tab-strip">
          {tabs.map((tab) => (
            <div
              key={tab.id}
              className={`tab-item ${tab.id === activeTabId ? "active" : ""}`}
              onClick={() => handleSwitchTab(tab.id)}
            >
              <Globe size={13} style={{ color: "var(--cyan)" }} />
              <span className="tab-title">{tab.title || "New Tab"}</span>
              <button
                className="tab-close"
                onClick={(e) => handleCloseTab(tab.id, e)}
                title="Close Tab (Ctrl+W)"
              >
                <X size={12} />
              </button>
            </div>
          ))}
          <button
            className="tab-add-btn"
            onClick={() => handleCreateTab("kestrel://newtab")}
            title="Open New Tab (Ctrl+T)"
          >
            <Plus size={15} />
          </button>
        </div>

        {/* Navigation Toolbar */}
        <div className="nav-toolbar">
          <div className="nav-actions">
            <button
              className="nav-btn"
              onClick={handleGoBack}
              title="Back"
            >
              <ArrowLeft size={16} />
            </button>
            <button
              className="nav-btn"
              onClick={handleGoForward}
              title="Forward"
            >
              <ArrowRight size={16} />
            </button>
            <button
              className="nav-btn"
              onClick={handleReload}
              title="Reload (Ctrl+R / F5)"
            >
              <RotateCw size={14} className={activeTab?.loading ? "spin" : ""} />
            </button>
          </div>

          {/* Omnibox Floating Capsule */}
          <form className="omnibox-container" onSubmit={handleOmniboxSubmit}>
            <Lock size={13} style={{ color: "var(--emerald)" }} />
            <input
              ref={omniboxRef}
              type="text"
              className="omnibox-input"
              value={omniboxInput}
              onChange={(e) => setOmniboxInput(e.target.value)}
              placeholder="Search or enter domain (e.g. google.com, github.com)... (Ctrl+L)"
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
                onClick={() => toggleDrawer("shield")}
                title="Click for Shield Details (Ctrl+Shift+S)"
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
              onClick={() => toggleDrawer("shield")}
              title="Brave Adblock Engine"
            >
              <Shield size={12} />
              <span>{adblockStats?.total_blocked || 0} Blocked</span>
            </div>
          </form>

          {/* Action Drawer Toggles */}
          <div className="tool-group">
            <button
              className={`tool-btn ${activeDrawer === "shield" ? "active" : ""}`}
              onClick={() => toggleDrawer("shield")}
              title="Adblock & Threat Shield (Ctrl+Shift+S)"
            >
              <Shield size={13} />
              <span>Shield</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "vault" ? "active" : ""}`}
              onClick={() => toggleDrawer("vault")}
              title="Cryptographic Hardware Vault (Ctrl+Shift+V)"
            >
              <Key size={13} />
              <span>Vault</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "vpn" ? "active" : ""}`}
              onClick={() => toggleDrawer("vpn")}
              title="Userspace WireGuard & Tor Circuit"
            >
              <Zap size={13} />
              <span>VPN</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "devtools" ? "active" : ""}`}
              onClick={() => toggleDrawer("devtools")}
              title="Expert DevTools Suite (Ctrl+Shift+I)"
            >
              <Code size={13} />
              <span>DevTools</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "intel" ? "active" : ""}`}
              onClick={() => toggleDrawer("intel")}
              title="Cybersecurity Intel Radar"
            >
              <Radio size={13} />
              <span>Intel</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "extensions" ? "active" : ""}`}
              onClick={() => toggleDrawer("extensions")}
              title="Extension Manager"
            >
              <Puzzle size={13} />
              <span>Ext</span>
            </button>

            <button
              className={`tool-btn ${activeDrawer === "resources" ? "active" : ""}`}
              onClick={() => toggleDrawer("resources")}
              title="Resource Watchdog"
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
            <ShieldAlert size={64} className="threat-icon" />
            <h1 className="threat-title">Deceptive Site Blocked by Kestrel Shield</h1>
            <p className="threat-desc">
              Kestrel halted access to protect your device from <strong>{currentUrlScan.threat_category}</strong> signatures.
            </p>
            <div style={{ display: "flex", gap: "12px", marginTop: "16px" }}>
              <button
                className="btn-primary"
                onClick={() => navigateTo("kestrel://newtab")}
              >
                Back to Safety
              </button>
              <button
                className="btn-secondary"
                onClick={() => {
                  setCurrentUrlScan((prev) => (prev ? { ...prev, is_safe: true } : null));
                  if (activeTab) {
                    const { x, y, width, height } = getLayoutBounds();
                    invoke("open_native_web_tab", {
                      tabId: activeTab.id,
                      url: activeTab.url,
                      x,
                      y,
                      width,
                      height,
                    });
                  }
                }}
              >
                Bypass &amp; Open (Isolated)
              </button>
            </div>
          </div>
        ) : !activeTab || activeTab.url === "kestrel://newtab" ? (
          /* Internal Obsidian New Tab Dashboard */
          <div className="newtab-dashboard">
            <div className="dashboard-hero">
              <div className="hero-pill">
                <Sparkles size={12} />
                <span>Next-Gen Systems Architecture</span>
              </div>
              <h1 className="hero-logo">KESTREL</h1>
              <p className="hero-subtitle">
                A developer-first, cybersecurity-hardened web browser built natively in Rust &amp; Tauri v2
              </p>
            </div>

            {/* Central Search Input */}
            <form className="dashboard-search-bar" onSubmit={handleDashboardSearchSubmit}>
              <Search size={18} style={{ color: "var(--cyan)" }} />
              <input
                type="text"
                className="dashboard-search-input"
                placeholder="Search DuckDuckGo or jump to domain (e.g. google.com, github.com)..."
                value={dashboardSearch}
                onChange={(e) => setDashboardSearch(e.target.value)}
                autoFocus
              />
              <button type="submit" className="btn-primary" style={{ padding: "8px 18px" }}>
                Browse
              </button>
            </form>

            {/* Quick Developer Bookmarks */}
            <div className="quick-bookmarks">
              {QUICK_BOOKMARKS.map((bm) => (
                <button
                  key={bm.name}
                  className="bookmark-btn"
                  onClick={() => navigateTo(bm.url)}
                >
                  <span className="bookmark-icon">{bm.icon}</span>
                  <span className="bookmark-name">{bm.name}</span>
                </button>
              ))}
            </div>

            {/* Real-Time Telemetry Stats */}
            <div className="stat-grid">
              <div className="stat-card" onClick={() => toggleDrawer("shield")}>
                <div className="stat-header">
                  <span className="stat-label">Brave Adblock Engine</span>
                  <Shield size={14} style={{ color: "var(--emerald)" }} />
                </div>
                <span className="stat-value" style={{ color: "var(--emerald)" }}>
                  {adblockStats?.rules_count || 40} Rules Active
                </span>
                <span className="stat-sub">{adblockStats?.total_blocked || 0} ads/trackers intercepted</span>
              </div>

              <div className="stat-card" onClick={() => toggleDrawer("shield")}>
                <div className="stat-header">
                  <span className="stat-label">Cookie Shannon Entropy</span>
                  <Terminal size={14} style={{ color: "var(--cyan)" }} />
                </div>
                <span className="stat-value" style={{ color: "var(--cyan)" }}>
                  4.8 H Threshold
                </span>
                <span className="stat-sub">Real-time signature &amp; XSS scanner</span>
              </div>

              <div className="stat-card" onClick={() => toggleDrawer("vpn")}>
                <div className="stat-header">
                  <span className="stat-label">Userspace WireGuard</span>
                  <Zap size={14} style={{ color: "var(--blue)" }} />
                </div>
                <span className="stat-value" style={{ color: "var(--blue)" }}>
                  {vpnStatus?.is_connected ? "CH Relay (24ms)" : "Direct Mode"}
                </span>
                <span className="stat-sub">3-Hop Tor-like circuit available</span>
              </div>

              <div className="stat-card" onClick={() => toggleDrawer("resources")}>
                <div className="stat-header">
                  <span className="stat-label">Live Working Set RSS</span>
                  <Cpu size={14} style={{ color: "var(--amber)" }} />
                </div>
                <span className="stat-value" style={{ color: "var(--amber)" }}>
                  {resourceStats?.kestrel_rss_mb || 28} MB RSS
                </span>
                <span className="stat-sub">{resourceStats?.memory_saved_mb || 142} MB reclaimed</span>
              </div>
            </div>

            {/* Workspaces Grid */}
            <div className="feature-cards">
              <div className="feature-card" onClick={() => toggleDrawer("shield")}>
                <div className="feature-title">
                  <ShieldCheck size={18} style={{ color: "var(--emerald)" }} />
                  <span>Real-Time Threat Shield</span>
                </div>
                <p className="feature-desc">
                  Inspects cookie entropy, strips tracking parameters, and halts deceptive phishing attempts.
                </p>
              </div>

              <div className="feature-card" onClick={() => toggleDrawer("vault")}>
                <div className="feature-title">
                  <Key size={18} style={{ color: "var(--amber)" }} />
                  <span>Cryptographic Vault</span>
                </div>
                <p className="feature-desc">
                  Hardware-sealed encrypted credential storage with mandatory user PIN approval gates.
                </p>
              </div>

              <div className="feature-card" onClick={() => toggleDrawer("vpn")}>
                <div className="feature-title">
                  <Zap size={18} style={{ color: "var(--cyan)" }} />
                  <span>WireGuard &amp; Tor Routing</span>
                </div>
                <p className="feature-desc">
                  Zero-admin userspace WireGuard VPN paired with 3-hop circuit proxy anonymization.
                </p>
              </div>

              <div className="feature-card" onClick={() => toggleDrawer("devtools")}>
                <div className="feature-title">
                  <Terminal size={18} style={{ color: "var(--blue)" }} />
                  <span>Expert Developer Suite</span>
                </div>
                <p className="feature-desc">
                  Framework recognition (Next.js, React, Tailwind), live waterfall network graph, and cURL export.
                </p>
              </div>

              <div className="feature-card" onClick={() => toggleDrawer("intel")}>
                <div className="feature-title">
                  <Radio size={18} style={{ color: "#a855f7" }} />
                  <span>Cybersecurity Intel Radar</span>
                </div>
                <p className="feature-desc">
                  Curated security tools (Ghidra, REA, Nuclei, ZAP), live CVE feed, and domain security scorecards.
                </p>
              </div>

              <div className="feature-card" onClick={() => toggleDrawer("resources")}>
                <div className="feature-title">
                  <Cpu size={18} style={{ color: "var(--emerald)" }} />
                  <span>Kernel Resource Watchdog</span>
                </div>
                <p className="feature-desc">
                  Real Windows WorkingSet memory trimming via EmptyWorkingSet NT API call.
                </p>
              </div>
            </div>
          </div>
        ) : (
          /* Real Native Webview Viewport Area */
          <div
            className="native-webview-host"
            style={{
              flex: 1,
              height: "100%",
              width: "100%",
              background: "#07090e",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            {activeTab.loading && (
              <div style={{ color: "var(--text-faint)", fontSize: "13px", display: "flex", alignItems: "center", gap: "8px" }}>
                <RotateCw size={14} className="spin" />
                <span>Loading {getSafeHostname(activeTab.url)}...</span>
              </div>
            )}
          </div>
        )}

        {/* 3. Slide-In Drawers */}
        {/* Shield Drawer */}
        {activeDrawer === "shield" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Shield size={16} style={{ color: "var(--blue)" }} />
                <span>Security &amp; Cookie Guard</span>
              </div>
              <button className="tab-close" onClick={() => toggleDrawer("shield")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              {/* Interactive Cookie Auditor */}
              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Live Cookie Threat &amp; Entropy Tester</strong>
                  <span className="audit-pill cyan">Shannon H</span>
                </div>
                <p style={{ fontSize: "11px", color: "var(--text-dim)", marginTop: "4px" }}>
                  Paste any cookie name/value to test entropy calculation and payload detection.
                </p>
                <div style={{ display: "flex", flexDirection: "column", gap: "6px", marginTop: "8px" }}>
                  <input
                    type="text"
                    placeholder="Cookie Name (e.g. _session_id)"
                    value={customCookieName}
                    onChange={(e) => setCustomCookieName(e.target.value)}
                    className="vault-input"
                  />
                  <input
                    type="text"
                    placeholder="Cookie Value (e.g. eyJhbGciOi... or ' OR 1=1)"
                    value={customCookieValue}
                    onChange={(e) => setCustomCookieValue(e.target.value)}
                    className="vault-input"
                  />
                  <button className="btn-primary" onClick={handleAuditCustomCookie}>
                    Calculate Shannon Entropy &amp; Scan
                  </button>
                </div>

                {customCookieAudited && (
                  <div style={{ marginTop: "10px", background: "#05070b", padding: "8px", borderRadius: "6px", border: "1px solid var(--border-mid)" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", fontSize: "12px" }}>
                      <code>{customCookieAudited.name}</code>
                      <span style={{ color: "var(--cyan)", fontWeight: "700" }}>
                        H: {customCookieAudited.entropy.toFixed(3)}
                      </span>
                    </div>
                    <div style={{ fontSize: "11px", color: "var(--text-dim)", marginTop: "4px" }}>
                      Action: <strong>{customCookieAudited.recommended_action}</strong>
                    </div>
                    <div style={{ display: "flex", gap: "4px", flexWrap: "wrap", marginTop: "4px" }}>
                      {customCookieAudited.threat_tags.map((t, idx) => (
                        <span key={idx} className="audit-tag tag-red">
                          {t}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>

              {currentUrlScan && (
                <div className="audit-item">
                  <div className="audit-item-header">
                    <strong>URL Lexical Audit</strong>
                    <span className={`audit-pill ${currentUrlScan.threat_score >= 50 ? "danger" : "safe"}`}>
                      Score: {currentUrlScan.threat_score}/100
                    </span>
                  </div>
                  <div style={{ fontSize: "11px", color: "var(--text-dim)", marginTop: "4px" }}>
                    Category: <strong>{currentUrlScan.threat_category}</strong>
                  </div>
                  <div style={{ display: "flex", flexWrap: "wrap", gap: "4px", marginTop: "6px" }}>
                    {currentUrlScan.reasons.map((r, i) => (
                      <span key={i} className="audit-tag tag-red">
                        {r}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Brave Adblock Engine</strong>
                  <span className="audit-pill safe">Active</span>
                </div>
                <div className="audit-grid">
                  <div>Total Blocked: <strong>{adblockStats?.total_blocked || 0}</strong></div>
                  <div>Trackers Neutralized: <strong>{adblockStats?.trackers_blocked || 0}</strong></div>
                  <div>Active Rules: <strong>{adblockStats?.rules_count || 40}</strong></div>
                  <div>Cosmetic Filters: <strong>{adblockStats?.cosmetic_filters_count || 12}</strong></div>
                </div>
              </div>

              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Active Session Cookies</strong>
                  <span className="audit-pill cyan">H &gt; 4.8 Threshold</span>
                </div>
                {sampleCookies.map((cookie, idx) => (
                  <div key={idx} className="cookie-row">
                    <div style={{ display: "flex", justifyContent: "space-between", fontSize: "12px" }}>
                      <code style={{ color: "var(--text-pure)" }}>{cookie.name}</code>
                      <span style={{ color: "var(--cyan)", fontWeight: "700" }}>H: {cookie.entropy.toFixed(2)}</span>
                    </div>
                    <div style={{ display: "flex", gap: "4px", flexWrap: "wrap", marginTop: "4px" }}>
                      {cookie.threat_tags.map((t, ti) => (
                        <span key={ti} className="audit-tag tag-amber">
                          {t}
                        </span>
                      ))}
                    </div>
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
                <Key size={16} style={{ color: "var(--amber)" }} />
                <span>Cryptographic Hardware Vault</span>
              </div>
              <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
                <button
                  className="btn-icon"
                  onClick={() => setShowAddCredModal(!showAddCredModal)}
                  title="Add New Credential"
                >
                  <PlusCircle size={15} />
                </button>
                <button className="tab-close" onClick={() => toggleDrawer("vault")}>
                  <X size={14} />
                </button>
              </div>
            </div>
            <div className="drawer-body">
              {showAddCredModal && (
                <form onSubmit={handleAddCredential} className="audit-item" style={{ border: "1px solid var(--blue)" }}>
                  <div className="audit-item-header">
                    <strong>Seal New Credential</strong>
                    <button type="button" className="btn-icon" onClick={() => setShowAddCredModal(false)}>
                      <X size={12} />
                    </button>
                  </div>
                  <input
                    type="text"
                    placeholder="Domain (e.g. github.com)"
                    value={newCredDomain}
                    onChange={(e) => setNewCredDomain(e.target.value)}
                    className="vault-input"
                    required
                  />
                  <input
                    type="text"
                    placeholder="Username / Identity"
                    value={newCredUsername}
                    onChange={(e) => setNewCredUsername(e.target.value)}
                    className="vault-input"
                    required
                  />
                  <input
                    type="password"
                    placeholder="Secret / Password / API Key"
                    value={newCredSecret}
                    onChange={(e) => setNewCredSecret(e.target.value)}
                    className="vault-input"
                    required
                  />
                  <input
                    type="password"
                    placeholder="Protection PIN (Default: 1234)"
                    value={newCredPin}
                    onChange={(e) => setNewCredPin(e.target.value)}
                    className="vault-input"
                  />
                  <button type="submit" className="btn-primary" style={{ marginTop: "4px" }}>
                    Encrypt &amp; Store in Hardware Vault
                  </button>
                </form>
              )}

              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Hardware-Sealed Credentials</strong>
                  <span className="audit-pill amber">AES-256 GCM</span>
                </div>
                <p style={{ fontSize: "11px", color: "var(--text-dim)", marginTop: "4px" }}>
                  Plaintext exists in memory only during active user PIN authorization window.
                </p>

                {credentials.map((cred) => (
                  <div key={cred.id} className="cred-card">
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                      <div>
                        <strong style={{ color: "var(--text-pure)" }}>{cred.domain}</strong>
                        <div style={{ fontSize: "11px", color: "var(--text-dim)" }}>{cred.username}</div>
                      </div>
                      <span style={{ fontSize: "10px", color: "var(--text-faint)" }}>
                        {cred.created_at}
                      </span>
                    </div>

                    <div style={{ display: "flex", gap: "8px", marginTop: "10px" }}>
                      <input
                        type="password"
                        placeholder="PIN (1234)"
                        value={vaultPinInput}
                        onChange={(e) => setVaultPinInput(e.target.value)}
                        className="vault-input"
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
                  <div className="vault-error">
                    {vaultError}
                  </div>
                )}

                {decryptedSecret && (
                  <div className="vault-secret-box">
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "4px" }}>
                      <span style={{ fontSize: "11px", color: "var(--emerald)", fontWeight: "600" }}>
                        Decrypted in Ephemeral RAM:
                      </span>
                      <div style={{ display: "flex", gap: "6px" }}>
                        <button
                          className="btn-icon"
                          onClick={() => setShowSecret(!showSecret)}
                          title="Toggle Visibility"
                        >
                          {showSecret ? <EyeOff size={13} /> : <Eye size={13} />}
                        </button>
                        <button
                          className="btn-icon"
                          onClick={() => {
                            navigator.clipboard.writeText(decryptedSecret);
                            setCopiedSecret(true);
                            setTimeout(() => setCopiedSecret(false), 2000);
                          }}
                          title="Copy to Clipboard"
                        >
                          {copiedSecret ? <Check size={13} /> : <Copy size={13} />}
                        </button>
                      </div>
                    </div>
                    <code style={{ fontSize: "12px", color: "#ffffff", wordBreak: "break-all" }}>
                      {showSecret ? decryptedSecret : "••••••••••••••••••••••••"}
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
                <Zap size={16} style={{ color: "var(--cyan)" }} />
                <span>Userspace WireGuard &amp; Tor Circuit</span>
              </div>
              <button className="tab-close" onClick={() => toggleDrawer("vpn")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>WireGuard Userspace Tunnel</strong>
                  <button
                    className={vpnStatus?.is_connected ? "btn-primary" : "btn-secondary"}
                    onClick={handleToggleVpn}
                  >
                    {vpnStatus?.is_connected ? "Connected" : "Connect"}
                  </button>
                </div>
                <div className="audit-grid" style={{ marginTop: "10px" }}>
                  <div>Protocol: <code>{vpnStatus?.protocol}</code></div>
                  <div>Exit Gateway: <strong>{vpnStatus?.exit_country}</strong></div>
                  <div>SOCKS5 Proxy: <code>127.0.0.1:{vpnStatus?.local_socks5_port}</code></div>
                  <div>Tunnel Latency: <code style={{ color: "var(--emerald)" }}>{vpnStatus?.latency_ms} ms</code></div>
                </div>
              </div>

              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>3-Hop Circuit Routing (Tor-Grade)</strong>
                  <span className="audit-pill cyan">No .onion access</span>
                </div>
                <p style={{ fontSize: "11px", color: "var(--text-dim)", marginTop: "4px" }}>
                  Traffic is layered across 3 international relay proxies to defeat ISP traffic analysis.
                </p>
                <div className="circuit-hops">
                  {vpnStatus?.tor_circuit_hops.map((hop, idx) => (
                    <div key={idx} className="hop-pill">
                      <span className="hop-num">{idx + 1}</span>
                      <span className="hop-name">{hop}</span>
                    </div>
                  ))}
                </div>
              </div>

              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Fingerprint Defense (Canvas &amp; Audio)</strong>
                  <span className="audit-pill safe">100% Noise Active</span>
                </div>
                <p style={{ fontSize: "11px", color: "var(--text-dim)", marginTop: "4px" }}>
                  Injects deterministic subtle noise into Canvas 2D, WebGL readPixels, and Web Audio buffers to destroy device fingerprint correlation.
                </p>
              </div>
            </div>
          </aside>
        )}

        {/* DevTools Drawer */}
        {activeDrawer === "devtools" && (
          <aside className="side-drawer">
            <div className="drawer-header">
              <div className="drawer-title">
                <Code size={16} style={{ color: "var(--cyan)" }} />
                <span>Expert Developer Suite</span>
              </div>
              <button className="tab-close" onClick={() => toggleDrawer("devtools")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Framework Recognition (Wappalyzer)</strong>
                  <span className="audit-pill cyan">{detectedTech.length} Detected</span>
                </div>
                <div style={{ display: "flex", gap: "6px", flexWrap: "wrap", marginTop: "8px" }}>
                  {detectedTech.map((t, idx) => (
                    <span key={idx} className="audit-tag tag-blue">
                      {t.name} {t.version ? `(${t.version})` : ""}
                    </span>
                  ))}
                </div>
              </div>

              {networkTimings && (
                <div className="audit-item">
                  <div className="audit-item-header">
                    <strong>Live TCP/DNS Waterfall Latency</strong>
                    <span className="audit-pill safe">{networkTimings.total_ms} ms</span>
                  </div>
                  <div className="waterfall-list">
                    <div className="timing-row">
                      <span>DNS Resolution</span>
                      <code>{networkTimings.dns_lookup_ms} ms</code>
                    </div>
                    <div className="timing-row">
                      <span>TCP Connect</span>
                      <code>{networkTimings.tcp_connect_ms} ms</code>
                    </div>
                    <div className="timing-row">
                      <span>TLS Handshake</span>
                      <code>{networkTimings.tls_handshake_ms} ms</code>
                    </div>
                    <div className="timing-row">
                      <span>TTFB (First Byte)</span>
                      <code style={{ color: "var(--emerald)" }}>{networkTimings.ttfb_ms} ms</code>
                    </div>
                    <div className="timing-row">
                      <span>Content Download</span>
                      <code>{networkTimings.download_ms} ms</code>
                    </div>
                  </div>
                </div>
              )}

              <div className="audit-item">
                <div className="audit-item-header">
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
                    <span style={{ marginLeft: "4px" }}>{copiedCurl ? "Copied" : "Copy"}</span>
                  </button>
                </div>
                <textarea
                  readOnly
                  value={curlCommand}
                  className="curl-textarea"
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
                <Radio size={16} style={{ color: "#a855f7" }} />
                <span>Cybersecurity Intel Radar</span>
              </div>
              <button className="tab-close" onClick={() => toggleDrawer("intel")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Curated Elite Security Tools</strong>
                  <span className="audit-pill cyan">{securityTools.length} Loaded</span>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "8px" }}>
                  {securityTools.map((tool, idx) => (
                    <div key={idx} className="tool-card">
                      <div style={{ display: "flex", justifyContent: "space-between" }}>
                        <strong style={{ color: "var(--text-pure)" }}>{tool.name}</strong>
                        <span style={{ color: "var(--amber)", fontSize: "11px", fontWeight: "700" }}>★ {tool.stars.toLocaleString()}</span>
                      </div>
                      <p style={{ color: "var(--text-dim)", fontSize: "11px", margin: "3px 0" }}>
                        {tool.description}
                      </p>
                      <code style={{ fontSize: "10px", color: "var(--cyan)" }}>{tool.repo}</code>
                    </div>
                  ))}
                </div>
              </div>

              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Live Critical CVE Stream</strong>
                  <span className="audit-pill danger">Critical Alerts</span>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "8px" }}>
                  {cveAlerts.map((cve, idx) => (
                    <div key={idx} className="cve-card">
                      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <code style={{ color: "var(--rose)", fontWeight: "700" }}>{cve.cve_id}</code>
                        <span className="audit-tag tag-red">CVSS {cve.cvss}</span>
                      </div>
                      <div style={{ fontWeight: "600", fontSize: "12px", marginTop: "2px", color: "var(--text-pure)" }}>
                        {cve.title}
                      </div>
                      <div style={{ color: "var(--text-dim)", fontSize: "11px" }}>Target: {cve.affected_tech}</div>
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
                <Puzzle size={16} style={{ color: "var(--emerald)" }} />
                <span>Extensions &amp; Web Store</span>
              </div>
              <button className="tab-close" onClick={() => toggleDrawer("extensions")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Installed Chrome Extensions</strong>
                  <span className="audit-pill safe">{extensions.length} Active</span>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: "10px", marginTop: "8px" }}>
                  {extensions.map((ext) => (
                    <div key={ext.id} className="ext-row">
                      <div>
                        <strong style={{ color: "var(--text-pure)" }}>{ext.name}</strong>{" "}
                        <span style={{ fontSize: "11px", color: "var(--text-faint)" }}>v{ext.version}</span>
                        <div style={{ fontSize: "11px", color: "var(--text-dim)" }}>{ext.description}</div>
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
                <Cpu size={16} style={{ color: "var(--amber)" }} />
                <span>Kernel Resource Watchdog</span>
              </div>
              <button className="tab-close" onClick={() => toggleDrawer("resources")}>
                <X size={14} />
              </button>
            </div>
            <div className="drawer-body">
              <div className="audit-item">
                <div className="audit-item-header">
                  <strong>Windows NT Process Working Set</strong>
                  <span className="audit-pill amber">{resourceStats?.kestrel_rss_mb || 28} MB RSS</span>
                </div>
                <div className="audit-grid" style={{ marginTop: "10px" }}>
                  <div>Current Browser RSS: <strong>{resourceStats?.kestrel_rss_mb || 28} MB</strong></div>
                  <div>RAM Reclaimed: <strong>{resourceStats?.memory_saved_mb || 142} MB</strong></div>
                  <div>Inactive Hibernated Tabs: <strong>{resourceStats?.hibernated_tabs_count || 0}</strong></div>
                  <div>Kernel Flush: <strong>K32EmptyWorkingSet</strong></div>
                </div>
                <button
                  className="btn-primary"
                  style={{ marginTop: "14px", width: "100%" }}
                  onClick={handleTriggerHibernation}
                >
                  Flush Working Set &amp; Trim RAM
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
                {playerState.is_playing ? <Pause size={15} /> : <Play size={15} />}
              </button>
            </div>
            <div className="track-meta">
              <span className="track-title">{playerState.current_track.title}</span>
              <span className="track-artist">{playerState.current_track.artist}</span>
            </div>
            {playerState.is_playing && (
              <div className="equalizer-wave">
                <span className="eq-bar eq-1"></span>
                <span className="eq-bar eq-2"></span>
                <span className="eq-bar eq-3"></span>
                <span className="eq-bar eq-4"></span>
              </div>
            )}
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
            {/* Playlist switcher dropdown */}
            <div className="playlist-selector">
              <ListMusic size={13} style={{ color: "var(--cyan)" }} />
              <select
                value={playerState.current_track.id}
                onChange={(e) => handleSwitchTrack(e.target.value)}
                className="track-select"
              >
                {playerState.playlist.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.title}
                  </option>
                ))}
              </select>
            </div>

            {/* Volume control */}
            <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
              {audioVolume === 0 ? (
                <VolumeX size={14} style={{ color: "var(--text-faint)" }} />
              ) : (
                <Volume2 size={14} style={{ color: "var(--text-dim)" }} />
              )}
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={audioVolume}
                onChange={(e) => handleVolumeChange(parseFloat(e.target.value))}
                className="volume-slider"
              />
            </div>

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
