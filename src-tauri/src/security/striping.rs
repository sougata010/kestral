use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripResult {
    pub original_url: String,
    pub sanitized_url: String,
    pub stripped_params: Vec<String>,
    pub was_modified: bool,
}

pub struct ParameterStriper;

impl ParameterStriper {
    const TRACKING_PARAMS: &'static [&'static str] = &[
        "utm_source",
        "utm_medium",
        "utm_campaign",
        "utm_term",
        "utm_content",
        "utm_id",
        "fbclid",
        "gclid",
        "dclid",
        "msclkid",
        "twclid",
        "gbraid",
        "wbraid",
        "mc_cid",
        "mc_eid",
        "_hsenc",
        "_hsmi",
        "igshid",
        "yclid",
        "spm",
        "ref_",
        "vero_id",
        "wickedid",
    ];

    pub fn sanitize_url(raw_url: &str) -> StripResult {
        if !raw_url.contains('?') {
            return StripResult {
                original_url: raw_url.to_string(),
                sanitized_url: raw_url.to_string(),
                stripped_params: Vec::new(),
                was_modified: false,
            };
        }

        let parts: Vec<&str> = raw_url.splitn(2, '?').collect();
        let base_url = parts[0];
        let query_string = parts[1];

        let mut clean_query_pairs = Vec::new();
        let mut stripped_params = Vec::new();

        for pair in query_string.split('&') {
            if pair.is_empty() {
                continue;
            }
            let key = pair.split('=').next().unwrap_or(pair).to_lowercase();
            if Self::TRACKING_PARAMS.contains(&key.as_str()) {
                stripped_params.push(key);
            } else {
                clean_query_pairs.push(pair);
            }
        }

        let sanitized_url = if clean_query_pairs.is_empty() {
            base_url.to_string()
        } else {
            format!("{}?{}", base_url, clean_query_pairs.join("&"))
        };

        let was_modified = !stripped_params.is_empty();

        StripResult {
            original_url: raw_url.to_string(),
            sanitized_url,
            stripped_params,
            was_modified,
        }
    }
}

