use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, WebviewBuilder, WebviewUrl, LogicalPosition, LogicalSize};

#[tauri::command]
pub fn open_native_web_tab(app: AppHandle, tab_id: String, url: String) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    let parsed_url: tauri::Url = url.parse().map_err(|e| format!("{}", e))?;
    
    // Hide other child webviews
    for (label, wv) in app.webviews() {
        if label != "main" && label != tab_id {
            let _ = wv.hide();
        }
    }

    // Check if webview already exists, show and navigate
    if let Some(existing) = app.get_webview(&tab_id) {
        let _ = existing.show();
        existing.navigate(parsed_url).map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Anchor below the 78px browser toolbar
    let webview = WebviewBuilder::new(&tab_id, WebviewUrl::External(parsed_url))
        .auto_resize();
        
    window.add_child(
        webview,
        LogicalPosition::new(0., 78.),
        LogicalSize::new(1280., 722.),
    ).map_err(|e| e.to_string())?;

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
