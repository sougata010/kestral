pub mod adblock;
pub mod cookie_guard;
pub mod striping;
pub mod url_shield;
pub mod vault;

pub use adblock::{AdBlockEngine, AdBlockStats};
pub use cookie_guard::{CookieAuditResult, CookieGuard};
pub use striping::{ParameterStriper, StripResult};
pub use url_shield::{UrlScanResult, UrlShield};
pub use vault::{CredentialSummary, VaultManager};

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

#[tauri::command]
pub fn list_vault_credentials(state: tauri::State<'_, VaultManager>) -> Vec<CredentialSummary> {
    state.list_credentials()
}

#[tauri::command]
pub fn add_vault_credential(
    domain: String,
    username: String,
    secret: String,
    state: tauri::State<'_, VaultManager>,
) -> CredentialSummary {
    state.add_credential(domain, username, secret)
}

#[tauri::command]
pub fn decrypt_vault_credential(
    id: String,
    user_pin: String,
    state: tauri::State<'_, VaultManager>,
) -> Result<String, String> {
    state.decrypt_with_approval(&id, &user_pin)
}
