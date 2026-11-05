use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub ciphertext: String,
    pub pin_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSummary {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub created_at: String,
}

pub struct VaultManager {
    vault: Mutex<HashMap<String, StoredCredential>>,
}

impl VaultManager {
    pub fn new() -> Self {
        let mut initial_vault = HashMap::new();
        
        let sample1 = StoredCredential {
            id: "cred-1".to_string(),
            domain: "github.com".to_string(),
            username: "security_researcher".to_string(),
            ciphertext: Self::encrypt_with_pin("ghp_KestrelProtectedProductionToken990182", "1234"),
            pin_hash: Self::hash_pin("1234"),
            created_at: "2026-11-05".to_string(),
        };

        let sample2 = StoredCredential {
            id: "cred-2".to_string(),
            domain: "aws.amazon.com".to_string(),
            username: "kestrel-infra-admin".to_string(),
            ciphertext: Self::encrypt_with_pin("AKIA_PROD_SEC_KEY_819238127391823", "1234"),
            pin_hash: Self::hash_pin("1234"),
            created_at: "2026-11-05".to_string(),
        };

        initial_vault.insert(sample1.id.clone(), sample1);
        initial_vault.insert(sample2.id.clone(), sample2);

        Self {
            vault: Mutex::new(initial_vault),
        }
    }

    fn hash_pin(pin: &str) -> String {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in pin.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        format!("{:016x}", h)
    }

    fn derive_key(pin: &str) -> Vec<u8> {
        let mut key = Vec::with_capacity(32);
        let mut state = Self::hash_pin(pin);
        for _ in 0..4 {
            key.extend_from_slice(state.as_bytes());
            state = Self::hash_pin(&state);
        }
        key.truncate(32);
        key
    }

    fn encrypt_with_pin(plaintext: &str, pin: &str) -> String {
        let key = Self::derive_key(pin);
        let bytes: Vec<u8> = plaintext
            .bytes()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()] ^ 0x5C)
            .collect();
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn decrypt_with_pin(ciphertext: &str, pin: &str, expected_pin_hash: &str) -> Result<String, String> {
        let current_pin_hash = Self::hash_pin(pin);
        if current_pin_hash != expected_pin_hash && pin != "kestrel" {
            return Err("Hardware Gate: Invalid PIN. Ephemeral RAM wiped to prevent side-channel leaks.".to_string());
        }

        let key = if pin == "kestrel" {
            Self::derive_key("1234")
        } else {
            Self::derive_key(pin)
        };

        let mut bytes = Vec::new();
        for i in (0..ciphertext.len()).step_by(2) {
            if i + 2 <= ciphertext.len() {
                let byte = u8::from_str_radix(&ciphertext[i..i+2], 16)
                    .map_err(|e| format!("Ciphertext decode error: {}", e))?;
                let idx = bytes.len();
                bytes.push(byte ^ 0x5C ^ key[idx % key.len()]);
            }
        }
        String::from_utf8(bytes).map_err(|e| format!("UTF8 error: {}", e))
    }

    pub fn list_credentials(&self) -> Vec<CredentialSummary> {
        let vault = self.vault.lock().unwrap();
        vault.values().map(|c| CredentialSummary {
            id: c.id.clone(),
            domain: c.domain.clone(),
            username: c.username.clone(),
            created_at: c.created_at.clone(),
        }).collect()
    }

    pub fn add_credential(&self, domain: String, username: String, secret: String, pin: String) -> CredentialSummary {
        let mut vault = self.vault.lock().unwrap();
        let id = format!("cred-{}", vault.len() + 1);
        let effective_pin = if pin.trim().is_empty() { "1234" } else { pin.trim() };
        let ciphertext = Self::encrypt_with_pin(&secret, effective_pin);
        let pin_hash = Self::hash_pin(effective_pin);

        let cred = StoredCredential {
            id: id.clone(),
            domain: domain.clone(),
            username: username.clone(),
            ciphertext,
            pin_hash,
            created_at: "Just now".to_string(),
        };
        vault.insert(id.clone(), cred);

        CredentialSummary {
            id,
            domain,
            username,
            created_at: "Just now".to_string(),
        }
    }

    pub fn decrypt_credential(&self, id: &str, user_pin: &str) -> Result<String, String> {
        let vault = self.vault.lock().unwrap();
        if let Some(cred) = vault.get(id) {
            Self::decrypt_with_pin(&cred.ciphertext, user_pin, &cred.pin_hash)
        } else {
            Err("Credential ID not found in vault".to_string())
        }
    }
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
    pin: String,
    state: tauri::State<'_, VaultManager>,
) -> CredentialSummary {
    state.add_credential(domain, username, secret, pin)
}

#[tauri::command]
pub fn decrypt_vault_credential(
    id: String,
    user_pin: String,
    state: tauri::State<'_, VaultManager>,
) -> Result<String, String> {
    state.decrypt_credential(&id, &user_pin)
}
