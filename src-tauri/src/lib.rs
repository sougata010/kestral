pub mod cyber_intel;
pub mod devtools;
pub mod extensions;
pub mod media;
pub mod resources;
pub mod security;
pub mod tabs;
pub mod vpn;

use cyber_intel::{audit_domain_security, get_cve_threats, get_cybersecurity_tools};
use devtools::{detect_technologies, generate_curl_command, get_network_timings};
use extensions::{list_extensions, toggle_extension, ExtensionManager};
use media::{download_audio_stream, get_player_state, toggle_playback, MediaManager};
use resources::{get_resource_stats, trigger_tab_hibernation, ResourceManager};
use security::{
    add_vault_credential, check_adblock, check_cookie, check_url, decrypt_vault_credential,
    get_adblock_stats, get_cosmetic_filters, list_vault_credentials, strip_tracking_parameters,
    AdBlockEngine, VaultManager,
};
use tabs::{close_tab, create_tab, get_active_tab_id, get_tabs, navigate_tab, switch_tab, TabState};
use vpn::{get_fingerprint_protection_script, get_vpn_status, toggle_vpn_state, VpnManager};

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
            navigate_tab,
            // Security
            check_cookie,
            check_url,
            check_adblock,
            get_adblock_stats,
            get_cosmetic_filters,
            strip_tracking_parameters,
            list_vault_credentials,
            add_vault_credential,
            decrypt_vault_credential,
            // DevTools
            detect_technologies,
            generate_curl_command,
            get_network_timings,
            // Cyber Intel
            get_cybersecurity_tools,
            get_cve_threats,
            audit_domain_security,
            // VPN & Tor Circuit
            get_vpn_status,
            toggle_vpn_state,
            get_fingerprint_protection_script,
            // Media
            get_player_state,
            toggle_playback,
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
