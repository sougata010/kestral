use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechCategory {
    pub name: String,
    pub version: Option<String>,
    pub category: String, // "Frontend", "Styling", "Hosting", "CDN"
    pub icon_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTiming {
    pub url: String,
    pub dns_lookup_ms: f64,
    pub tcp_connect_ms: f64,
    pub tls_handshake_ms: f64,
    pub ttfb_ms: f64,
    pub download_ms: f64,
    pub total_ms: f64,
    pub status: u16,
    pub size_bytes: usize,
}

pub struct TechDetector;

impl TechDetector {
    pub fn analyze_url_and_body(url: &str, html_snippet: &str) -> Vec<TechCategory> {
        let mut detected = Vec::new();
        let lower_html = html_snippet.to_lowercase();
        let lower_url = url.to_lowercase();

        // 1. Next.js
        if lower_html.contains("__next") || lower_html.contains("_next/static") {
            detected.push(TechCategory {
                name: "Next.js".to_string(),
                version: Some("14.x".to_string()),
                category: "Frontend Framework".to_string(),
                icon_color: "#ffffff".to_string(),
            });
        }

        // 2. React
        if lower_html.contains("react") || lower_html.contains("data-reactroot") || lower_html.contains("_next") {
            detected.push(TechCategory {
                name: "React".to_string(),
                version: Some("18.3".to_string()),
                category: "UI Library".to_string(),
                icon_color: "#61dafb".to_string(),
            });
        }

        // 3. Vue / Nuxt
        if lower_html.contains("v-bind") || lower_html.contains("v-model") || lower_html.contains("__nuxt") {
            detected.push(TechCategory {
                name: "Vue.js".to_string(),
                version: Some("3.x".to_string()),
                category: "Frontend Framework".to_string(),
                icon_color: "#42b883".to_string(),
            });
        }

        // 4. Tailwind CSS
        if lower_html.contains("flex") && lower_html.contains("grid") && lower_html.contains("space-y-") {
            detected.push(TechCategory {
                name: "Tailwind CSS".to_string(),
                version: Some("3.4".to_string()),
                category: "CSS Framework".to_string(),
                icon_color: "#38bdf8".to_string(),
            });
        }

        // 5. Cloudflare CDN
        if lower_url.contains("cloudflare") || lower_html.contains("cf-cache-status") {
            detected.push(TechCategory {
                name: "Cloudflare".to_string(),
                version: None,
                category: "CDN & Edge".to_string(),
                icon_color: "#f38020".to_string(),
            });
        }

        // 6. Vite
        if lower_html.contains("/@vite/client") || lower_html.contains("vite") {
            detected.push(TechCategory {
                name: "Vite".to_string(),
                version: Some("6.0".to_string()),
                category: "Bundler".to_string(),
                icon_color: "#bd34fe".to_string(),
            });
        }

        // Default fallback if nothing matches
        if detected.is_empty() {
            detected.push(TechCategory {
                name: "HTML5 / Standard Web".to_string(),
                version: None,
                category: "Web Platform".to_string(),
                icon_color: "#e34f26".to_string(),
            });
        }

        detected
    }

    pub fn generate_curl(url: &str) -> String {
        format!("curl -X GET \"{}\" \\\n  -H \"User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Kestrel/0.1.0\" \\\n  -H \"Accept: text/html,application/xhtml+xml\" \\\n  --compressed", url)
    }
}

