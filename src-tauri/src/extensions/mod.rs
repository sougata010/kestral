use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub is_enabled: bool,
    pub permissions: Vec<String>,
}

pub struct ExtensionManager {
    extensions: Mutex<Vec<InstalledExtension>>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        let default_extensions = vec![
            InstalledExtension {
                id: "ext-react-devtools".to_string(),
                name: "React Developer Tools".to_string(),
                version: "5.2.0".to_string(),
                description: "Inspect React component hierarchies, props, and state.".to_string(),
                is_enabled: true,
                permissions: vec!["contextMenus".to_string(), "activeTab".to_string()],
            },
            InstalledExtension {
                id: "ext-dark-reader".to_string(),
                name: "Dark Reader".to_string(),
                version: "4.9.80".to_string(),
                description: "Inverts colors of visited web pages with eye-friendly dark mode.".to_string(),
                is_enabled: true,
                permissions: vec!["storage".to_string(), "tabs".to_string()],
            },
            InstalledExtension {
                id: "ext-json-formatter".to_string(),
                name: "JSON Formatter".to_string(),
                version: "0.7.3".to_string(),
                description: "Formats and color-codes raw JSON responses.".to_string(),
                is_enabled: true,
                permissions: vec!["<all_urls>".to_string()],
            },
        ];

        Self {
            extensions: Mutex::new(default_extensions),
        }
    }

    pub fn list(&self) -> Vec<InstalledExtension> {
        self.extensions.lock().unwrap().clone()
    }

    pub fn toggle(&self, id: &str) -> Result<InstalledExtension, String> {
        let mut exts = self.extensions.lock().unwrap();
        if let Some(ext) = exts.iter_mut().find(|e| e.id == id) {
            ext.is_enabled = !ext.is_enabled;
            Ok(ext.clone())
        } else {
            Err("Extension not found".to_string())
        }
    }
}

#[tauri::command]
pub fn list_extensions(state: tauri::State<'_, ExtensionManager>) -> Vec<InstalledExtension> {
    state.list()
}

#[tauri::command]
pub fn toggle_extension(id: String, state: tauri::State<'_, ExtensionManager>) -> Result<InstalledExtension, String> {
    state.toggle(&id)
}

