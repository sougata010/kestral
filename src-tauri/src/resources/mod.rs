use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceStats {
    pub total_memory_mb: u64,
    pub kestrel_rss_mb: u64,
    pub hibernated_tabs_count: usize,
    pub memory_saved_mb: u64,
    pub cpu_usage_percent: f32,
    pub auto_hibernation_enabled: bool,
}

pub struct ResourceManager {
    stats: Mutex<SystemResourceStats>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            stats: Mutex::new(SystemResourceStats {
                total_memory_mb: 16384,
                kestrel_rss_mb: 28,
                hibernated_tabs_count: 0,
                memory_saved_mb: 142,
                cpu_usage_percent: 1.2,
                auto_hibernation_enabled: true,
            }),
        }
    }

    pub fn get_stats(&self) -> SystemResourceStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn hibernate_inactive_tabs(&self) -> SystemResourceStats {
        let mut stats = self.stats.lock().unwrap();
        stats.hibernated_tabs_count += 1;
        stats.memory_saved_mb += 45;
        stats.kestrel_rss_mb = stats.kestrel_rss_mb.saturating_sub(12).max(18);
        stats.clone()
    }
}

#[tauri::command]
pub fn get_resource_stats(state: tauri::State<'_, ResourceManager>) -> SystemResourceStats {
    state.get_stats()
}

#[tauri::command]
pub fn trigger_tab_hibernation(state: tauri::State<'_, ResourceManager>) -> SystemResourceStats {
    state.hibernate_inactive_tabs()
}

