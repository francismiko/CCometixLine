use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct QuotaSegment;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuotaCache {
    fetched_at: u64,
    data: QuotaData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct QuotaData {
    daily_remaining: f64,
    daily_total: f64,
    month_remaining: f64,
    month_total: f64,
    today_requests: u64,
    today_cost: f64,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: QuotaData,
    success: bool,
}

impl QuotaSegment {
    pub fn new() -> Self {
        Self
    }

    fn get_config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("ccline")
    }

    fn get_cache_path() -> PathBuf {
        Self::get_config_dir().join("quota_cache.json")
    }

    fn get_token_path() -> PathBuf {
        Self::get_config_dir().join("quota_token")
    }

    fn get_config_path() -> PathBuf {
        Self::get_config_dir().join("quota.toml")
    }

    fn load_quota_config() -> QuotaConfig {
        let config_path = Self::get_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        QuotaConfig::default()
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn load_cache() -> Option<QuotaCache> {
        let cache_path = Self::get_cache_path();
        if !cache_path.exists() {
            return None;
        }
        let content = fs::read_to_string(&cache_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_cache(cache: &QuotaCache) {
        let cache_path = Self::get_cache_path();
        if let Ok(content) = serde_json::to_string_pretty(cache) {
            let _ = fs::write(&cache_path, content);
        }
    }

    fn load_token() -> Option<String> {
        let token_path = Self::get_token_path();
        fs::read_to_string(&token_path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn fetch_quota(config: &QuotaConfig, token: &str) -> Option<QuotaData> {
        let cookie = format!("satoken-user={}", token);

        let response = ureq::get(&config.api_url)
            .set("Cookie", &cookie)
            .set("Accept", "application/json")
            .timeout(std::time::Duration::from_secs(config.timeout))
            .call()
            .ok()?;

        let api_response: ApiResponse = response.into_json().ok()?;

        if api_response.success {
            Some(api_response.data)
        } else {
            None
        }
    }

    fn get_quota_data(config: &QuotaConfig) -> Option<QuotaData> {
        let now = Self::current_timestamp();

        // Check cache first
        if let Some(cache) = Self::load_cache() {
            if now - cache.fetched_at < config.cache_ttl {
                return Some(cache.data);
            }
        }

        // Load token
        let token = Self::load_token()?;

        // Fetch new data
        if let Some(data) = Self::fetch_quota(config, &token) {
            let cache = QuotaCache {
                fetched_at: now,
                data: data.clone(),
            };
            Self::save_cache(&cache);
            return Some(data);
        }

        // Fallback to stale cache if fetch failed
        Self::load_cache().map(|c| c.data)
    }

    fn format_number(n: f64) -> String {
        if n >= 1000.0 {
            format!("{:.1}k", n / 1000.0)
        } else if n >= 100.0 {
            format!("{:.0}", n)
        } else {
            format!("{:.1}", n)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuotaConfig {
    api_url: String,
    cache_ttl: u64,
    timeout: u64,
    show_requests: bool,
    warning_threshold: f64,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            api_url: "https://cc.yhlxj.com/8081/api/applet/claude/code/get/dashboard".to_string(),
            cache_ttl: 60,
            timeout: 3,
            show_requests: false,
            warning_threshold: 0.15,
        }
    }
}

impl Segment for QuotaSegment {
    fn collect(&self, _input: &InputData) -> Option<SegmentData> {
        let config = Self::load_quota_config();
        let data = Self::get_quota_data(&config)?;

        let daily_pct = data.daily_remaining / data.daily_total;
        let month_pct = data.month_remaining / data.month_total;

        // Warning indicator for low quota
        let daily_warn = if daily_pct < config.warning_threshold {
            "⚠"
        } else {
            ""
        };

        let primary = format!(
            "{}日 {}/{}",
            daily_warn,
            Self::format_number(data.daily_remaining),
            Self::format_number(data.daily_total)
        );

        let secondary = format!(
            "月 {}/{}",
            Self::format_number(data.month_remaining),
            Self::format_number(data.month_total)
        );

        let mut metadata = HashMap::new();
        metadata.insert("daily_remaining".to_string(), data.daily_remaining.to_string());
        metadata.insert("daily_total".to_string(), data.daily_total.to_string());
        metadata.insert("month_remaining".to_string(), data.month_remaining.to_string());
        metadata.insert("month_total".to_string(), data.month_total.to_string());
        metadata.insert("daily_pct".to_string(), format!("{:.1}", daily_pct * 100.0));
        metadata.insert("month_pct".to_string(), format!("{:.1}", month_pct * 100.0));

        if config.show_requests {
            metadata.insert("today_requests".to_string(), data.today_requests.to_string());
        }

        Some(SegmentData {
            primary,
            secondary,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Quota
    }
}
