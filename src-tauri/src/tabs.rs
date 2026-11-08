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

    // Injected Brave-grade adblock, YouTube ad annihilator, and tracker defense script
    let init_script = r#"
    (function() {
        if (window.__KESTREL_BRAVE_ADBLOCK__) return;
        window.__KESTREL_BRAVE_ADBLOCK__ = true;

        const isYouTube = location.hostname.includes("youtube.com");

        if (isYouTube) {
            // =========================================================================
            // BRAVE-GRADE YOUTUBE AD ANNIHILATOR & SABR DEFUSER
            // =========================================================================

            // Helper to deeply prune ad metadata (uBlock Origin / Brave json-prune engine)
            function pruneAdPayload(obj) {
                if (!obj || typeof obj !== 'object') return;
                
                // Root ad keys
                delete obj.adPlacements;
                delete obj.adSlots;
                delete obj.playerAds;
                delete obj.adBreakHeartbeatParams;
                delete obj.instreamAdPlayerConfig;
                delete obj.adParams;

                // Nested playerResponse ad structures
                if (obj.playerResponse && typeof obj.playerResponse === 'object') {
                    delete obj.playerResponse.adPlacements;
                    delete obj.playerResponse.adSlots;
                    delete obj.playerResponse.playerAds;
                    delete obj.playerResponse.adBreakHeartbeatParams;
                    delete obj.playerResponse.instreamAdPlayerConfig;
                }

                // Tracking and ad beacon telemetry endpoints
                if (obj.playbackTracking && typeof obj.playbackTracking === 'object') {
                    delete obj.playbackTracking.videostatsPlaybackUrl;
                    delete obj.playbackTracking.videostatsDelayplayUrl;
                    delete obj.playbackTracking.videostatsWatchtimeUrl;
                    delete obj.playbackTracking.ptrackingUrl;
                    delete obj.playbackTracking.qoeUrl;
                    delete obj.playbackTracking.atrUrl;
                }
            }

            // 1. JSON.parse Interceptor (Catches desktop SSR and client-side player parses)
            const _origJSONParse = JSON.parse;
            JSON.parse = function(...args) {
                const result = _origJSONParse.apply(this, args);
                if (result && typeof result === 'object') {
                    pruneAdPayload(result);
                }
                return result;
            };

            // 2. Fetch API Interceptor & Network Ad Blocker
            const _origFetch = window.fetch;
            window.fetch = async function(...args) {
                const url = typeof args[0] === 'string' ? args[0] : (args[0] && args[0].url ? args[0].url : '');
                
                // Immediately reject Google Ad and DoubleClick network calls with clean 204 No Content
                if (
                    url.includes('googleads.g.doubleclick.net') ||
                    url.includes('pagead2.googlesyndication.com') ||
                    url.includes('/pagead/') ||
                    url.includes('doubleclick.net') ||
                    url.includes('/api/stats/ads') ||
                    url.includes('/api/stats/qoe?ad_') ||
                    url.includes('/ptracking') ||
                    url.includes('/get_midroll_info')
                ) {
                    return new Response('', { status: 204 });
                }

                // Intercept player responses
                if (url.includes('/youtubei/v1/player')) {
                    try {
                        const response = await _origFetch.apply(this, args);
                        const clone = response.clone();
                        const data = await clone.json();
                        if (data && typeof data === 'object') {
                            pruneAdPayload(data);
                            return new Response(JSON.stringify(data), {
                                status: response.status,
                                statusText: response.statusText,
                                headers: response.headers
                            });
                        }
                        return response;
                    } catch(e) {
                        return _origFetch.apply(this, args);
                    }
                }
                return _origFetch.apply(this, args);
            };

            // 3. XMLHttpRequest Interceptor (Blocks XHR beacons & DoubleClick ad calls without throwing abort errors)
            const _origXHROpen = XMLHttpRequest.prototype.open;
            const _origXHRSend = XMLHttpRequest.prototype.send;
            XMLHttpRequest.prototype.open = function(method, url, ...rest) {
                this._kestrel_ad_url = typeof url === 'string' ? url : '';
                return _origXHROpen.apply(this, [method, url, ...rest]);
            };
            XMLHttpRequest.prototype.send = function(...args) {
                if (this._kestrel_ad_url) {
                    const u = this._kestrel_ad_url;
                    if (
                        u.includes('googleads.g.doubleclick.net') ||
                        u.includes('pagead2.googlesyndication.com') ||
                        u.includes('/pagead/') ||
                        u.includes('doubleclick.net') ||
                        u.includes('/api/stats/ads') ||
                        u.includes('/ptracking')
                    ) {
                        try {
                            Object.defineProperty(this, 'status', { value: 204, configurable: true });
                            Object.defineProperty(this, 'readyState', { value: 4, configurable: true });
                            Object.defineProperty(this, 'responseText', { value: '', configurable: true });
                            if (typeof this.onload === 'function') {
                                this.onload(new ProgressEvent('load'));
                            }
                        } catch(e) {}
                        return;
                    }
                }
                return _origXHRSend.apply(this, args);
            };

            // 4. ytInitialPlayerResponse Property Trap (Brave set-constant / json-prune standard)
            let _ytInitial = window.ytInitialPlayerResponse;
            try {
                Object.defineProperty(window, 'ytInitialPlayerResponse', {
                    get() { return _ytInitial; },
                    set(val) {
                        pruneAdPayload(val);
                        _ytInitial = val;
                    },
                    configurable: true
                });
            } catch(e) {}

            // 5. Disable YouTube Server-Side Ad Experiment Flags (ytcfg)
            function neuterYtcfg() {
                try {
                    if (window.ytcfg && typeof window.ytcfg.set === 'function') {
                        const _origYtcfgSet = window.ytcfg.set;
                        window.ytcfg.set = function(config, ...rest) {
                            if (config && typeof config === 'object') {
                                if (config.EXPERIMENT_FLAGS) {
                                    config.EXPERIMENT_FLAGS.web_enable_ab_testing = false;
                                    config.EXPERIMENT_FLAGS.html5_enable_embedded_player_visibility_signals = false;
                                    config.EXPERIMENT_FLAGS.html5_ad_timeout_ms = 0;
                                }
                            }
                            return _origYtcfgSet.apply(this, [config, ...rest]);
                        };
                    }
                } catch(e) {}
            }
            neuterYtcfg();

            // 6. Active Video Ad Neutralizer & Anti-Adblock Popup Annihilator
            function executeBraveDefuser() {
                // Anti-adblock modal remover ("Ad blockers violate YouTube's Terms of Service")
                const antiAdblockModal = document.querySelector('ytd-enforcement-message-view-model, tp-yt-paper-dialog:has(#feedback), tp-yt-paper-dialog:has(.ytd-enforcement-message-view-model)');
                if (antiAdblockModal) {
                    antiAdblockModal.remove();
                    document.querySelectorAll('tp-yt-iron-overlay-backdrop').forEach(el => el.remove());
                    const v = document.querySelector('video');
                    if (v && v.paused) {
                        v.play().catch(() => {});
                    }
                }

                const player = document.getElementById('movie_player') || document.querySelector('.html5-video-player');
                if (!player) return;

                // Click native skip buttons immediately (0ms trigger)
                const skipBtn = document.querySelector(
                    '.ytp-skip-ad-button, .ytp-ad-skip-button, .ytp-ad-skip-button-modern, .ytp-ad-skip-button-slot, button.ytp-ad-skip-button, .videoAdUiSkipButton'
                );
                if (skipBtn) {
                    try {
                        skipBtn.click();
                        return;
                    } catch(e) {}
                }

                // If player has internal skipAd API, invoke it
                if (typeof player.skipAd === 'function') {
                    try { player.skipAd(); } catch(e) {}
                }

                // If ad video is playing, mute instantly and advance
                const isAdShowing = player.classList.contains('ad-showing') || player.classList.contains('ad-interrupting');
                if (isAdShowing) {
                    const video = player.querySelector('video') || document.querySelector('video.html5-main-video');
                    if (video) {
                        video.muted = true;
                        video.playbackRate = 16.0;
                        if (isFinite(video.duration) && video.duration > 0) {
                            video.currentTime = video.duration - 0.1;
                        }
                    }
                }
            }

            // High-frequency observer on movie_player
            const playerObserver = new MutationObserver(() => {
                executeBraveDefuser();
            });

            function attachPlayerWatcher() {
                const player = document.getElementById('movie_player') || document.querySelector('.html5-video-player');
                if (player) {
                    playerObserver.observe(player, { attributes: true, childList: true, subtree: true, attributeFilter: ['class'] });
                    executeBraveDefuser();
                } else {
                    setTimeout(attachPlayerWatcher, 200);
                }
            }
            attachPlayerWatcher();
            setInterval(executeBraveDefuser, 400);
        }

        // COSMETIC CSS INJECTION - Scoped strictly to prevent collateral damage on web applications
        function injectCosmeticCSS() {
            if (document.getElementById('kestrel-adblock-css')) return;
            const style = document.createElement('style');
            style.id = 'kestrel-adblock-css';
            if (isYouTube) {
                style.textContent = `
                    .video-ads, .ytp-ad-module, .ytp-ad-player-overlay,
                    .ytp-ad-overlay-container, .ytp-ad-message-container,
                    .ytp-ad-progress-list, .ytp-ad-progress, .ytp-ad-duration-remaining,
                    ytd-ad-slot-renderer, ytd-rich-item-renderer:has(ytd-ad-slot-renderer),
                    ytd-in-feed-ad-layout-renderer, ytd-banner-promo-renderer,
                    ytd-promoted-video-renderer, ytd-display-ad-renderer,
                    ytd-statement-banner-renderer, #player-ads, #masthead-ad, #offer-module,
                    .ytp-ad-preview-container, .ytp-ad-skip-button-container,
                    ytd-enforcement-message-view-model,
                    tp-yt-paper-dialog:has(.ytd-enforcement-message-view-model) {
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
            } else {
                style.textContent = `
                    .adsbygoogle, [id^="div-gpt-ad"], [data-ad-slot], [data-ad-client] {
                        display: none !important;
                        visibility: hidden !important;
                        height: 0 !important;
                        width: 0 !important;
                        opacity: 0 !important;
                        pointer-events: none !important;
                    }
                `;
            }
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
        return Ok(());
    }

    let app_handle = app.clone();
    let tid = tab_id.clone();
    // Anchor below the 74px browser toolbar and above the 48px player bar
    // Uses native WebView2 User-Agent to avoid CDN/Cloudflare/Wikimedia 429 rate limit blocks
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
pub fn toggle_native_reader_mode(app: AppHandle, tab_id: String, enabled: bool) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_id) {
        if enabled {
            let script = r#"
            (function() {
                try {
                    let overlay = document.getElementById('kestrel-reader-overlay');
                    if (overlay) overlay.remove();

                    // Hide all other children of body to eliminate floating sidebars, share buttons, sticky headers
                    document.querySelectorAll('body > *:not(#kestrel-reader-overlay)').forEach(el => {
                        el.setAttribute('data-kestrel-hidden', 'true');
                        el.style.setProperty('display', 'none', 'important');
                    });

                    // 1. Identify primary article container with scoring
                    const candidates = [
                        'article',
                        'main',
                        '[role="main"]',
                        '#mw-content-text',
                        '#content',
                        '.article-content',
                        '.post-content',
                        '.entry-content',
                        '.article-body',
                        '.story-content',
                        '#article-body',
                        '.markdown-body',
                        'body'
                    ];

                    let bestEl = null;
                    let bestScore = 0;
                    for (const sel of candidates) {
                        const els = document.querySelectorAll(sel);
                        for (const el of els) {
                            if (el.id === 'kestrel-reader-overlay') continue;
                            const text = el.innerText || '';
                            const pCount = el.querySelectorAll('p').length;
                            const score = text.length + (pCount * 150);
                            if (score > bestScore) {
                                bestScore = score;
                                bestEl = el;
                            }
                        }
                    }

                    if (!bestEl) bestEl = document.body;

                    // 2. Clone and aggressively strip ads, social widgets, sponsors, and floating elements
                    const clone = bestEl.cloneNode(true);

                    // Deep scrub junk nodes
                    const junkSelectors = [
                        'script', 'style', 'noscript', 'iframe', 'object', 'embed', 'ins',
                        'header', 'footer', 'nav', 'aside',
                        '.ad', '.ads', '.advertisement', '.ad-container', '[class*="advert"]',
                        '.social-share', '.share-buttons', '.share-box', '.share', '[class*="share"]', '[id*="share"]',
                        '.comments', '#comments', '.comment-section',
                        '.navbox', '.vector-menu', '#mw-navigation', '.sidebar',
                        '.cookie-notice', '.banner', '.subscribe-prompt', '.newsletter',
                        '[id*="ad-"]', '[class*="ad-"]', '[id*="ad_"]', '[class*="ad_"]',
                        '[class*="sponsor"]', '[id*="sponsor"]', '[class*="promo"]', '[id*="promo"]',
                        '.separator', '.widget', '#bottom-wrapper', '.bottom-ad',
                        '.ad_wrapper', '.ad-slot', '[id*="google_ads"]', '.post-share-bot',
                        '.share-bar', '.sidebar-share', '[class*="social"]', '[id*="social"]'
                    ];
                    junkSelectors.forEach(s => {
                        clone.querySelectorAll(s).forEach(n => {
                            try { n.remove(); } catch(e) {}
                        });
                    });

                    // Remove any element whose class, id, or content indicates ads or sponsors
                    clone.querySelectorAll('*').forEach(el => {
                        // Force dark readability
                        el.style.setProperty('background-color', 'transparent', 'important');
                        el.style.setProperty('background', 'transparent', 'important');
                        el.style.setProperty('color', '#cdd6f4', 'important');
                        el.style.setProperty('box-shadow', 'none', 'important');

                        const str = ((el.className || '') + ' ' + (el.id || '')).toLowerCase();
                        if (str.includes('threatlocker') || str.includes('ad-slot') || str.includes('sponsored') || str.includes('ad_wrapper') || str.includes('social') || str.includes('judgment') || str.includes('share')) {
                            el.remove();
                            return;
                        }
                        if (el.tagName === 'A') {
                            const href = (el.getAttribute('href') || '').toLowerCase();
                            if (href.includes('threatlocker') || href.includes('doubleclick') || href.includes('adnxs') || href.includes('googleadservices') || href.includes('/ads/') || href.includes('sponsor')) {
                                el.remove();
                                return;
                            }
                        }
                        if (el.tagName === 'IMG') {
                            const src = (el.getAttribute('src') || '').toLowerCase();
                            const alt = (el.getAttribute('alt') || '').toLowerCase();
                            if (src.includes('threatlocker') || alt.includes('threatlocker') || src.includes('judgment') || alt.includes('judgment') || src.includes('banner') || src.includes('ads/') || src.includes('ad_') || src.includes('promo')) {
                                el.closest('div, a, p, table, section, figure')?.remove();
                                return;
                            }
                        }
                    });

                    // 3. Extract title, metrics and metadata
                    const title = (document.querySelector('h1')?.innerText || document.title || 'Reader View').trim();
                    const hostname = window.location.hostname;
                    const url = window.location.href;
                    const words = (clone.innerText || '').trim().split(/\s+/).filter(Boolean).length;
                    const readTime = Math.max(1, Math.ceil(words / 220));

                    // 4. Create Cyberpunk Reader Container Overlay
                    overlay = document.createElement('div');
                    overlay.id = 'kestrel-reader-overlay';
                    overlay.style.cssText = 'position:fixed!important;top:0!important;left:0!important;width:100vw!important;height:100vh!important;background:#181825!important;color:#cdd6f4!important;z-index:2147483647!important;overflow-y:auto!important;padding:0!important;margin:0!important;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif!important;line-height:1.75!important;box-sizing:border-box!important;';

                    overlay.innerHTML = `
                        <style>
                            #kestrel-reader-overlay * {
                                background-color: transparent !important;
                                color: #cdd6f4 !important;
                                box-shadow: none !important;
                                border-color: #313244 !important;
                                text-shadow: none !important;
                                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif !important;
                            }
                            #kestrel-reader-overlay #kestrel-reader-bar,
                            #kestrel-reader-overlay #kestrel-reader-bar * {
                                background: transparent !important;
                            }
                            #kestrel-reader-overlay #kestrel-reader-bar {
                                background: rgba(24, 24, 37, 0.98) !important;
                            }
                            #kestrel-reader-overlay #kestrel-font-box {
                                background: #1e1e2e !important;
                            }
                            #kestrel-reader-overlay #kestrel-copy-md {
                                background: #89b4fa !important;
                                color: #11111b !important;
                            }
                            #kestrel-reader-overlay #kestrel-close-reader {
                                background: #313244 !important;
                                color: #cdd6f4 !important;
                            }
                            #kestrel-reader-overlay p {
                                margin: 0 0 20px 0 !important;
                                line-height: 1.8 !important;
                                font-size: 18px !important;
                                color: #bac2de !important;
                            }
                            #kestrel-reader-overlay h1, 
                            #kestrel-reader-overlay h2, 
                            #kestrel-reader-overlay h3, 
                            #kestrel-reader-overlay h4 {
                                color: #89b4fa !important;
                                font-weight: 700 !important;
                                margin-top: 32px !important;
                                margin-bottom: 14px !important;
                                line-height: 1.3 !important;
                            }
                            #kestrel-reader-overlay a {
                                color: #89b4fa !important;
                                text-decoration: underline !important;
                            }
                            #kestrel-reader-overlay pre, 
                            #kestrel-reader-overlay code {
                                background: #11111b !important;
                                color: #a6e3a1 !important;
                                padding: 8px 12px !important;
                                border-radius: 6px !important;
                                font-family: Consolas, monospace !important;
                            }
                            #kestrel-reader-overlay blockquote {
                                border-left: 3px solid #89b4fa !important;
                                padding-left: 16px !important;
                                color: #a6adc8 !important;
                                margin: 16px 0 !important;
                                font-style: italic !important;
                            }
                        </style>
                        <div id="kestrel-reader-bar" style="position:sticky;top:0;left:0;right:0;border-bottom:1px solid #313244;padding:12px 24px;display:flex;align-items:center;justify-content:space-between;z-index:10;box-sizing:border-box;">
                            <div style="display:flex;align-items:center;gap:12px;">
                                <span style="background:rgba(137,180,250,0.15)!important;color:#89b4fa!important;border:1px solid rgba(137,180,250,0.3)!important;font-size:10px!important;font-weight:700!important;padding:3px 8px!important;border-radius:4px!important;letter-spacing:0.5px!important;">CYBERPUNK CLEAN READER</span>
                                <span style="color:#a6adc8!important;font-size:12px!important;font-weight:500!important;">~${readTime} min read</span>
                                <span style="color:#6c7086!important;font-size:12px!important;">•</span>
                                <span style="color:#a6adc8!important;font-size:12px!important;">${words.toLocaleString()} words</span>
                                <span style="color:#6c7086!important;font-size:12px!important;">•</span>
                                <span style="color:#89b4fa!important;font-size:12px!important;font-weight:600!important;">${hostname}</span>
                            </div>
                            <div style="display:flex;align-items:center;gap:12px;">
                                <div id="kestrel-font-box" style="display:flex;align-items:center;gap:4px;border:1px solid #313244;border-radius:6px;padding:2px 6px;">
                                    <button id="kestrel-font-down" style="background:none!important;border:none!important;color:#cdd6f4!important;font-weight:700!important;cursor:pointer;padding:3px 8px;font-size:11px;">A-</button>
                                    <span id="kestrel-font-val" style="color:#a6adc8!important;font-size:11px!important;min-width:32px;text-align:center;">18px</span>
                                    <button id="kestrel-font-up" style="background:none!important;border:none!important;color:#cdd6f4!important;font-weight:700!important;cursor:pointer;padding:3px 8px;font-size:11px;">A+</button>
                                </div>
                                <button id="kestrel-copy-md" style="font-weight:700;font-size:11px;border:none;border-radius:6px;padding:6px 14px;cursor:pointer;display:flex;align-items:center;gap:6px;">📋 Copy Clean Markdown</button>
                                <button id="kestrel-close-reader" style="font-weight:700;font-size:13px;border:none;border-radius:6px;width:28px;height:28px;cursor:pointer;display:flex;align-items:center;justify-content:center;" title="Exit Reader Mode (Alt+R)">✕</button>
                            </div>
                        </div>

                        <div id="kestrel-article-container" style="max-width:780px;margin:36px auto 100px auto;padding:0 24px;box-sizing:border-box;">
                            <h1 style="font-size:32px!important;font-weight:800!important;line-height:1.25!important;color:#cdd6f4!important;margin:0 0 12px 0!important;">${title}</h1>
                            <div style="font-size:12px!important;color:#a6adc8!important;margin-bottom:28px!important;padding-bottom:14px!important;border-bottom:1px solid #313244!important;display:flex!important;gap:16px!important;">
                                <span>Source: <a href="${url}" style="color:#89b4fa!important;text-decoration:none!important;">${hostname}</a></span>
                                <span>•</span>
                                <span>De-cluttered by Kestrel Shield</span>
                            </div>
                            <div id="kestrel-article-body">
                                ${clone.innerHTML}
                            </div>
                        </div>
                    `;

                    document.body.appendChild(overlay);

                    // Font controls
                    let currentSize = 18;
                    const bodyEl = document.getElementById('kestrel-article-body');
                    const fontVal = document.getElementById('kestrel-font-val');
                    document.getElementById('kestrel-font-up').onclick = () => {
                        currentSize = Math.min(28, currentSize + 2);
                        bodyEl.querySelectorAll('p, span, div, li').forEach(el => {
                            el.style.setProperty('font-size', currentSize + 'px', 'important');
                        });
                        fontVal.innerText = currentSize + 'px';
                    };
                    document.getElementById('kestrel-font-down').onclick = () => {
                        currentSize = Math.max(12, currentSize - 2);
                        bodyEl.querySelectorAll('p, span, div, li').forEach(el => {
                            el.style.setProperty('font-size', currentSize + 'px', 'important');
                        });
                        fontVal.innerText = currentSize + 'px';
                    };

                    // Close overlay & restore body children
                    const closeReader = () => {
                        overlay.remove();
                        document.querySelectorAll('[data-kestrel-hidden]').forEach(n => {
                            n.removeAttribute('data-kestrel-hidden');
                            n.style.removeProperty('display');
                        });
                    };
                    document.getElementById('kestrel-close-reader').onclick = closeReader;

                    // Copy real Markdown
                    document.getElementById('kestrel-copy-md').onclick = function() {
                        let md = '# ' + title + '\n\nSource: ' + url + '\n\n---\n\n';
                        const paras = bodyEl.querySelectorAll('p, h1, h2, h3, h4, pre, blockquote, li');
                        paras.forEach(p => {
                            const tag = p.tagName.toLowerCase();
                            const txt = (p.innerText || '').trim();
                            if (!txt) return;
                            if (tag === 'h1') md += '\n# ' + txt + '\n\n';
                            else if (tag === 'h2') md += '\n## ' + txt + '\n\n';
                            else if (tag === 'h3') md += '\n### ' + txt + '\n\n';
                            else if (tag === 'h4') md += '\n#### ' + txt + '\n\n';
                            else if (tag === 'pre') md += '\n```\n' + txt + '\n```\n\n';
                            else if (tag === 'blockquote') md += '\n> ' + txt + '\n\n';
                            else if (tag === 'li') md += '- ' + txt + '\n';
                            else md += txt + '\n\n';
                        });

                        navigator.clipboard.writeText(md).then(() => {
                            const btn = document.getElementById('kestrel-copy-md');
                            btn.innerText = '✓ Copied Markdown!';
                            btn.style.setProperty('background', '#a6e3a1', 'important');
                            setTimeout(() => {
                                btn.innerText = '📋 Copy Clean Markdown';
                                btn.style.setProperty('background', '#89b4fa', 'important');
                            }, 2000);
                        });
                    };

                    // Clean image and link styling inside article
                    bodyEl.querySelectorAll('img').forEach(img => {
                        img.style.maxWidth = '100%';
                        img.style.height = 'auto';
                        img.style.borderRadius = '8px';
                        img.style.margin = '16px 0';
                    });
                    bodyEl.querySelectorAll('a').forEach(a => {
                        a.style.color = '#89b4fa';
                        a.style.textDecoration = 'underline';
                    });
                    bodyEl.querySelectorAll('table').forEach(t => {
                        t.style.maxWidth = '100%';
                        t.style.overflowX = 'auto';
                        t.style.display = 'block';
                    });

                } catch (err) {
                    console.error("Kestrel reader error:", err);
                }
            })();
            "#;
            let _ = wv.eval(script);
        } else {
            let script = r#"
            (function() {
                const el = document.getElementById('kestrel-reader-overlay');
                if (el) el.remove();
                document.querySelectorAll('[data-kestrel-hidden]').forEach(n => {
                    n.removeAttribute('data-kestrel-hidden');
                    n.style.removeProperty('display');
                });
            })();
            "#;
            let _ = wv.eval(script);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn switch_native_web_tab(
    app: AppHandle,
    tab_id: String,
    target_url: Option<String>,
    x: Option<f64>,
    y: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<(), String> {
    if tab_id.is_empty() {
        return Ok(());
    }

    let is_internal = if let Some(ref u) = target_url {
        u.starts_with("kestrel://") || u.trim().is_empty()
    } else {
        tab_id.starts_with("kestrel://")
    };

    // If target is internal page, hide all child webviews immediately
    if is_internal {
        for (label, wv) in app.webviews() {
            if label != "main" {
                let _ = wv.hide();
            }
        }
        return Ok(());
    }

    // Hide all webviews except the active one
    for (label, wv) in app.webviews() {
        if label != "main" && label != tab_id {
            let _ = wv.hide();
        }
    }

    // Show and position the active webview
    if let Some(wv) = app.get_webview(&tab_id) {
        if let (Some(px), Some(py)) = (x, y) {
            let _ = wv.set_position(LogicalPosition::new(px, py));
        }
        if let (Some(w), Some(h)) = (width, height) {
            let _ = wv.set_size(LogicalSize::new(w.max(100.0), h.max(100.0)));
        }
        let _ = wv.show();
    }

    Ok(())
}

#[tauri::command]
pub fn save_file_to_downloads(file_name: String, base64_content: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let downloads_dir = {
        let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:".to_string());
        std::path::PathBuf::from(userprofile).join("Downloads")
    };

    #[cfg(not(target_os = "windows"))]
    let downloads_dir = {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join("Downloads")
    };

    let _ = std::fs::create_dir_all(&downloads_dir);

    let clean_name = file_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let target_path = downloads_dir.join(&clean_name);

    let raw_b64 = if let Some(idx) = base64_content.find(";base64,") {
        &base64_content[idx + 8..]
    } else {
        &base64_content
    };

    let bytes = match decode_base64_payload(raw_b64) {
        Ok(b) => b,
        Err(_) => base64_content.into_bytes(),
    };

    std::fs::write(&target_path, &bytes)
        .map_err(|e| format!("Failed to save file to Downloads: {}", e))?;

    Ok(target_path.to_string_lossy().to_string())
}

fn decode_base64_payload(input: &str) -> Result<Vec<u8>, ()> {
    let clean = input.trim();
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;

    for byte in clean.bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\r' | b'\n' | b' ' => continue,
            _ => return Err(()),
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn open_in_file_explorer(file_path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", file_path))
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Failed to open finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent = std::path::Path::new(&file_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_system_clipboard() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        type HWND = *mut std::ffi::c_void;
        type HANDLE = *mut std::ffi::c_void;
        type BOOL = i32;

        #[link(name = "user32")]
        unsafe extern "system" {
            fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
            fn CloseClipboard() -> BOOL;
            fn GetClipboardData(uFormat: u32) -> HANDLE;
            fn IsClipboardFormatAvailable(format: u32) -> BOOL;
        }

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GlobalLock(hMem: HANDLE) -> *mut u16;
            fn GlobalUnlock(hMem: HANDLE) -> BOOL;
        }

        unsafe {
            if IsClipboardFormatAvailable(13 /* CF_UNICODETEXT */) == 0 {
                return Ok(String::new());
            }
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return Ok(String::new());
            }
            let h_data = GetClipboardData(13);
            if h_data.is_null() {
                CloseClipboard();
                return Ok(String::new());
            }
            let p_wide = GlobalLock(h_data);
            if p_wide.is_null() {
                CloseClipboard();
                return Ok(String::new());
            }

            let mut len = 0;
            while *p_wide.add(len) != 0 {
                len += 1;
                if len > 500_000 { break; }
            }
            let slice = std::slice::from_raw_parts(p_wide, len);
            let text = String::from_utf16_lossy(slice);

            GlobalUnlock(h_data);
            CloseClipboard();

            Ok(text)
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("pbpaste").output().map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("wl-paste").output() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else if let Ok(out) = std::process::Command::new("xclip").args(&["-o", "-selection", "clipboard"]).output() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            Ok(String::new())
        }
    }
}

#[tauri::command]
pub async fn capture_tab_screenshot() -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                type HDC = *mut std::ffi::c_void;
                type HBITMAP = *mut std::ffi::c_void;
                type HWND = *mut std::ffi::c_void;
                type HGDIOBJ = *mut std::ffi::c_void;
                type BOOL = i32;

                #[link(name = "user32")]
                unsafe extern "system" {
                    fn GetDC(hWnd: HWND) -> HDC;
                    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
                    fn GetSystemMetrics(nIndex: i32) -> i32;
                    fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
                    fn CloseClipboard() -> BOOL;
                    fn EmptyClipboard() -> BOOL;
                    fn SetClipboardData(uFormat: u32, hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
                }

                #[link(name = "gdi32")]
                unsafe extern "system" {
                    fn CreateCompatibleDC(hdc: HDC) -> HDC;
                    fn DeleteDC(hdc: HDC) -> BOOL;
                    fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP;
                    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
                    fn BitBlt(hdc: HDC, x: i32, y: i32, cx: i32, cy: i32, hdcSrc: HDC, x1: i32, y1: i32, rop: u32) -> BOOL;
                    fn GetDIBits(hdc: HDC, hbm: HBITMAP, start: u32, cLines: u32, lpvBits: *mut u8, lpbmi: *mut u8, usage: u32) -> i32;
                }

                let width = GetSystemMetrics(0); // SM_CXSCREEN
                let height = GetSystemMetrics(1); // SM_CYSCREEN

                if width <= 0 || height <= 0 {
                    let _ = tx.send(Err("Invalid screen metrics".to_string()));
                    return;
                }

                let hdc_screen = GetDC(std::ptr::null_mut());
                if hdc_screen.is_null() {
                    let _ = tx.send(Err("GetDC failed".to_string()));
                    return;
                }

                let hdc_mem = CreateCompatibleDC(hdc_screen);
                if hdc_mem.is_null() {
                    ReleaseDC(std::ptr::null_mut(), hdc_screen);
                    let _ = tx.send(Err("CreateCompatibleDC failed".to_string()));
                    return;
                }

                let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
                if hbm.is_null() {
                    DeleteDC(hdc_mem);
                    ReleaseDC(std::ptr::null_mut(), hdc_screen);
                    let _ = tx.send(Err("CreateCompatibleBitmap failed".to_string()));
                    return;
                }

                let old_obj = SelectObject(hdc_mem, hbm);
                // SRCCOPY = 0x00CC0020
                BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, 0x00CC0020);
                SelectObject(hdc_mem, old_obj);

                // Prepare DIB for BMP generation
                let mut bmi = vec![0u8; 40];
                bmi[0..4].copy_from_slice(&40u32.to_le_bytes());
                bmi[4..8].copy_from_slice(&(width as i32).to_le_bytes());
                bmi[8..12].copy_from_slice(&(-(height as i32)).to_le_bytes());
                bmi[12..14].copy_from_slice(&1u16.to_le_bytes());
                bmi[14..16].copy_from_slice(&32u16.to_le_bytes());
                bmi[16..20].copy_from_slice(&0u32.to_le_bytes());

                let image_size = (width * height * 4) as usize;
                let mut pixels = vec![0u8; image_size];

                GetDIBits(
                    hdc_mem,
                    hbm,
                    0,
                    height as u32,
                    pixels.as_mut_ptr(),
                    bmi.as_mut_ptr(),
                    0,
                );

                // Copy bitmap directly to Windows OS Clipboard (CF_BITMAP)
                if OpenClipboard(std::ptr::null_mut()) != 0 {
                    EmptyClipboard();
                    SetClipboardData(2 /* CF_BITMAP */, hbm);
                    CloseClipboard();
                }

                DeleteDC(hdc_mem);
                ReleaseDC(std::ptr::null_mut(), hdc_screen);

                // Save to file
                let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:".to_string());
                let dir = std::path::PathBuf::from(home).join("Pictures").join("Kestrel_Screenshots");
                let _ = std::fs::create_dir_all(&dir);

                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                let file_path = dir.join(format!("kestrel_ss_{}.bmp", now));

                let total_size = (14 + 40 + image_size) as u32;
                let mut bmp_data = Vec::with_capacity(14 + 40 + image_size);
                bmp_data.extend_from_slice(b"BM");
                bmp_data.extend_from_slice(&total_size.to_le_bytes());
                bmp_data.extend_from_slice(&[0u8; 4]);
                bmp_data.extend_from_slice(&54u32.to_le_bytes());
                bmp_data.extend_from_slice(&bmi);
                bmp_data.extend_from_slice(&pixels);

                let res = match std::fs::write(&file_path, bmp_data) {
                    Ok(_) => Ok(file_path.to_string_lossy().to_string()),
                    Err(e) => Err(format!("Save screenshot failed: {}", e)),
                };
                let _ = tx.send(res);
            }
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let dir = format!("{}/Pictures/Kestrel_Screenshots", home);
            let _ = std::fs::create_dir_all(&dir);
            let path = format!("{}/kestrel_ss.png", dir);

            let res = match std::process::Command::new("screencapture")
                .args(&["-c", "-x", &path])
                .output()
            {
                Ok(_) => Ok(path),
                Err(e) => Err(format!("macOS screencapture failed: {}", e)),
            };
            let _ = tx.send(res);
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let dir = format!("{}/Pictures/Kestrel_Screenshots", home);
            let _ = std::fs::create_dir_all(&dir);
            let path = format!("{}/kestrel_ss.png", dir);

            let res = if let Ok(_) = std::process::Command::new("gnome-screenshot").args(&["-c", "-f", &path]).output() {
                Ok(path)
            } else if let Ok(_) = std::process::Command::new("spectacle").args(&["-b", "-c", "-o", &path]).output() {
                Ok(path)
            } else if let Ok(_) = std::process::Command::new("grim").arg(&path).output() {
                Ok(path)
            } else {
                Err("No Linux screenshot utility found".to_string())
            };
            let _ = tx.send(res);
        }
    });

    rx.recv().map_err(|e| format!("Channel receive error: {}", e))?
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
