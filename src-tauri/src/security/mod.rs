pub mod adblock;
pub mod cookie_guard;
pub mod striping;
pub mod url_shield;
pub mod vault;

pub use adblock::{AdBlockEngine, AdBlockStats};
pub use cookie_guard::{CookieAuditResult, CookieGuard};
pub use striping::{ParameterStriper, StripResult};
pub use url_shield::{UrlScanResult, UrlShield};
pub use vault::{
    add_vault_credential, decrypt_vault_credential, list_vault_credentials, CredentialSummary,
    VaultManager,
};

#[tauri::command]
pub fn check_cookie(name: String, value: String, domain: String) -> CookieAuditResult {
    CookieGuard::audit_cookie(&name, &value, &domain)
}

#[tauri::command]
pub fn check_url(url: String) -> UrlScanResult {
    UrlShield::scan_url(&url)
}

#[tauri::command]
pub fn check_adblock(url: String, state: tauri::State<'_, AdBlockEngine>) -> (bool, bool) {
    state.check_url(&url)
}

#[tauri::command]
pub fn get_adblock_stats(state: tauri::State<'_, AdBlockEngine>) -> AdBlockStats {
    state.get_stats()
}

#[tauri::command]
pub fn get_cosmetic_filters(state: tauri::State<'_, AdBlockEngine>) -> String {
    state.get_cosmetic_css()
}

#[tauri::command]
pub fn strip_tracking_parameters(url: String) -> StripResult {
    ParameterStriper::sanitize_url(&url)
}
