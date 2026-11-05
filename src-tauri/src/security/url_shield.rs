use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlScanResult {
    pub url: String,
    pub is_safe: bool,
    pub threat_score: u8, // 0 to 100 (100 = critical threat)
    pub threat_category: String,
    pub reasons: Vec<String>,
}

pub struct UrlShield;

impl UrlShield {
    pub fn scan_url(url_str: &str) -> UrlScanResult {
        let mut threat_score: u8 = 0;
        let mut reasons = Vec::new();
        let lower_url = url_str.to_lowercase();

        // 1. IP address in hostname (e.g. http://192.168.1.1/verify)
        if Self::has_ip_hostname(&lower_url) {
            threat_score += 45;
            reasons.push("IP address used directly in hostname instead of verified domain".to_string());
        }

        // 2. Punycode / IDN Homograph attack check
        if lower_url.contains("xn--") {
            threat_score += 50;
            reasons.push("Punycode (IDN homograph) detected, potential domain impersonation".to_string());
        }

        // 3. Known Phishing keywords combined with suspicious patterns
        let phishing_keywords = ["paypal-security", "verify-account", "apple-support", "bank-login", "wallet-connect"];
        for keyword in phishing_keywords {
            if lower_url.contains(keyword) {
                threat_score += 40;
                reasons.push(format!("Detected high-risk phishing keyword combination: {}", keyword));
            }
        }

        // 4. Excessive subdomain depth (e.g. login.secure.bank.verify.example.com)
        if lower_url.matches('.').count() > 4 {
            threat_score += 25;
            reasons.push("Excessive subdomain nesting (> 4 dots), common in obfuscated phishing tunnels".to_string());
        }

        // 5. High-abuse TLD reputation check
        let suspicious_tlds = [".tk", ".ml", ".ga", ".cf", ".gq", ".buzz", ".top", ".rest"];
        for tld in suspicious_tlds {
            if lower_url.contains(tld) {
                threat_score += 20;
                reasons.push(format!("Domain uses high-abuse top-level domain: {}", tld));
            }
        }

        // 6. Suspicious credential embedding in URL (http://user:pass@evil.com)
        if lower_url.contains('@') {
            threat_score += 35;
            reasons.push("URL contains embedded '@' character, indicating potential credential leak or URL spoofing".to_string());
        }

        let is_safe = threat_score < 50;
        let threat_category = if threat_score >= 80 {
            "MALICIOUS_PHISHING".to_string()
        } else if threat_score >= 50 {
            "SUSPICIOUS_UNVERIFIED".to_string()
        } else {
            "SAFE".to_string()
        };

        UrlScanResult {
            url: url_str.to_string(),
            is_safe,
            threat_score: threat_score.min(100),
            threat_category,
            reasons,
        }
    }

    fn has_ip_hostname(url: &str) -> bool {
        // Strip scheme
        let without_scheme = url
            .strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .unwrap_or(url);

        let host = without_scheme.split('/').next().unwrap_or("");
        let host_clean = host.split(':').next().unwrap_or("");

        let parts: Vec<&str> = host_clean.split('.').collect();
        if parts.len() == 4 {
            parts.iter().all(|p| p.parse::<u8>().is_ok())
        } else {
            false
        }
    }
}

