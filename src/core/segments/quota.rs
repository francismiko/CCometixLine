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
    remaining_usd: f64,
    daily_limit_usd: f64,
    total_cost_usd: f64,
    request_count: u64,
    can_make_request: bool,
    api_healthy: bool,
}

// API response structure for relay.nf.video/v1/usage
#[derive(Debug, Deserialize)]
struct ApiResponse {
    usage: UsageData,
    limits: LimitsData,
}

#[derive(Debug, Deserialize)]
struct UsageData {
    #[serde(rename = "remainingUSD")]
    remaining_usd: f64,
    #[serde(rename = "dailyLimitUSD")]
    daily_limit_usd: f64,
    #[serde(rename = "totalCostUSD")]
    total_cost_usd: f64,
    #[serde(rename = "requestCount")]
    request_count: u64,
    #[serde(rename = "canMakeRequest")]
    can_make_request: bool,
}

#[derive(Debug, Deserialize)]
struct LimitsData {
    #[serde(rename = "dailyUSD")]
    daily_usd: f64,
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
        let auth_header = format!("Bearer {}", token);

        let response = ureq::get(&config.api_url)
            .set("Authorization", &auth_header)
            .set("Accept", "application/json")
            .timeout(std::time::Duration::from_secs(config.timeout))
            .call()
            .ok()?;

        let api_response: ApiResponse = response.into_json().ok()?;

        Some(QuotaData {
            remaining_usd: api_response.usage.remaining_usd,
            daily_limit_usd: api_response.usage.daily_limit_usd,
            total_cost_usd: api_response.usage.total_cost_usd,
            request_count: api_response.usage.request_count,
            can_make_request: api_response.usage.can_make_request,
            api_healthy: true,
        })
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

    fn format_usd(n: f64) -> String {
        if n >= 1000.0 {
            format!("{:.1}k", n / 1000.0)
        } else if n >= 100.0 {
            format!("{:.0}", n)
        } else {
            format!("{:.1}", n)
        }
    }

    fn get_battery_icon(remaining_pct: f64) -> &'static str {
        if remaining_pct > 0.95 {
            "󰁹"  // battery-100
        } else if remaining_pct > 0.85 {
            "󰂂"  // battery-90
        } else if remaining_pct > 0.75 {
            "󰂁"  // battery-80
        } else if remaining_pct > 0.65 {
            "󰂀"  // battery-70
        } else if remaining_pct > 0.55 {
            "󰁿"  // battery-60
        } else if remaining_pct > 0.45 {
            "󰁾"  // battery-50
        } else if remaining_pct > 0.35 {
            "󰁽"  // battery-40
        } else if remaining_pct > 0.25 {
            "󰁼"  // battery-30
        } else if remaining_pct > 0.15 {
            "󰁻"  // battery-20
        } else if remaining_pct > 0.05 {
            "󰁺"  // battery-10
        } else {
            "󰂃"  // battery-alert (empty with !)
        }
    }

    // 256-color gradient matching ANSI 16-color style used by other segments
    // Uses colors similar to c16 bright colors (9-14)
    fn get_color_code(remaining_pct: f64) -> String {
        if remaining_pct > 0.90 {
            "38;5;10".to_string()   // bright green (like directory)
        } else if remaining_pct > 0.80 {
            "38;5;10".to_string()   // bright green
        } else if remaining_pct > 0.70 {
            "38;5;10".to_string()   // bright green
        } else if remaining_pct > 0.60 {
            "38;5;14".to_string()   // bright cyan
        } else if remaining_pct > 0.50 {
            "38;5;11".to_string()   // bright yellow
        } else if remaining_pct > 0.40 {
            "38;5;11".to_string()   // bright yellow
        } else if remaining_pct > 0.30 {
            "38;5;3".to_string()    // yellow/orange
        } else if remaining_pct > 0.20 {
            "38;5;9".to_string()    // bright red
        } else if remaining_pct > 0.10 {
            "38;5;9".to_string()    // bright red
        } else {
            "38;5;1".to_string()    // red
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
            api_url: "https://relay.nf.video/v1/usage".to_string(),
            cache_ttl: 30,
            timeout: 5,
            show_requests: false,
            warning_threshold: 0.15,
        }
    }
}

impl Segment for QuotaSegment {
    fn collect(&self, _input: &InputData) -> Option<SegmentData> {
        let config = Self::load_quota_config();
        let data = Self::get_quota_data(&config)?;

        let remaining_pct = data.remaining_usd / data.daily_limit_usd;

        // Health indicator
        let health = if data.api_healthy { "✓" } else { "✗" };

        // Dynamic battery icon based on remaining percentage
        let battery = Self::get_battery_icon(remaining_pct);

        // Dynamic color based on remaining percentage
        let color = Self::get_color_code(remaining_pct);

        // Block indicator when cannot make request
        let block = if !data.can_make_request { "🚫" } else { "" };

        // Format with ANSI color codes
        // Use dynamic_icon for battery, amount in white, health indicator at end
        let battery_colored = format!("\x1b[{}m{}\x1b[0m", color, battery);

        let primary = format!(
            "{}\x1b[37m${}/{}\x1b[0m {}",
            block,
            Self::format_usd(data.remaining_usd),
            Self::format_usd(data.daily_limit_usd),
            health
        );

        let secondary = if config.show_requests {
            format!("{}次", data.request_count)
        } else {
            String::new()
        };

        let mut metadata = HashMap::new();
        metadata.insert("dynamic_icon".to_string(), battery_colored);
        metadata.insert("remaining_usd".to_string(), data.remaining_usd.to_string());
        metadata.insert("daily_limit_usd".to_string(), data.daily_limit_usd.to_string());
        metadata.insert("total_cost_usd".to_string(), data.total_cost_usd.to_string());
        metadata.insert("request_count".to_string(), data.request_count.to_string());
        metadata.insert("remaining_pct".to_string(), format!("{:.1}", remaining_pct * 100.0));
        metadata.insert("can_make_request".to_string(), data.can_make_request.to_string());
        metadata.insert("api_healthy".to_string(), data.api_healthy.to_string());

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
