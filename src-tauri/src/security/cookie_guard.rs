use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieAuditResult {
    pub name: String,
    pub domain: String,
    pub entropy: f64,
    pub is_suspicious: bool,
    pub is_malicious: bool,
    pub threat_tags: Vec<String>,
    pub recommended_action: String,
}

pub struct CookieGuard;

impl CookieGuard {
    /// Calculate Shannon entropy for cookie value
    pub fn calculate_entropy(value: &str) -> f64 {
        if value.is_empty() {
            return 0.0;
        }

        let mut frequency: HashMap<char, usize> = HashMap::new();
        for ch in value.chars() {
            *frequency.entry(ch).or_insert(0) += 1;
        }

        let len = value.chars().count() as f64;
        let mut entropy = 0.0;

        for &count in frequency.values() {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }

        entropy
    }

    /// Audit cookie for malicious signatures and anomalous entropy
    pub fn audit_cookie(name: &str, value: &str, domain: &str) -> CookieAuditResult {
        let entropy = Self::calculate_entropy(value);
        let mut threat_tags = Vec::new();
        let mut is_malicious = false;
        let mut is_suspicious = false;

        let lower_val = value.to_lowercase();

        // 1. Check for Cross-Site Scripting (XSS) signatures
        if lower_val.contains("<script") 
            || lower_val.contains("javascript:") 
            || lower_val.contains("onerror=") 
            || lower_val.contains("onload=")
        {
            threat_tags.push("XSS_PAYLOAD_DETECTED".to_string());
            is_malicious = true;
        }

        // 2. Check for SQL Injection (SQLi) signatures
        if lower_val.contains("' or '1'='1") 
            || lower_val.contains("union select") 
            || lower_val.contains("--")
            || lower_val.contains("; drop table")
        {
            threat_tags.push("SQLI_PAYLOAD_DETECTED".to_string());
            is_malicious = true;
        }

        // 3. Check for Deserialization / Object Injections
        if value.starts_with("rO0AB") // Java serialized object magic bytes
            || lower_val.starts_with("o:8:\"") // PHP serialization
        {
            threat_tags.push("DESERIALIZATION_EXPLOIT".to_string());
            is_malicious = true;
        }

        // 4. Check for high Shannon entropy tracking beacons
        if entropy > 4.8 && value.len() > 40 {
            threat_tags.push("HIGH_ENTROPY_TRACKER".to_string());
            is_suspicious = true;
        }

        // 5. Check for known tracking cookie prefixes
        let lower_name = name.to_lowercase();
        if lower_name.starts_with("_ga") 
            || lower_name.starts_with("_gid") 
            || lower_name.starts_with("_fbp") 
            || lower_name.starts_with("__cf_bm")
            || lower_name.contains("tracker")
            || lower_name.contains("visitor_id")
        {
            threat_tags.push("CROSS_SITE_TRACKER".to_string());
            is_suspicious = true;
        }

        let recommended_action = if is_malicious {
            "BLOCK_IMMEDIATELY".to_string()
        } else if is_suspicious {
            "STRIP_OR_QUARANTINE_TO_SESSION".to_string()
        } else {
            "ALLOW".to_string()
        };

        CookieAuditResult {
            name: name.to_string(),
            domain: domain.to_string(),
            entropy,
            is_suspicious,
            is_malicious,
            threat_tags,
            recommended_action,
        }
    }
}

