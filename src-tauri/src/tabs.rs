use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State, WebviewBuilder, WebviewUrl, LogicalPosition, LogicalSize};

fn normalize_target_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "kestrel://newtab" {
        return "kestrel://newtab".to_string();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") || trimmed.starts_with("kestrel://") {
        trimmed.to_string()
    } else if trimmed.contains('.') && !trimmed.contains(' ') {
        format!("https://{}", trimmed)
    } else {
        format!("https://duckduckgo.com/?q={}", trimmed.replace(' ', "+"))
    }
}

#[tauri::command]
pub async fn open_native_web_tab(
    app: AppHandle,
    tab_id: String,
    url: String,
    x: Option<f64>,
    y: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    let normalized = normalize_target_url(&url);

    // If it's internal dashboard, hide all child webviews and exit cleanly
    if normalized.starts_with("kestrel://") {
        for (label, wv) in app.webviews() {
            if label != "main" {
                let _ = wv.hide();
            }
        }
        return Ok(());
    }

    let parsed_url: tauri::Url = normalized.parse().map_err(|e| format!("Invalid URL {}: {}", normalized, e))?;
    
    // Hide other child webviews
    for (label, wv) in app.webviews() {
        if label != "main" && label != tab_id {
            let _ = wv.hide();
        }
    }

    let pos_x = x.unwrap_or(0.0);
    let pos_y = y.unwrap_or(74.0);
    
    let (target_w, target_h) = if let (Some(w), Some(h)) = (width, height) {
        (w.max(200.0), h.max(200.0))
    } else {
        let win_size = window.inner_size().unwrap_or(tauri::PhysicalSize::new(1920, 1080));
        let scale_factor = window.scale_factor().unwrap_or(1.0);
        let logical_width = (win_size.width as f64) / scale_factor;
        let logical_height = (win_size.height as f64) / scale_factor;
        (logical_width.max(800.0), (logical_height - 74.0 - 48.0).max(400.0))
    };

    // Injected adblock, YouTube ad annihilator, and tracker defense script
    let init_script = r#"
    (function() {
        if (window.__KESTREL_ADBLOCK_INITIALIZED__) return;
        window.__KESTREL_ADBLOCK_INITIALIZED__ = true;

        // 1. UNIVERSAL JSON & RESPONSE AD PRUNER (uBlock Origin / Brave architecture)
        // Automatically strips ad breaks from any player JSON without network delays or stream corruption
        const _origParse = JSON.parse;
        JSON.parse = function(...args) {
            const data = _origParse.apply(this, args);
            if (data && typeof data === 'object') {
                try {
                    delete data.adPlacements;
                    delete data.adSlots;
                    delete data.playerAds;
                    delete data.adBreakHeartbeatParams;
                    if (data.playerResponse && typeof data.playerResponse === 'object') {
                        delete data.playerResponse.adPlacements;
                        delete data.playerResponse.adSlots;
                        delete data.playerResponse.playerAds;
                        delete data.playerResponse.adBreakHeartbeatParams;
                    }
                } catch(e) {}
            }
            return data;
        };

        // Intercept Response.prototype.json (catches all internal fetch deserializations)
        const _origRespJson = Response.prototype.json;
        Response.prototype.json = async function() {
            const data = await _origRespJson.apply(this, arguments);
            if (data && typeof data === 'object') {
                try {
                    delete data.adPlacements;
                    delete data.adSlots;
                    delete data.playerAds;
                    delete data.adBreakHeartbeatParams;
                    if (data.playerResponse && typeof data.playerResponse === 'object') {
                        delete data.playerResponse.adPlacements;
                        delete data.playerResponse.adSlots;
                        delete data.playerResponse.playerAds;
                        delete data.playerResponse.adBreakHeartbeatParams;
                    }
                } catch(e) {}
            }
            return data;
        };

        // Intercept ytInitialPlayerResponse
        let _ytInitialPlayerResponse = window.ytInitialPlayerResponse;
        try {
            Object.defineProperty(window, 'ytInitialPlayerResponse', {
                get() { return _ytInitialPlayerResponse; },
                set(val) {
                    if (val && typeof val === 'object') {
                        delete val.adPlacements;
                        delete val.adSlots;
                        delete val.playerAds;
                        delete val.adBreakHeartbeatParams;
                    }
                    _ytInitialPlayerResponse = val;
                },
                configurable: true
            });
        } catch(e) {}

        // 2. THIRD-PARTY TRACKER DOMAINS
        const AD_DOMAINS = [
            "doubleclick.net", "googleads.g.doubleclick.net", "pagead2.googlesyndication.com",
            "adservice.google.com", "adsystem.amazon.com", "adnxs.com", "criteo.com",
            "advertising.com", "bidswitch.net", "casale-media.com", "taboola.com",
            "outbrain.com", "popcash.net", "popads.net", "adcolony.com", "chartbeat.net",
            "scorecardresearch.com", "quantserve.com", "zedo.com", "rubiconproject.com",
            "analytics.google.com", "google-analytics.com", "hotjar.com", "fullstory.com",
            "segment.io", "mixpanel.com", "mouseflow.com", "crazyegg.com", "inspectlet.com",
            "facebook.com/tr", "connect.facebook.net", "pixel.wp.com", "statcounter.com"
        ];

        function isThirdPartyAd(u) {
            if (!u) return false;
            try {
                const s = String(u).toLowerCase();
                // Never touch YouTube video streams or player resources
                if (s.includes("youtube.com") || s.includes("googlevideo.com") || s.includes("ytimg.com")) {
                    return false;
                }
                for (let i = 0; i < AD_DOMAINS.length; i++) {
                    if (s.includes(AD_DOMAINS[i])) return true;
                }
            } catch(e) {}
            return false;
        }

        // Intercept Fetch - drop third-party ad beacons without stalling legitimate streams
        const _origFetch = window.fetch;
        window.fetch = function(...args) {
            const url = (typeof args[0] === 'string') ? args[0] : (args[0] && args[0].url) || '';

            if (url.includes('/api/stats/ads') || url.includes('/pagead/') || isThirdPartyAd(url)) {
                return Promise.resolve(new Response(JSON.stringify({ blocked: true }), {
                    status: 200,
                    headers: { 'Content-Type': 'application/json' }
                }));
            }

            return _origFetch.apply(this, args);
        };

        // Intercept XHR
        const _origOpen = XMLHttpRequest.prototype.open;
        XMLHttpRequest.prototype.open = function(method, url, ...rest) {
            if (isThirdPartyAd(url) || (typeof url === 'string' && (url.includes('/pagead/') || url.includes('/api/stats/ads')))) {
                url = 'data:application/json,{"blocked":true}';
            }
            return _origOpen.call(this, method, url, ...rest);
        };

        // 3. ZERO-DELAY AD DEFEATER (Eliminates ad overlays and clicks skip instantly)
        function sweepYouTubeAds() {
            if (!location.hostname.includes("youtube.com")) return;

            // Auto-click any skip buttons immediately
            const skipButtons = document.querySelectorAll(
                '.ytp-skip-ad-button, .ytp-ad-skip-button, .ytp-ad-skip-button-modern, .ytp-ad-skip-button-slot, .videoAdUiSkipButton, button.ytp-ad-skip-button-text'
            );
            skipButtons.forEach(b => {
                try { b.click(); } catch(e) {}
            });

            const player = document.getElementById('movie_player') || document.querySelector('.html5-video-player');
            if (player) {
                const isAdShowing = player.classList.contains('ad-showing') || player.classList.contains('ad-interrupting');
                const video = player.querySelector('video') || document.querySelector('video.html5-main-video');

                if (isAdShowing) {
                    // 1. Invoke YouTube native player skipAd API if available
                    if (typeof player.skipAd === 'function') {
                        try { player.skipAd(); } catch(e) {}
                    }
                    if (video) {
                        video.muted = true;
                        video.playbackRate = 16.0;
                        // Only fast forward if it is genuinely a short ad clip (duration <= 90s)
                        const adOverlay = document.querySelector('.ytp-ad-player-overlay, .ytp-ad-text, .ytp-ad-preview-text, .ytp-ad-duration-remaining');
                        if (adOverlay && isFinite(video.duration) && video.duration > 0 && video.duration <= 90) {
                            video.currentTime = Math.max(0, video.duration - 0.05);
                        }
                    }
                    const btn = document.querySelector('.ytp-skip-ad-button, .ytp-ad-skip-button, .ytp-ad-skip-button-modern, .ytp-ad-skip-button-slot');
                    if (btn) try { btn.click(); } catch(e) {}
                } else if (video) {
                    // Normal main video playback: ensure standard 1.0 rate and restore audio
                    if (video.playbackRate > 2.0) {
                        video.playbackRate = 1.0;
                    }
                    video.muted = false;
                }
            }

            // Remove banner and overlay ad nodes
            const bannerAds = document.querySelectorAll(
                'ytd-ad-slot-renderer, ytd-promoted-video-renderer, ytd-display-ad-renderer, ytd-banner-promo-renderer, #player-ads, #masthead-ad, .ytp-ad-overlay-container, #offer-module, tp-yt-paper-dialog:has(.ytd-ad-slot-renderer)'
            );
            bannerAds.forEach(el => {
                try { el.remove(); } catch(e) {}
            });
        }

        setInterval(sweepYouTubeAds, 100);

        // 4. COSMETIC CSS INJECTION - Hides all ad frames AND the yellow timeline bar
        function injectCosmeticCSS() {
            if (document.getElementById('kestrel-adblock-css')) return;
            const style = document.createElement('style');
            style.id = 'kestrel-adblock-css';
            style.textContent = `
                .video-ads, .ytp-ad-module, .ytp-ad-player-overlay,
                .ytp-ad-overlay-container, .ytp-ad-message-container,
                .ytp-ad-progress-list, .ytp-ad-progress, .ytp-ad-duration-remaining,
                ytd-ad-slot-renderer, ytd-rich-item-renderer:has(ytd-ad-slot-renderer),
                ytd-promoted-video-renderer, ytd-display-ad-renderer,
                ytd-banner-promo-renderer, ytd-statement-banner-renderer,
                #player-ads, #masthead-ad, #offer-module,
                .adsbygoogle, .ad-container, [id^="div-gpt-ad"] {
                    display: none !important;
                    visibility: hidden !important;
                    height: 0 !important;
                    width: 0 !important;
                    opacity: 0 !important;
                    pointer-events: none !important;
                }
                .ad-showing .ytp-play-progress {
                    background: #89b4fa !important;
                }
            `;
            (document.head || document.documentElement).appendChild(style);
        }

        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', injectCosmeticCSS);
        } else {
            injectCosmeticCSS();
        }
    })();
    "#;

    // Check if webview already exists, show and navigate immediately
    if let Some(existing) = app.get_webview(&tab_id) {
        let _ = existing.show();
        let _ = existing.set_position(LogicalPosition::new(pos_x, pos_y));
        let _ = existing.set_size(LogicalSize::new(target_w, target_h));
        let _ = existing.navigate(parsed_url);
        let _ = existing.eval(init_script);
        return Ok(());
    }

    let app_handle = app.clone();
    let tid = tab_id.clone();
    // Anchor below the 74px browser toolbar and above the 48px player bar
    let webview = WebviewBuilder::new(&tab_id, WebviewUrl::External(parsed_url))
        .initialization_script(init_script)
        .on_navigation(move |url| {
            let u_str = url.to_string();
            let _ = app_handle.emit("child-navigated", serde_json::json!({
                "tabId": tid,
                "url": u_str
            }));
            true
        });
        
    window.add_child(
        webview,
        LogicalPosition::new(pos_x, pos_y),
        LogicalSize::new(target_w, target_h),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_native_web_tab_bounds(app: AppHandle, tab_id: String, x: f64, y: f64, width: f64, height: f64) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_id) {
        let _ = wv.set_position(LogicalPosition::new(x, y));
        let _ = wv.set_size(LogicalSize::new(width.max(100.0), height.max(100.0)));
    }
    Ok(())
}

#[tauri::command]
pub fn reload_native_web_tab(app: AppHandle, tab_id: String) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_id) {
        let _ = wv.eval("window.location.reload()");
    }
    Ok(())
}

#[tauri::command]
pub fn navigate_native_web_tab_back(app: AppHandle, tab_id: String) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_id) {
        let _ = wv.eval("window.history.back()");
    }
    Ok(())
}

#[tauri::command]
pub fn navigate_native_web_tab_forward(app: AppHandle, tab_id: String) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_id) {
        let _ = wv.eval("window.history.forward()");
    }
    Ok(())
}

#[tauri::command]
pub fn hide_native_web_tabs(app: AppHandle) -> Result<(), String> {
    for (label, wv) in app.webviews() {
        if label != "main" {
            let _ = wv.hide();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn close_native_web_tab(app: AppHandle, tab_id: String) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_id) {
        let _ = wv.close();
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub security_score: u8,
    pub blocked_trackers: u32,
    pub is_private: bool,
}

pub struct TabState {
    pub tabs: Mutex<Vec<Tab>>,
    pub active_tab_id: Mutex<String>,
}

impl TabState {
    pub fn new() -> Self {
        let initial_tab = Tab {
            id: "tab-1".to_string(),
            url: "kestrel://newtab".to_string(),
            title: "New Tab".to_string(),
            loading: false,
            can_go_back: false,
            can_go_forward: false,
            security_score: 100,
            blocked_trackers: 0,
            is_private: false,
        };

        Self {
            tabs: Mutex::new(vec![initial_tab.clone()]),
            active_tab_id: Mutex::new(initial_tab.id),
        }
    }
}

#[tauri::command]
pub fn get_tabs(state: State<'_, TabState>) -> Result<Vec<Tab>, String> {
    let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    Ok(tabs.clone())
}

#[tauri::command]
pub fn get_active_tab_id(state: State<'_, TabState>) -> Result<String, String> {
    let active_id = state.active_tab_id.lock().map_err(|e| e.to_string())?;
    Ok(active_id.clone())
}

#[tauri::command]
pub fn create_tab(url: Option<String>, state: State<'_, TabState>) -> Result<Tab, String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let mut active_id = state.active_tab_id.lock().map_err(|e| e.to_string())?;

    let tab_id = format!("tab-{}", tabs.len() + 1);
    let target_url = url.unwrap_or_else(|| "kestrel://newtab".to_string());
    
    let new_tab = Tab {
        id: tab_id.clone(),
        url: target_url,
        title: "New Tab".to_string(),
        loading: false,
        can_go_back: false,
        can_go_forward: false,
        security_score: 100,
        blocked_trackers: 0,
        is_private: false,
    };

    tabs.push(new_tab.clone());
    *active_id = tab_id;

    Ok(new_tab)
}

#[tauri::command]
pub fn close_tab(tab_id: String, state: State<'_, TabState>) -> Result<Vec<Tab>, String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let mut active_id = state.active_tab_id.lock().map_err(|e| e.to_string())?;

    if tabs.len() <= 1 {
        // Always keep at least one tab open
        if let Some(tab) = tabs.first_mut() {
            tab.url = "kestrel://newtab".to_string();
            tab.title = "New Tab".to_string();
            tab.blocked_trackers = 0;
            tab.security_score = 100;
        }
        return Ok(tabs.clone());
    }

    tabs.retain(|t| t.id != tab_id);

    // If the active tab was closed, switch to the last tab
    if *active_id == tab_id {
        if let Some(last_tab) = tabs.last() {
            *active_id = last_tab.id.clone();
        }
    }

    Ok(tabs.clone())
}

#[tauri::command]
pub fn switch_tab(tab_id: String, state: State<'_, TabState>) -> Result<String, String> {
    let mut active_id = state.active_tab_id.lock().map_err(|e| e.to_string())?;
    *active_id = tab_id.clone();
    Ok(active_id.clone())
}

#[tauri::command]
pub fn navigate_tab(tab_id: String, url: String, state: State<'_, TabState>) -> Result<Tab, String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;

    if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
        let mut clean_url = url.trim().to_string();
        
        // Auto-prefix http/https if missing and not a kestrel internal URL
        if !clean_url.starts_with("http://") 
            && !clean_url.starts_with("https://") 
            && !clean_url.starts_with("kestrel://") 
        {
            if clean_url.contains('.') && !clean_url.contains(' ') {
                clean_url = format!("https://{}", clean_url);
            } else {
                // Search query
                clean_url = format!("https://duckduckgo.com/?q={}", clean_url.replace(' ', "+"));
            }
        }

        tab.url = clean_url.clone();
        tab.title = if clean_url.starts_with("https://") || clean_url.starts_with("http://") {
            clean_url.clone()
        } else {
            "New Tab".to_string()
        };
        tab.loading = true;

        Ok(tab.clone())
    } else {
        Err(format!("Tab {} not found", tab_id))
    }
}

#[tauri::command]
pub fn update_tab_title(tab_id: String, title: String, state: State<'_, TabState>) -> Result<(), String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
        tab.title = title;
    }
    Ok(())
}
