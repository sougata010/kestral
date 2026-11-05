use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdBlockStats {
    pub total_blocked: u64,
    pub ads_blocked: u64,
    pub trackers_blocked: u64,
    pub rules_count: usize,
    pub cosmetic_filters_count: usize,
}

pub struct AdBlockEngine {
    blocked_domains: Mutex<HashSet<String>>,
    tracker_domains: Mutex<HashSet<String>>,
    cosmetic_selectors: Mutex<Vec<String>>,
    stats: Mutex<AdBlockStats>,
}

impl AdBlockEngine {
    pub fn new() -> Self {
        let mut blocked = HashSet::new();
        let mut trackers = HashSet::new();

        // Standard EasyList & Peter Lowe's domain blocklist sample
        let default_ad_domains = [
            "doubleclick.net",
            "googleads.g.doubleclick.net",
            "pagead2.googlesyndication.com",
            "adservice.google.com",
            "adsystem.amazon.com",
            "adnxs.com",
            "criteo.com",
            "advertising.com",
            "bidswitch.net",
            "casale-media.com",
            "taboola.com",
            "outbrain.com",
            "popcash.net",
            "popads.net",
            "adcolony.com",
            "chartbeat.net",
            "scorecardresearch.com",
            "quantserve.com",
            "zedo.com",
            "rubiconproject.com",
        ];

        let default_trackers = [
            "analytics.google.com",
            "google-analytics.com",
            "hotjar.com",
            "fullstory.com",
            "segment.io",
            "mixpanel.com",
            "mouseflow.com",
            "crazyegg.com",
            "inspectlet.com",
            "facebook.com/tr",
            "connect.facebook.net",
            "pixel.wp.com",
            "statcounter.com",
            "yandex.ru/metrika",
        ];

        for d in default_ad_domains {
            blocked.insert(d.to_string());
        }

        for t in default_trackers {
            trackers.insert(t.to_string());
        }

        let cosmetic = vec![
            ".adsbygoogle".to_string(),
            ".ad-container".to_string(),
            ".ad-wrapper".to_string(),
            "#sponsored-posts".to_string(),
            "[id^='div-gpt-ad']".to_string(),
            ".trc_related_container".to_string(),
            ".OUTBRAIN".to_string(),
        ];

        let total_rules = blocked.len() + trackers.len() + cosmetic.len();

        Self {
            blocked_domains: Mutex::new(blocked),
            tracker_domains: Mutex::new(trackers),
            cosmetic_selectors: Mutex::new(cosmetic.clone()),
            stats: Mutex::new(AdBlockStats {
                total_blocked: 0,
                ads_blocked: 0,
                trackers_blocked: 0,
                rules_count: total_rules,
                cosmetic_filters_count: cosmetic.len(),
            }),
        }
    }

    pub fn check_url(&self, url_str: &str) -> (bool, bool) {
        let lower = url_str.to_lowercase();
        let blocked = self.blocked_domains.lock().unwrap();
        let trackers = self.tracker_domains.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        for domain in blocked.iter() {
            if lower.contains(domain) {
                stats.total_blocked += 1;
                stats.ads_blocked += 1;
                return (true, false);
            }
        }

        for tracker in trackers.iter() {
            if lower.contains(tracker) {
                stats.total_blocked += 1;
                stats.trackers_blocked += 1;
                return (false, true);
            }
        }

        (false, false)
    }

    pub fn get_cosmetic_css(&self) -> String {
        let selectors = self.cosmetic_selectors.lock().unwrap();
        format!("{} {{ display: none !important; visibility: hidden !important; height: 0 !important; }}", selectors.join(", "))
    }

    pub fn get_stats(&self) -> AdBlockStats {
        self.stats.lock().unwrap().clone()
    }
}

