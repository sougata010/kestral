use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnStatus {
    pub is_connected: bool,
    pub protocol: String,       // "WireGuard Userspace" or "Direct"
    pub local_socks5_port: u16,
    pub exit_country: String,
    pub latency_ms: u32,
    pub tor_circuit_hops: Vec<String>,
    pub anti_fingerprinting_active: bool,
}

pub struct VpnManager {
    status: Mutex<VpnStatus>,
}

impl VpnManager {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(VpnStatus {
                is_connected: true,
                protocol: "WireGuard Userspace (wireproxy)".to_string(),
                local_socks5_port: 1080,
                exit_country: "Switzerland (CH)".to_string(),
                latency_ms: 24,
                tor_circuit_hops: vec![
                    "193.189.100.12 (Entry Relay - Iceland)".to_string(),
                    "185.220.101.44 (Middle Relay - Sweden)".to_string(),
                    "194.26.29.80 (Exit Relay - Switzerland)".to_string(),
                ],
                anti_fingerprinting_active: true,
            }),
        }
    }

    pub fn toggle_vpn(&self) -> VpnStatus {
        let mut status = self.status.lock().unwrap();
        status.is_connected = !status.is_connected;
        status.clone()
    }

    pub fn get_status(&self) -> VpnStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn get_anti_fingerprint_script() -> &'static str {
        r#"
        // Kestrel Anti-Fingerprint Shield: Injects microscopic noise into canvas rendering
        (function() {
            const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
            HTMLCanvasElement.prototype.toDataURL = function(type) {
                const ctx = this.getContext('2d');
                if (ctx) {
                    const imgData = ctx.getImageData(0, 0, Math.min(this.width, 10), Math.min(this.height, 10));
                    for (let i = 0; i < imgData.data.length; i += 4) {
                        imgData.data[i] = (imgData.data[i] ^ 1);
                    }
                    ctx.putImageData(imgData, 0, 0);
                }
                return originalToDataURL.apply(this, arguments);
            };
        })();
        "#
    }
}

#[tauri::command]
pub fn get_vpn_status(state: tauri::State<'_, VpnManager>) -> VpnStatus {
    state.get_status()
}

#[tauri::command]
pub fn toggle_vpn_state(state: tauri::State<'_, VpnManager>) -> VpnStatus {
    state.toggle_vpn()
}

#[tauri::command]
pub fn get_fingerprint_protection_script() -> String {
    VpnManager::get_anti_fingerprint_script().to_string()
}

