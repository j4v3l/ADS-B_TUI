use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DEFAULT_URL: &str = "http://adsb.local/data/aircraft.json";
pub const DEFAULT_REFRESH_SECS: u64 = 2;
pub const DEFAULT_STALE_SECS: u64 = 60;
pub const DEFAULT_HIDE_STALE: bool = false;
pub const DEFAULT_LOW_NIC: i64 = 5;
pub const DEFAULT_LOW_NAC: i64 = 8;
pub const DEFAULT_TRAIL_LEN: u64 = 6;
pub const DEFAULT_FAVORITES_FILE: &str = "adsb-favorites.txt";
pub const DEFAULT_WATCHLIST_FILE: &str = "adsb-watchlist.toml";
pub const DEFAULT_WATCHLIST_ENABLED: bool = true;
pub const DEFAULT_ALLOW_HTTP: bool = true;
pub const DEFAULT_API_KEY_HEADER: &str = "api-auth";
pub const DEFAULT_ROUTE_BASE: &str = "https://api.airplanes.live";
pub const DEFAULT_ROUTE_TTL_SECS: u64 = 3600;
pub const DEFAULT_ROUTE_REFRESH_SECS: u64 = 15;
pub const DEFAULT_ROUTE_BATCH: u64 = 20;
pub const DEFAULT_ROUTE_TIMEOUT_SECS: u64 = 6;
pub const DEFAULT_ROUTE_MODE: &str = "routeset";
pub const DEFAULT_ROUTE_PATH: &str = "tar1090/data/routes.json";
pub const DEFAULT_UI_FPS: u64 = 10;
pub const DEFAULT_SMOOTH_MODE: bool = true;
pub const DEFAULT_SMOOTH_MERGE: bool = true;
pub const DEFAULT_RATE_WINDOW_MS: u64 = 300;
pub const DEFAULT_RATE_MIN_SECS: f64 = 0.25;
pub const DEFAULT_NOTIFY_RADIUS_MI: f64 = 10.0;
pub const DEFAULT_OVERPASS_MI: f64 = 0.5;
pub const DEFAULT_NOTIFY_COOLDOWN_SECS: u64 = 120;
pub const DEFAULT_ALTITUDE_TREND_ARROWS: bool = true;
pub const DEFAULT_COLUMN_CACHE: bool = true;
pub const DEFAULT_TRACK_ARROWS: bool = true;
pub const DEFAULT_STATS_METRIC_1: &str = "msg_rate_total";
pub const DEFAULT_STATS_METRIC_2: &str = "kbps_total";
pub const DEFAULT_STATS_METRIC_3: &str = "msg_rate_avg";
pub const DEFAULT_FLAGS_ENABLED: bool = true;
pub const DEFAULT_FLAG_STYLE: &str = "emoji";
pub const DEFAULT_DEMO_MODE: bool = false;
pub const DEFAULT_RADAR_RANGE_NM: f64 = 200.0;
pub const DEFAULT_RADAR_ASPECT: f64 = 1.0;
pub const DEFAULT_RADAR_RENDERER: &str = "canvas";
pub const DEFAULT_RADAR_LABELS: bool = false;
pub const DEFAULT_RADAR_BLIP: &str = "dot";
pub const DEFAULT_ROLE_ENABLED: bool = true;
pub const DEFAULT_ROLE_HIGHLIGHT: bool = true;

#[derive(Debug, Clone)]
pub struct Config {
    pub url: String,
    pub urls: Vec<String>,
    pub refresh: Duration,
    pub insecure: bool,
    pub allow_http: bool,
    pub allow_insecure: bool,
    pub config_path: PathBuf,
    pub stale_secs: u64,
    pub hide_stale: bool,
    pub low_nic: i64,
    pub low_nac: i64,
    pub trail_len: u64,
    pub favorites: Vec<String>,
    pub favorites_file: String,
    pub watchlist_enabled: bool,
    pub watchlist_file: String,
    pub api_key: String,
    pub api_key_header: String,
    pub log_enabled: bool,
    pub log_level: String,
    pub log_file: String,
    pub filter: String,
    pub layout: String,
    pub theme: String,
    pub radar_range_nm: f64,
    pub radar_aspect: f64,
    pub radar_renderer: String,
    pub radar_labels: bool,
    pub radar_blip: String,
    pub site_lat: Option<f64>,
    pub site_lon: Option<f64>,
    pub site_alt_m: Option<f64>,
    pub route_enabled: bool,
    pub route_base: String,
    pub route_ttl_secs: u64,
    pub route_refresh_secs: u64,
    pub route_batch: u64,
    pub route_timeout_secs: u64,
    pub route_mode: String,
    pub route_path: String,
    pub ui_fps: u64,
    pub smooth_mode: bool,
    pub smooth_merge: bool,
    pub rate_window_ms: u64,
    pub rate_min_secs: f64,
    pub notify_radius_mi: f64,
    pub overpass_mi: f64,
    pub notify_cooldown_secs: u64,
    pub altitude_trend_arrows: bool,
    pub column_cache: bool,
    pub track_arrows: bool,
    pub flags_enabled: bool,
    pub flag_style: String,
    pub demo_mode: bool,
    pub stats_metric_1: String,
    pub stats_metric_2: String,
    pub stats_metric_3: String,
    pub role_enabled: bool,
    pub role_highlight: bool,
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    url: Option<String>,
    urls: Option<Vec<String>>,
    refresh_secs: Option<u64>,
    insecure: Option<bool>,
    allow_http: Option<bool>,
    allow_insecure: Option<bool>,
    stale_secs: Option<u64>,
    hide_stale: Option<bool>,
    low_nic: Option<i64>,
    low_nac: Option<i64>,
    trail_len: Option<u64>,
    favorites: Option<Vec<String>>,
    favorites_file: Option<String>,
    watchlist_enabled: Option<bool>,
    watchlist_file: Option<String>,
    api_key: Option<String>,
    api_key_header: Option<String>,
    log_enabled: Option<bool>,
    log_level: Option<String>,
    log_file: Option<String>,
    filter: Option<String>,
    layout: Option<String>,
    theme: Option<String>,
    radar_range_nm: Option<f64>,
    radar_aspect: Option<f64>,
    radar_renderer: Option<String>,
    radar_labels: Option<bool>,
    radar_blip: Option<String>,
    site_lat: Option<f64>,
    site_lon: Option<f64>,
    site_alt_m: Option<f64>,
    route_enabled: Option<bool>,
    route_base: Option<String>,
    route_ttl_secs: Option<u64>,
    route_refresh_secs: Option<u64>,
    route_batch: Option<u64>,
    route_timeout_secs: Option<u64>,
    route_mode: Option<String>,
    route_path: Option<String>,
    ui_fps: Option<u64>,
    smooth_mode: Option<bool>,
    smooth_merge: Option<bool>,
    rate_window_ms: Option<u64>,
    rate_min_secs: Option<f64>,
    notify_radius_mi: Option<f64>,
    overpass_mi: Option<f64>,
    notify_cooldown_secs: Option<u64>,
    altitude_trend_arrows: Option<bool>,
    column_cache: Option<bool>,
    track_arrows: Option<bool>,
    flags_enabled: Option<bool>,
    flag_style: Option<String>,
    demo_mode: Option<bool>,
    stats_metric_1: Option<String>,
    stats_metric_2: Option<String>,
    stats_metric_3: Option<String>,
    role_enabled: Option<bool>,
    role_highlight: Option<bool>,
}

pub fn parse_args() -> Result<Config> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut explicit_config: Option<PathBuf> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--config" {
            let value = iter
                .next()
                .ok_or_else(|| anyhow!("--config needs a value"))?;
            explicit_config = Some(PathBuf::from(value));
        }
    }

    let env_config = env::var("ADSB_CONFIG").ok().map(PathBuf::from);
    let config_path = explicit_config
        .clone()
        .or(env_config)
        .unwrap_or_else(|| PathBuf::from("adsb-tui.toml"));

    let mut config = Config {
        url: DEFAULT_URL.to_string(),
        urls: vec![DEFAULT_URL.to_string()],
        refresh: Duration::from_secs(DEFAULT_REFRESH_SECS),
        insecure: false,
        allow_http: DEFAULT_ALLOW_HTTP,
        allow_insecure: false,
        config_path: config_path.clone(),
        stale_secs: DEFAULT_STALE_SECS,
        hide_stale: DEFAULT_HIDE_STALE,
        low_nic: DEFAULT_LOW_NIC,
        low_nac: DEFAULT_LOW_NAC,
        trail_len: DEFAULT_TRAIL_LEN,
        favorites: Vec::new(),
        favorites_file: DEFAULT_FAVORITES_FILE.to_string(),
        watchlist_enabled: DEFAULT_WATCHLIST_ENABLED,
        watchlist_file: DEFAULT_WATCHLIST_FILE.to_string(),
        api_key: String::new(),
        api_key_header: DEFAULT_API_KEY_HEADER.to_string(),
        log_enabled: false,
        log_level: "info".to_string(),
        log_file: "adsb-tui.log".to_string(),
        filter: String::new(),
        layout: "full".to_string(),
        theme: "default".to_string(),
        radar_range_nm: DEFAULT_RADAR_RANGE_NM,
        radar_aspect: DEFAULT_RADAR_ASPECT,
        radar_renderer: DEFAULT_RADAR_RENDERER.to_string(),
        radar_labels: DEFAULT_RADAR_LABELS,
        radar_blip: DEFAULT_RADAR_BLIP.to_string(),
        site_lat: None,
        site_lon: None,
        site_alt_m: None,
        route_enabled: true,
        route_base: DEFAULT_ROUTE_BASE.to_string(),
        route_ttl_secs: DEFAULT_ROUTE_TTL_SECS,
        route_refresh_secs: DEFAULT_ROUTE_REFRESH_SECS,
        route_batch: DEFAULT_ROUTE_BATCH,
        route_timeout_secs: DEFAULT_ROUTE_TIMEOUT_SECS,
        route_mode: DEFAULT_ROUTE_MODE.to_string(),
        route_path: DEFAULT_ROUTE_PATH.to_string(),
        ui_fps: DEFAULT_UI_FPS,
        smooth_mode: DEFAULT_SMOOTH_MODE,
        smooth_merge: DEFAULT_SMOOTH_MERGE,
        rate_window_ms: DEFAULT_RATE_WINDOW_MS,
        rate_min_secs: DEFAULT_RATE_MIN_SECS,
        notify_radius_mi: DEFAULT_NOTIFY_RADIUS_MI,
        overpass_mi: DEFAULT_OVERPASS_MI,
        notify_cooldown_secs: DEFAULT_NOTIFY_COOLDOWN_SECS,
        altitude_trend_arrows: DEFAULT_ALTITUDE_TREND_ARROWS,
        column_cache: DEFAULT_COLUMN_CACHE,
        track_arrows: DEFAULT_TRACK_ARROWS,
        flags_enabled: DEFAULT_FLAGS_ENABLED,
        flag_style: DEFAULT_FLAG_STYLE.to_string(),
        demo_mode: DEFAULT_DEMO_MODE,
        stats_metric_1: DEFAULT_STATS_METRIC_1.to_string(),
        stats_metric_2: DEFAULT_STATS_METRIC_2.to_string(),
        stats_metric_3: DEFAULT_STATS_METRIC_3.to_string(),
        role_enabled: DEFAULT_ROLE_ENABLED,
        role_highlight: DEFAULT_ROLE_HIGHLIGHT,
    };

    if config_path.exists() {
        if let Some(file_config) = load_file_config(&config_path)? {
            apply_file_config(&mut config, file_config);
        }
    } else if explicit_config.is_some() {
        return Err(anyhow!("Config file not found: {}", config_path.display()));
    }

    config.config_path = config_path.clone();

    if let Ok(url) = env::var("ADSB_URL") {
        config.url = url;
    }
    if let Ok(value) = env::var("ADSB_URLS") {
        let urls: Vec<String> = value
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
        if !urls.is_empty() {
            config.urls = urls;
        }
    }
    if let Ok(value) = env::var("ADSB_REFRESH") {
        if let Ok(secs) = value.parse::<u64>() {
            config.refresh = Duration::from_secs(secs);
        }
    }
    if let Ok(value) = env::var("ADSB_INSECURE") {
        config.insecure = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_ALLOW_HTTP") {
        config.allow_http = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_ALLOW_INSECURE") {
        config.allow_insecure = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_STALE_SECS") {
        if let Ok(secs) = value.parse::<u64>() {
            config.stale_secs = secs.max(1);
        }
    }
    if let Ok(value) = env::var("ADSB_HIDE_STALE") {
        config.hide_stale = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_LOW_NIC") {
        if let Ok(val) = value.parse::<i64>() {
            config.low_nic = val;
        }
    }
    if let Ok(value) = env::var("ADSB_LOW_NAC") {
        if let Ok(val) = value.parse::<i64>() {
            config.low_nac = val;
        }
    }
    if let Ok(value) = env::var("ADSB_TRAIL_LEN") {
        if let Ok(val) = value.parse::<u64>() {
            config.trail_len = val.max(1);
        }
    }
    if let Ok(value) = env::var("ADSB_FILTER") {
        config.filter = value;
    }
    if let Ok(value) = env::var("ADSB_FAVORITES") {
        config.favorites = value
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
    }
    if let Ok(value) = env::var("ADSB_FAVORITES_FILE") {
        config.favorites_file = value;
    }
    if let Ok(value) = env::var("ADSB_API_KEY") {
        config.api_key = value;
    }
    if let Ok(value) = env::var("ADSB_API_KEY_HEADER") {
        config.api_key_header = value;
    }
    if let Ok(value) = env::var("ADSB_LOG_ENABLED") {
        config.log_enabled = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_LOG_LEVEL") {
        config.log_level = value;
    }

    if config.urls.is_empty() {
        config.urls.push(config.url.clone());
    }
    if config.url.trim().is_empty() {
        if let Some(first) = config.urls.first() {
            config.url = first.clone();
        }
    } else if let Some(first) = config.urls.first() {
        if first != &config.url {
            let mut urls = config.urls.clone();
            urls.retain(|u| !u.trim().is_empty());
            if urls.is_empty() {
                urls.push(config.url.clone());
            } else {
                urls[0] = config.url.clone();
            }
            config.urls = urls;
        }
    }
    if let Ok(value) = env::var("ADSB_LOG_FILE") {
        config.log_file = value;
    }
    if let Ok(value) = env::var("ADSB_WATCHLIST_ENABLED") {
        config.watchlist_enabled = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_WATCHLIST_FILE") {
        config.watchlist_file = value;
    }
    if let Ok(value) = env::var("ADSB_LAYOUT") {
        config.layout = value;
    }
    if let Ok(value) = env::var("ADSB_THEME") {
        config.theme = value;
    }
    if let Ok(value) = env::var("ADSB_FLAG_STYLE") {
        config.flag_style = value;
    }
    if let Ok(value) = env::var("ADSB_DEMO_MODE") {
        config.demo_mode = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_RADAR_RANGE_NM") {
        if let Ok(val) = value.parse::<f64>() {
            config.radar_range_nm = val.max(1.0);
        }
    }
    if let Ok(value) = env::var("ADSB_RADAR_ASPECT") {
        if let Ok(val) = value.parse::<f64>() {
            config.radar_aspect = val.max(0.2);
        }
    }
    if let Ok(value) = env::var("ADSB_RADAR_RENDERER") {
        config.radar_renderer = value;
    }
    if let Ok(value) = env::var("ADSB_RADAR_LABELS") {
        config.radar_labels = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_RADAR_BLIP") {
        config.radar_blip = value;
    }
    if let Ok(value) = env::var("ADSB_SITE_LAT") {
        if let Ok(val) = value.parse::<f64>() {
            config.site_lat = Some(val);
        }
    }
    if let Ok(value) = env::var("ADSB_SITE_LON") {
        if let Ok(val) = value.parse::<f64>() {
            config.site_lon = Some(val);
        }
    }
    if let Ok(value) = env::var("ADSB_SITE_ALT_M") {
        if let Ok(val) = value.parse::<f64>() {
            config.site_alt_m = Some(val);
        }
    }
    if let Ok(value) = env::var("ADSB_ROUTE_ENABLED") {
        config.route_enabled = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_ROUTE_BASE") {
        config.route_base = value;
    }
    if let Ok(value) = env::var("ADSB_ROUTE_TTL") {
        if let Ok(val) = value.parse::<u64>() {
            config.route_ttl_secs = val;
        }
    }
    if let Ok(value) = env::var("ADSB_ROUTE_REFRESH") {
        if let Ok(val) = value.parse::<u64>() {
            config.route_refresh_secs = val;
        }
    }
    if let Ok(value) = env::var("ADSB_ROUTE_BATCH") {
        if let Ok(val) = value.parse::<u64>() {
            config.route_batch = val.max(1);
        }
    }
    if let Ok(value) = env::var("ADSB_ROUTE_TIMEOUT") {
        if let Ok(val) = value.parse::<u64>() {
            config.route_timeout_secs = val.max(2);
        }
    }
    if let Ok(value) = env::var("ADSB_ROUTE_MODE") {
        config.route_mode = value;
    }
    if let Ok(value) = env::var("ADSB_ROUTE_PATH") {
        config.route_path = value;
    }
    if let Ok(value) = env::var("ADSB_UI_FPS") {
        if let Ok(val) = value.parse::<u64>() {
            config.ui_fps = val;
        }
    }
    if let Ok(value) = env::var("ADSB_SMOOTH") {
        config.smooth_mode = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_SMOOTH_MERGE") {
        config.smooth_merge = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_RATE_WINDOW_MS") {
        if let Ok(val) = value.parse::<u64>() {
            config.rate_window_ms = val.max(50);
        }
    }
    if let Ok(value) = env::var("ADSB_RATE_MIN_SECS") {
        if let Ok(val) = value.parse::<f64>() {
            config.rate_min_secs = val.max(0.05);
        }
    }
    if let Ok(value) = env::var("ADSB_NOTIFY_MI") {
        if let Ok(val) = value.parse::<f64>() {
            config.notify_radius_mi = val.max(0.1);
        }
    }
    if let Ok(value) = env::var("ADSB_OVERPASS_MI") {
        if let Ok(val) = value.parse::<f64>() {
            config.overpass_mi = val.max(0.05);
        }
    }
    if let Ok(value) = env::var("ADSB_NOTIFY_COOLDOWN") {
        if let Ok(val) = value.parse::<u64>() {
            config.notify_cooldown_secs = val.max(10);
        }
    }
    if let Ok(value) = env::var("ADSB_ALT_TREND") {
        config.altitude_trend_arrows = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_COLUMN_CACHE") {
        config.column_cache = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_TRACK_ARROWS") {
        config.track_arrows = matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = env::var("ADSB_STATS_METRIC_1") {
        config.stats_metric_1 = value;
    }
    if let Ok(value) = env::var("ADSB_STATS_METRIC_2") {
        config.stats_metric_2 = value;
    }
    if let Ok(value) = env::var("ADSB_STATS_METRIC_3") {
        config.stats_metric_3 = value;
    }

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => {
                iter.next();
            }
            "--url" => {
                config.url = iter
                    .next()
                    .ok_or_else(|| anyhow!("--url needs a value"))?
                    .to_string();
            }
            "--refresh" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--refresh needs a value"))?;
                let secs: u64 = value.parse()?;
                config.refresh = Duration::from_secs(secs);
            }
            "--insecure" => {
                config.insecure = true;
            }
            "--allow-http" => {
                config.allow_http = true;
            }
            "--allow-insecure" => {
                config.allow_insecure = true;
            }
            "--stale" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--stale needs a value"))?;
                let secs: u64 = value.parse()?;
                config.stale_secs = secs.max(1);
            }
            "--hide-stale" => {
                config.hide_stale = true;
            }
            "--show-stale" => {
                config.hide_stale = false;
            }
            "--low-nic" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--low-nic needs a value"))?;
                config.low_nic = value.parse()?;
            }
            "--low-nac" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--low-nac needs a value"))?;
                config.low_nac = value.parse()?;
            }
            "--trail" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--trail needs a value"))?;
                let len: u64 = value.parse()?;
                config.trail_len = len.max(1);
            }
            "--filter" => {
                config.filter = iter
                    .next()
                    .ok_or_else(|| anyhow!("--filter needs a value"))?
                    .to_string();
            }
            "--favorite" => {
                if let Some(value) = iter.next() {
                    config.favorites.push(value.to_string());
                } else {
                    return Err(anyhow!("--favorite needs a value"));
                }
            }
            "--favorites-file" => {
                config.favorites_file = iter
                    .next()
                    .ok_or_else(|| anyhow!("--favorites-file needs a value"))?
                    .to_string();
            }
            "--api-key" => {
                config.api_key = iter
                    .next()
                    .ok_or_else(|| anyhow!("--api-key needs a value"))?
                    .to_string();
            }
            "--api-key-header" => {
                config.api_key_header = iter
                    .next()
                    .ok_or_else(|| anyhow!("--api-key-header needs a value"))?
                    .to_string();
            }
            "--log" => {
                config.log_enabled = true;
            }
            "--no-log" => {
                config.log_enabled = false;
            }
            "--log-level" => {
                config.log_level = iter
                    .next()
                    .ok_or_else(|| anyhow!("--log-level needs a value"))?
                    .to_string();
            }
            "--log-file" => {
                config.log_file = iter
                    .next()
                    .ok_or_else(|| anyhow!("--log-file needs a value"))?
                    .to_string();
            }
            "--watchlist-file" => {
                config.watchlist_file = iter
                    .next()
                    .ok_or_else(|| anyhow!("--watchlist-file needs a value"))?
                    .to_string();
            }
            "--watchlist" => {
                config.watchlist_enabled = true;
            }
            "--no-watchlist" => {
                config.watchlist_enabled = false;
            }
            "--layout" => {
                config.layout = iter
                    .next()
                    .ok_or_else(|| anyhow!("--layout needs a value"))?
                    .to_string();
            }
            "--theme" => {
                config.theme = iter
                    .next()
                    .ok_or_else(|| anyhow!("--theme needs a value"))?
                    .to_string();
            }
            "--flag-style" => {
                config.flag_style = iter
                    .next()
                    .ok_or_else(|| anyhow!("--flag-style needs a value"))?
                    .to_string();
            }
            "--demo-mode" => {
                config.demo_mode = true;
            }
            "--no-demo-mode" => {
                config.demo_mode = false;
            }
            "--radar-range-nm" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--radar-range-nm needs a value"))?;
                config.radar_range_nm = value.parse::<f64>()?.max(1.0);
            }
            "--radar-aspect" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--radar-aspect needs a value"))?;
                config.radar_aspect = value.parse::<f64>()?.max(0.2);
            }
            "--radar-renderer" => {
                config.radar_renderer = iter
                    .next()
                    .ok_or_else(|| anyhow!("--radar-renderer needs a value"))?
                    .to_string();
            }
            "--radar-blip" => {
                config.radar_blip = iter
                    .next()
                    .ok_or_else(|| anyhow!("--radar-blip needs a value"))?
                    .to_string();
            }
            "--radar-labels" => {
                config.radar_labels = true;
            }
            "--no-radar-labels" => {
                config.radar_labels = false;
            }
            "--site-lat" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--site-lat needs a value"))?;
                config.site_lat = Some(value.parse()?);
            }
            "--site-lon" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--site-lon needs a value"))?;
                config.site_lon = Some(value.parse()?);
            }
            "--site-alt-m" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--site-alt-m needs a value"))?;
                config.site_alt_m = Some(value.parse()?);
            }
            "--route-base" => {
                config.route_base = iter
                    .next()
                    .ok_or_else(|| anyhow!("--route-base needs a value"))?
                    .to_string();
            }
            "--route-ttl" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--route-ttl needs a value"))?;
                let val: u64 = value.parse()?;
                config.route_ttl_secs = val;
            }
            "--route-refresh" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--route-refresh needs a value"))?;
                let val: u64 = value.parse()?;
                config.route_refresh_secs = val;
            }
            "--route-batch" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--route-batch needs a value"))?;
                let val: u64 = value.parse()?;
                config.route_batch = val.max(1);
            }
            "--route-timeout" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--route-timeout needs a value"))?;
                let val: u64 = value.parse()?;
                config.route_timeout_secs = val.max(2);
            }
            "--route-disable" => {
                config.route_enabled = false;
            }
            "--route-mode" => {
                config.route_mode = iter
                    .next()
                    .ok_or_else(|| anyhow!("--route-mode needs a value"))?
                    .to_string();
            }
            "--route-path" => {
                config.route_path = iter
                    .next()
                    .ok_or_else(|| anyhow!("--route-path needs a value"))?
                    .to_string();
            }
            "--ui-fps" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--ui-fps needs a value"))?;
                config.ui_fps = value.parse()?;
            }
            "--smooth" => {
                config.smooth_mode = true;
            }
            "--no-smooth" => {
                config.smooth_mode = false;
            }
            "--smooth-merge" => {
                config.smooth_merge = true;
            }
            "--no-smooth-merge" => {
                config.smooth_merge = false;
            }
            "--alt-arrows" => {
                config.altitude_trend_arrows = true;
            }
            "--no-alt-arrows" => {
                config.altitude_trend_arrows = false;
            }
            "--rate-window-ms" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--rate-window-ms needs a value"))?;
                config.rate_window_ms = value.parse::<u64>()?.max(50);
            }
            "--rate-min-secs" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--rate-min-secs needs a value"))?;
                config.rate_min_secs = value.parse::<f64>()?.max(0.05);
            }
            "--notify-mi" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--notify-mi needs a value"))?;
                config.notify_radius_mi = value.parse::<f64>()?.max(0.1);
            }
            "--overpass-mi" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--overpass-mi needs a value"))?;
                config.overpass_mi = value.parse::<f64>()?.max(0.05);
            }
            "--notify-cooldown" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--notify-cooldown needs a value"))?;
                config.notify_cooldown_secs = value.parse::<u64>()?.max(10);
            }
            "--column-cache" => {
                config.column_cache = true;
            }
            "--no-column-cache" => {
                config.column_cache = false;
            }
            "--track-arrows" => {
                config.track_arrows = true;
            }
            "--no-track-arrows" => {
                config.track_arrows = false;
            }
            "--stats-metric-1" => {
                config.stats_metric_1 = iter
                    .next()
                    .ok_or_else(|| anyhow!("--stats-metric-1 needs a value"))?
                    .to_string();
            }
            "--stats-metric-2" => {
                config.stats_metric_2 = iter
                    .next()
                    .ok_or_else(|| anyhow!("--stats-metric-2 needs a value"))?
                    .to_string();
            }
            "--stats-metric-3" => {
                config.stats_metric_3 = iter
                    .next()
                    .ok_or_else(|| anyhow!("--stats-metric-3 needs a value"))?
                    .to_string();
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(anyhow!("Unknown argument: {other}"));
            }
        }
    }

    validate_security(&config)?;
    Ok(config)
}

fn load_file_config(path: &Path) -> Result<Option<FileConfig>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let cfg: FileConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;
    Ok(Some(cfg))
}

fn apply_file_config(target: &mut Config, file: FileConfig) {
    if let Some(url) = file.url {
        target.url = url;
    }
    if let Some(urls) = file.urls {
        target.urls = urls;
    }
    if let Some(refresh) = file.refresh_secs {
        target.refresh = Duration::from_secs(refresh);
    }
    if let Some(insecure) = file.insecure {
        target.insecure = insecure;
    }
    if let Some(allow_http) = file.allow_http {
        target.allow_http = allow_http;
    }
    if let Some(allow_insecure) = file.allow_insecure {
        target.allow_insecure = allow_insecure;
    }
    if let Some(stale_secs) = file.stale_secs {
        target.stale_secs = stale_secs.max(1);
    }
    if let Some(hide_stale) = file.hide_stale {
        target.hide_stale = hide_stale;
    }
    if let Some(low_nic) = file.low_nic {
        target.low_nic = low_nic;
    }
    if let Some(low_nac) = file.low_nac {
        target.low_nac = low_nac;
    }
    if let Some(trail_len) = file.trail_len {
        target.trail_len = trail_len.max(1);
    }
    if let Some(favorites) = file.favorites {
        target.favorites = favorites;
    }
    if let Some(favorites_file) = file.favorites_file {
        target.favorites_file = favorites_file;
    }
    if let Some(api_key) = file.api_key {
        target.api_key = api_key;
    }
    if let Some(api_key_header) = file.api_key_header {
        target.api_key_header = api_key_header;
    }
    if let Some(log_enabled) = file.log_enabled {
        target.log_enabled = log_enabled;
    }
    if let Some(log_level) = file.log_level {
        target.log_level = log_level;
    }
    if let Some(log_file) = file.log_file {
        target.log_file = log_file;
    }
    if let Some(watchlist_enabled) = file.watchlist_enabled {
        target.watchlist_enabled = watchlist_enabled;
    }
    if let Some(watchlist_file) = file.watchlist_file {
        target.watchlist_file = watchlist_file;
    }
    if let Some(filter) = file.filter {
        target.filter = filter;
    }
    if let Some(layout) = file.layout {
        target.layout = layout;
    }
    if let Some(theme) = file.theme {
        target.theme = theme;
    }
    if let Some(radar_range_nm) = file.radar_range_nm {
        target.radar_range_nm = radar_range_nm.max(1.0);
    }
    if let Some(radar_aspect) = file.radar_aspect {
        target.radar_aspect = radar_aspect.max(0.2);
    }
    if let Some(radar_renderer) = file.radar_renderer {
        target.radar_renderer = radar_renderer;
    }
    if let Some(radar_labels) = file.radar_labels {
        target.radar_labels = radar_labels;
    }
    if let Some(radar_blip) = file.radar_blip {
        target.radar_blip = radar_blip;
    }
    if let Some(site_lat) = file.site_lat {
        target.site_lat = Some(site_lat);
    }
    if let Some(site_lon) = file.site_lon {
        target.site_lon = Some(site_lon);
    }
    if let Some(site_alt_m) = file.site_alt_m {
        target.site_alt_m = Some(site_alt_m);
    }
    if let Some(route_enabled) = file.route_enabled {
        target.route_enabled = route_enabled;
    }
    if let Some(route_base) = file.route_base {
        target.route_base = route_base;
    }
    if let Some(route_ttl_secs) = file.route_ttl_secs {
        target.route_ttl_secs = route_ttl_secs;
    }
    if let Some(route_refresh_secs) = file.route_refresh_secs {
        target.route_refresh_secs = route_refresh_secs;
    }
    if let Some(route_batch) = file.route_batch {
        target.route_batch = route_batch.max(1);
    }
    if let Some(route_timeout_secs) = file.route_timeout_secs {
        target.route_timeout_secs = route_timeout_secs.max(2);
    }
    if let Some(route_mode) = file.route_mode {
        target.route_mode = route_mode;
    }
    if let Some(route_path) = file.route_path {
        target.route_path = route_path;
    }
    if let Some(ui_fps) = file.ui_fps {
        target.ui_fps = ui_fps;
    }
    if let Some(smooth_mode) = file.smooth_mode {
        target.smooth_mode = smooth_mode;
    }
    if let Some(smooth_merge) = file.smooth_merge {
        target.smooth_merge = smooth_merge;
    }
    if let Some(rate_window_ms) = file.rate_window_ms {
        target.rate_window_ms = rate_window_ms.max(50);
    }
    if let Some(rate_min_secs) = file.rate_min_secs {
        target.rate_min_secs = rate_min_secs.max(0.05);
    }
    if let Some(notify_radius_mi) = file.notify_radius_mi {
        target.notify_radius_mi = notify_radius_mi.max(0.1);
    }
    if let Some(overpass_mi) = file.overpass_mi {
        target.overpass_mi = overpass_mi.max(0.05);
    }
    if let Some(notify_cooldown_secs) = file.notify_cooldown_secs {
        target.notify_cooldown_secs = notify_cooldown_secs.max(10);
    }
    if let Some(altitude_trend_arrows) = file.altitude_trend_arrows {
        target.altitude_trend_arrows = altitude_trend_arrows;
    }
    if let Some(column_cache) = file.column_cache {
        target.column_cache = column_cache;
    }
    if let Some(track_arrows) = file.track_arrows {
        target.track_arrows = track_arrows;
    }
    if let Some(flags_enabled) = file.flags_enabled {
        target.flags_enabled = flags_enabled;
    }
    if let Some(flag_style) = file.flag_style {
        target.flag_style = flag_style;
    }
    if let Some(demo_mode) = file.demo_mode {
        target.demo_mode = demo_mode;
    }
    if let Some(stats_metric_1) = file.stats_metric_1 {
        target.stats_metric_1 = stats_metric_1;
    }
    if let Some(stats_metric_2) = file.stats_metric_2 {
        target.stats_metric_2 = stats_metric_2;
    }
    if let Some(stats_metric_3) = file.stats_metric_3 {
        target.stats_metric_3 = stats_metric_3;
    }
    if let Some(role_enabled) = file.role_enabled {
        target.role_enabled = role_enabled;
    }
    if let Some(role_highlight) = file.role_highlight {
        target.role_highlight = role_highlight;
    }
}

fn print_help() {
    println!("adsb-tui");
    println!("Usage: adsb-tui [--url URL] [--refresh SECONDS] [--insecure]");
    println!("       [--allow-http] [--allow-insecure]");
    println!("       [--filter TEXT] [--favorite HEX] [--favorites-file PATH] [--config PATH]");
    println!("       [--api-key KEY] [--api-key-header NAME]");
    println!("       [--watchlist] [--no-watchlist] [--watchlist-file PATH]");
    println!("       [--log] [--no-log] [--log-level LEVEL] [--log-file PATH]");
    println!("       [--stale SECONDS] [--hide-stale] [--show-stale] [--low-nic N] [--low-nac N]");
    println!(
        "       [--trail N] [--layout full|compact|radar] [--theme default|color|amber|ocean|matrix|mono]"
    );
    println!("       [--demo-mode] [--no-demo-mode]");
    println!("       [--radar-range-nm NM] [--radar-aspect RATIO] [--radar-renderer canvas|ascii]");
    println!("       [--radar-blip dot|block|plane]");
    println!("       [--radar-labels] [--no-radar-labels]");
    println!("       [--site-lat LAT] [--site-lon LON] [--site-alt-m METERS]");
    println!("       [--route-base URL] [--route-ttl SECS] [--route-refresh SECS]");
    println!("       [--route-batch N] [--route-timeout SECS] [--route-disable]");
    println!("       [--route-mode tar1090|routeset] [--route-path PATH]");
    println!("       [--ui-fps FPS] [--smooth] [--no-smooth] [--smooth-merge] [--no-smooth-merge]");
    println!("       [--rate-window-ms MS] [--rate-min-secs SECS]");
    println!("       [--notify-mi MILES] [--overpass-mi MILES] [--notify-cooldown SECS]");
    println!("       [--column-cache] [--no-column-cache]");
    println!("       [--track-arrows] [--no-track-arrows]");
    println!("       [--flag-style emoji|text|none]");
    println!("       [--alt-arrows] [--no-alt-arrows]");
    println!("       [--stats-metric-1 NAME] [--stats-metric-2 NAME] [--stats-metric-3 NAME]");
    println!("Environment: ADSB_URL overrides the primary URL");
    println!("Environment: ADSB_URLS sets comma-separated fallback URLs");
    println!("Environment: ADSB_INSECURE=1 enables invalid TLS certs");
    println!("Environment: ADSB_ALLOW_HTTP=1 allows http:// URLs");
    println!("Environment: ADSB_ALLOW_INSECURE=1 allows --insecure");
    println!("Environment: ADSB_CONFIG overrides config path");
    println!("Environment: ADSB_TRAIL_LEN sets radar trail length");
    println!("Environment: ADSB_HIDE_STALE filters stale aircraft from the table");
    println!("Environment: ADSB_FAVORITES_FILE sets favorites path");
    println!("Environment: ADSB_API_KEY/ADSB_API_KEY_HEADER configure API auth header");
    println!("Environment: ADSB_WATCHLIST_ENABLED/FILE configure watchlist loading");
    println!("Environment: ADSB_LOG_ENABLED/LEVEL/FILE configure logging");
    println!("Environment: ADSB_SITE_LAT/LON/ALT_M set receiver location");
    println!("Environment: ADSB_ROUTE_* configure route lookups");
    println!("Environment: ADSB_UI_FPS ADSB_SMOOTH ADSB_SMOOTH_MERGE control smoothing");
    println!("Environment: ADSB_RATE_WINDOW_MS ADSB_RATE_MIN_SECS control msg rate smoothing");
    println!("Environment: ADSB_NOTIFY_MI ADSB_OVERPASS_MI ADSB_NOTIFY_COOLDOWN control proximity alerts");
    println!("Environment: ADSB_RADAR_RANGE_NM/ASPECT/RENDERER/BLIP control radar display");
    println!("Environment: ADSB_RADAR_LABELS toggles radar blip labels");
    println!("Environment: ADSB_ALT_TREND toggles altitude trend arrows");
    println!("Environment: ADSB_COLUMN_CACHE toggles column width cache");
    println!("Environment: ADSB_TRACK_ARROWS toggles track direction arrows");
    println!("Environment: ADSB_FLAG_STYLE sets flag rendering mode");
    println!("Environment: ADSB_DEMO_MODE toggles demo mode");
    println!("Environment: ADSB_STATS_METRIC_1/2/3 control stats metrics");
    println!("Keys: q quit | up/down move | s sort | / filter | f favorite | m columns | ? help");
    println!("      t theme | l layout | R radar | b labels | e export csv | E export json");
    println!("      C config editor");
}

fn validate_security(config: &Config) -> Result<()> {
    for url in config.urls.iter().chain(std::iter::once(&config.url)) {
        let trimmed = url.trim();
        if trimmed.to_ascii_lowercase().starts_with("http://") && !config.allow_http {
            return Err(anyhow!(
                "Refusing insecure http URL (set allow_http=true or ADSB_ALLOW_HTTP=1 to override)"
            ));
        }
    }
    if config.insecure && !config.allow_insecure {
        return Err(anyhow!(
            "Refusing --insecure without explicit allow_insecure=true or ADSB_ALLOW_INSECURE=1"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        dir.push(format!("adsb-tui-config-test-{suffix}"));
        let _ = fs::create_dir_all(&dir);
        dir.push(name);
        dir
    }

    fn base_config() -> Config {
        Config {
            url: DEFAULT_URL.to_string(),
            urls: vec![DEFAULT_URL.to_string()],
            refresh: Duration::from_secs(DEFAULT_REFRESH_SECS),
            insecure: false,
            allow_http: DEFAULT_ALLOW_HTTP,
            allow_insecure: false,
            config_path: PathBuf::from("adsb-tui.toml"),
            stale_secs: DEFAULT_STALE_SECS,
            hide_stale: DEFAULT_HIDE_STALE,
            low_nic: DEFAULT_LOW_NIC,
            low_nac: DEFAULT_LOW_NAC,
            trail_len: DEFAULT_TRAIL_LEN,
            favorites: Vec::new(),
            favorites_file: DEFAULT_FAVORITES_FILE.to_string(),
            watchlist_enabled: DEFAULT_WATCHLIST_ENABLED,
            watchlist_file: DEFAULT_WATCHLIST_FILE.to_string(),
            api_key: String::new(),
            api_key_header: DEFAULT_API_KEY_HEADER.to_string(),
            log_enabled: false,
            log_level: "info".to_string(),
            log_file: "adsb-tui.log".to_string(),
            filter: String::new(),
            layout: "full".to_string(),
            theme: "default".to_string(),
            radar_range_nm: DEFAULT_RADAR_RANGE_NM,
            radar_aspect: DEFAULT_RADAR_ASPECT,
            radar_renderer: DEFAULT_RADAR_RENDERER.to_string(),
            radar_labels: DEFAULT_RADAR_LABELS,
            radar_blip: DEFAULT_RADAR_BLIP.to_string(),
            site_lat: None,
            site_lon: None,
            site_alt_m: None,
            route_enabled: true,
            route_base: DEFAULT_ROUTE_BASE.to_string(),
            route_ttl_secs: DEFAULT_ROUTE_TTL_SECS,
            route_refresh_secs: DEFAULT_ROUTE_REFRESH_SECS,
            route_batch: DEFAULT_ROUTE_BATCH,
            route_timeout_secs: DEFAULT_ROUTE_TIMEOUT_SECS,
            route_mode: DEFAULT_ROUTE_MODE.to_string(),
            route_path: DEFAULT_ROUTE_PATH.to_string(),
            ui_fps: DEFAULT_UI_FPS,
            smooth_mode: DEFAULT_SMOOTH_MODE,
            smooth_merge: DEFAULT_SMOOTH_MERGE,
            rate_window_ms: DEFAULT_RATE_WINDOW_MS,
            rate_min_secs: DEFAULT_RATE_MIN_SECS,
            notify_radius_mi: DEFAULT_NOTIFY_RADIUS_MI,
            overpass_mi: DEFAULT_OVERPASS_MI,
            notify_cooldown_secs: DEFAULT_NOTIFY_COOLDOWN_SECS,
            altitude_trend_arrows: DEFAULT_ALTITUDE_TREND_ARROWS,
            column_cache: DEFAULT_COLUMN_CACHE,
            track_arrows: DEFAULT_TRACK_ARROWS,
            flags_enabled: DEFAULT_FLAGS_ENABLED,
            flag_style: DEFAULT_FLAG_STYLE.to_string(),
            demo_mode: DEFAULT_DEMO_MODE,
            stats_metric_1: DEFAULT_STATS_METRIC_1.to_string(),
            stats_metric_2: DEFAULT_STATS_METRIC_2.to_string(),
            stats_metric_3: DEFAULT_STATS_METRIC_3.to_string(),
            role_enabled: DEFAULT_ROLE_ENABLED,
            role_highlight: DEFAULT_ROLE_HIGHLIGHT,
        }
    }

    #[test]
    fn default_allows_http_url() {
        let cfg = base_config();
        assert!(validate_security(&cfg).is_ok());
    }

    #[test]
    fn http_url_rejected_when_disabled() {
        let mut cfg = base_config();
        cfg.allow_http = false;
        let err = validate_security(&cfg).unwrap_err();
        assert!(err.to_string().contains("Refusing insecure http URL"));
    }

    #[test]
    fn load_file_config_parses_values() {
        let path = temp_file("config.toml");
        let content = r#"
url = "http://example.test/data.json"
refresh_secs = 3
api_key = "abc123"
api_key_header = "api-auth"
log_enabled = true
log_level = "debug"
log_file = "adsb-tui.log"
watchlist_enabled = false
watchlist_file = "custom-watch.toml"
hide_stale = true
flag_style = "text"
demo_mode = true
radar_range_nm = 250.0
radar_aspect = 1.2
radar_renderer = "ascii"
radar_labels = true
radar_blip = "block"
role_enabled = false
role_highlight = false
"#;
        fs::write(&path, content).unwrap();
        let cfg = load_file_config(&path).unwrap().unwrap();
        assert_eq!(cfg.url.as_deref(), Some("http://example.test/data.json"));
        assert_eq!(cfg.refresh_secs, Some(3));
        assert_eq!(cfg.api_key.as_deref(), Some("abc123"));
        assert_eq!(cfg.api_key_header.as_deref(), Some("api-auth"));
        assert_eq!(cfg.log_enabled, Some(true));
        assert_eq!(cfg.log_level.as_deref(), Some("debug"));
        assert_eq!(cfg.log_file.as_deref(), Some("adsb-tui.log"));
        assert_eq!(cfg.watchlist_enabled, Some(false));
        assert_eq!(cfg.watchlist_file.as_deref(), Some("custom-watch.toml"));
        assert_eq!(cfg.hide_stale, Some(true));
        assert_eq!(cfg.flag_style.as_deref(), Some("text"));
        assert_eq!(cfg.demo_mode, Some(true));
        assert_eq!(cfg.radar_range_nm, Some(250.0));
        assert_eq!(cfg.radar_aspect, Some(1.2));
        assert_eq!(cfg.radar_renderer.as_deref(), Some("ascii"));
        assert_eq!(cfg.radar_labels, Some(true));
        assert_eq!(cfg.radar_blip.as_deref(), Some("block"));
        assert_eq!(cfg.role_enabled, Some(false));
        assert_eq!(cfg.role_highlight, Some(false));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(path.parent().unwrap());
    }

    #[test]
    fn apply_file_config_overrides_and_clamps() {
        let mut cfg = base_config();
        let file = FileConfig {
            route_batch: Some(0),
            rate_window_ms: Some(10),
            notify_cooldown_secs: Some(2),
            api_key: Some("key".to_string()),
            api_key_header: Some("x-api-key".to_string()),
            log_enabled: Some(true),
            log_level: Some("trace".to_string()),
            log_file: Some("trace.log".to_string()),
            watchlist_enabled: Some(false),
            watchlist_file: Some("wl.toml".to_string()),
            hide_stale: Some(true),
            flag_style: Some("none".to_string()),
            demo_mode: Some(true),
            radar_range_nm: Some(300.0),
            radar_aspect: Some(0.1),
            radar_renderer: Some("ascii".to_string()),
            radar_labels: Some(true),
            radar_blip: Some("plane".to_string()),
            role_enabled: Some(false),
            role_highlight: Some(false),
            ..Default::default()
        };
        apply_file_config(&mut cfg, file);
        assert_eq!(cfg.route_batch, 1);
        assert_eq!(cfg.rate_window_ms, 50);
        assert_eq!(cfg.notify_cooldown_secs, 10);
        assert_eq!(cfg.api_key, "key");
        assert_eq!(cfg.api_key_header, "x-api-key");
        assert!(cfg.log_enabled);
        assert_eq!(cfg.log_level, "trace");
        assert_eq!(cfg.log_file, "trace.log");
        assert!(!cfg.watchlist_enabled);
        assert_eq!(cfg.watchlist_file, "wl.toml");
        assert!(cfg.hide_stale);
        assert_eq!(cfg.flag_style, "none");
        assert!(cfg.demo_mode);
        assert_eq!(cfg.radar_range_nm, 300.0);
        assert_eq!(cfg.radar_aspect, 0.2);
        assert_eq!(cfg.radar_renderer, "ascii");
        assert!(cfg.radar_labels);
        assert_eq!(cfg.radar_blip, "plane");
        assert!(!cfg.role_enabled);
        assert!(!cfg.role_highlight);
    }
}
