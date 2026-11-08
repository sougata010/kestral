pub mod cyber_intel;
pub mod devtools;
pub mod extensions;
pub mod media;
pub mod resources;
pub mod security;
pub mod tabs;
pub mod vpn;

use cyber_intel::{audit_domain_security, get_cve_threats, get_cybersecurity_tools};
use devtools::{detect_technologies, generate_curl_command, get_network_timings, get_network_topology};
use extensions::{list_extensions, toggle_extension, ExtensionManager};
use media::{download_audio_stream, get_player_state, switch_audio_track, toggle_playback, MediaManager};
use resources::{get_resource_stats, trigger_tab_hibernation, ResourceManager};
use security::{
    add_vault_credential, check_adblock, check_cookie, check_url, decrypt_vault_credential,
    get_adblock_stats, get_cosmetic_filters, list_vault_credentials, record_blocked_ad,
    sign_and_encrypt_secret, strip_tracking_parameters, AdBlockEngine, VaultManager,
};
use tabs::{
    capture_tab_screenshot, close_native_web_tab, close_tab, create_tab, get_active_tab_id,
    get_system_clipboard, get_tabs, hide_native_web_tabs, navigate_native_web_tab_back,
    navigate_native_web_tab_forward, navigate_tab, open_in_file_explorer, open_native_web_tab,
    reload_native_web_tab, save_file_to_downloads, switch_native_web_tab, switch_tab,
    toggle_native_reader_mode, update_native_web_tab_bounds, update_tab_title, TabState,
};
use vpn::{
    get_fingerprint_protection_script, get_vpn_status, renew_tor_circuit, switch_vpn_relay,
    toggle_vpn_state, VpnManager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(TabState::new())
        .manage(AdBlockEngine::new())
        .manage(VaultManager::new())
        .manage(VpnManager::new())
        .manage(MediaManager::new())
        .manage(ResourceManager::new())
        .manage(ExtensionManager::new())
        .invoke_handler(tauri::generate_handler![
            // Tabs
            get_tabs,
            get_active_tab_id,
            create_tab,
            close_tab,
            switch_tab,
            switch_native_web_tab,
            get_system_clipboard,
            navigate_tab,
            open_native_web_tab,
            hide_native_web_tabs,
            close_native_web_tab,
            update_native_web_tab_bounds,
            update_tab_title,
            reload_native_web_tab,
            navigate_native_web_tab_back,
            navigate_native_web_tab_forward,
            toggle_native_reader_mode,
            capture_tab_screenshot,
            save_file_to_downloads,
            open_in_file_explorer,
            // Security
            check_cookie,
            check_url,
            check_adblock,
            get_adblock_stats,
            record_blocked_ad,
            get_cosmetic_filters,
            strip_tracking_parameters,
            list_vault_credentials,
            add_vault_credential,
            decrypt_vault_credential,
            sign_and_encrypt_secret,
            // DevTools
            detect_technologies,
            generate_curl_command,
            get_network_timings,
            get_network_topology,
            // Cyber Intel
            get_cybersecurity_tools,
            get_cve_threats,
            audit_domain_security,
            // VPN & Tor Circuit
            get_vpn_status,
            toggle_vpn_state,
            switch_vpn_relay,
            renew_tor_circuit,
            get_fingerprint_protection_script,
            // Media
            get_player_state,
            toggle_playback,
            switch_audio_track,
            download_audio_stream,
            // Resource Management
            get_resource_stats,
            trigger_tab_hibernation,
            // Extensions
            list_extensions,
            toggle_extension,
        ])
        .run(tauri::generate_context!())
        .expect("error while running kestrel browser");
}
