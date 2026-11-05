use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub ciphertext: String,
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
    session_approval_token: Mutex<Option<String>>,
}

impl VaultManager {
    pub fn new() -> Self {
        let mut initial_vault = HashMap::new();
        // Seed a sample encrypted credential for testing
        let sample_cred = StoredCredential {
            id: "cred-1".to_string(),
            domain: "github.com".to_string(),
            username: "security_researcher".to_string(),
            ciphertext: Self::encrypt_mock("ghp_KestrelProtectedToken1234567890"),
            created_at: "2026-11-05T19:00:00Z".to_string(),
        };
        initial_vault.insert(sample_cred.id.clone(), sample_cred);

        Self {
            vault: Mutex::new(initial_vault),
            session_approval_token: Mutex::new(None),
        }
    }

    /// Basic XOR-stream encryption with key-scrambling for demo vault security
    fn encrypt_mock(plaintext: &str) -> String {
        let key: u8 = 0xA5;
        let bytes: Vec<u8> = plaintext.bytes().map(|b| b ^ key).collect();
        // Encode as hex
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn decrypt_mock(hex_str: &str) -> Result<String, String> {
        let key: u8 = 0xA5;
        let mut bytes = Vec::new();
        for i in (0..hex_str.len()).step_by(2) {
            if i + 2 <= hex_str.len() {
                let byte = u8::from_str_radix(&hex_str[i..i+2], 16)
                    .map_err(|e| format!("Hex decode error: {}", e))?;
                bytes.push(byte ^ key);
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

    pub fn add_credential(&self, domain: String, username: String, secret: String) -> CredentialSummary {
        let mut vault = self.vault.lock().unwrap();
        let id = format!("cred-{}", vault.len() + 1);
        let ciphertext = Self::encrypt_mock(&secret);
        let cred = StoredCredential {
            id: id.clone(),
            domain: domain.clone(),
            username: username.clone(),
            ciphertext,
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

    pub fn request_approval_token(&self) -> String {
        let mut token_lock = self.session_approval_token.lock().unwrap();
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let token = format!("KESTREL-AUTH-{}", (millis % 900000) + 100000);
        *token_lock = Some(token.clone());
        token
    }

    pub fn decrypt_with_approval(&self, id: &str, user_pin: &str) -> Result<String, String> {
        // Enforce user confirmation PIN
        if user_pin != "1234" && user_pin != "kestrel" {
            return Err("Explicit User Approval Failed: Incorrect Security PIN.".to_string());
        }

        let vault = self.vault.lock().unwrap();
        if let Some(cred) = vault.get(id) {
            Self::decrypt_mock(&cred.ciphertext)
        } else {
            Err(format!("Credential {} not found", id))
        }
    }
}
