use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use toml::Value;
use toml_edit::DocumentMut;
use tracing::{debug, info, trace, warn};

use crate::config;
use crate::lookup::{LookupKind, LookupRequest};
use crate::model::{seen_seconds, Aircraft, ApiResponse};
use crate::storage;
use crate::watchlist::WatchEntry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortMode {
    LastSeen,
    Altitude,
    Speed,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::LastSeen => SortMode::Altitude,
            SortMode::Altitude => SortMode::Speed,
            SortMode::Speed => SortMode::LastSeen,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::LastSeen => "SEEN",
            SortMode::Altitude => "ALT",
            SortMode::Speed => "SPD",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filter,
    Columns,
    Help,
    Config,
    Legend,
    Watchlist,
    Lookup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutMode {
    Full,
    Compact,
    Radar,
    Performance,
}

impl LayoutMode {
    pub fn toggle(self) -> Self {
        match self {
            LayoutMode::Full => LayoutMode::Compact,
            LayoutMode::Compact => LayoutMode::Full,
            LayoutMode::Radar => LayoutMode::Full,
            LayoutMode::Performance => LayoutMode::Full,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LayoutMode::Full => "FULL",
            LayoutMode::Compact => "COMPACT",
            LayoutMode::Radar => "RADAR",
            LayoutMode::Performance => "PERF",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "compact" => LayoutMode::Compact,
            "radar" => LayoutMode::Radar,
            "perf" | "performance" | "graph" => LayoutMode::Performance,
            _ => LayoutMode::Full,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadarRenderer {
    Canvas,
    Ascii,
}

impl RadarRenderer {
    pub fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "ascii" => RadarRenderer::Ascii,
            _ => RadarRenderer::Canvas,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadarBlip {
    Dot,
    Block,
    Plane,
}

impl RadarBlip {
    pub fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "plane" | "airplane" | "aircraft" => RadarBlip::Plane,
            "block" | "cube" | "solid" => RadarBlip::Block,
            _ => RadarBlip::Dot,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Default,
    ColorBlind,
    Amber,
    Ocean,
    Matrix,
    Monochrome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagStyle {
    Emoji,
    Text,
    None,
}

impl FlagStyle {
    pub fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "text" | "code" => FlagStyle::Text,
            "none" | "off" => FlagStyle::None,
            _ => FlagStyle::Emoji,
        }
    }
}

impl ThemeMode {
    pub fn toggle(self) -> Self {
        match self {
            ThemeMode::Default => ThemeMode::ColorBlind,
            ThemeMode::ColorBlind => ThemeMode::Amber,
            ThemeMode::Amber => ThemeMode::Ocean,
            ThemeMode::Ocean => ThemeMode::Matrix,
            ThemeMode::Matrix => ThemeMode::Monochrome,
            ThemeMode::Monochrome => ThemeMode::Default,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Default => "DEFAULT",
            ThemeMode::ColorBlind => "COLOR",
            ThemeMode::Amber => "AMBER",
            ThemeMode::Ocean => "OCEAN",
            ThemeMode::Matrix => "MATRIX",
            ThemeMode::Monochrome => "MONO",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "color" | "colorblind" | "cb" => ThemeMode::ColorBlind,
            "amber" | "gold" => ThemeMode::Amber,
            "ocean" | "blue" => ThemeMode::Ocean,
            "matrix" | "green" => ThemeMode::Matrix,
            "mono" | "monochrome" | "bw" | "grayscale" => ThemeMode::Monochrome,
            _ => ThemeMode::Default,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrendDir {
    Up,
    Down,
    Flat,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AircraftRole {
    Military,
    Government,
    Commercial,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trend {
    pub alt: TrendDir,
    pub gs: TrendDir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnId {
    Fav,
    Watch,
    Flight,
    Reg,
    Type,
    Route,
    Alt,
    Gs,
    Trk,
    Lat,
    Lon,
    Dist,
    Brg,
    Seen,
    Msgs,
    Hex,
    Flag,
}

#[derive(Clone, Debug)]
pub struct ColumnConfig {
    pub id: ColumnId,
    pub label: &'static str,
    pub width: u16,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct TrailPoint {
    pub lat: f64,
    pub lon: f64,
    pub at: SystemTime,
}

#[derive(Clone, Copy, Debug)]
pub struct SiteLocation {
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub at: SystemTime,
}

#[derive(Clone, Debug)]
pub struct ConfigItem {
    pub key: String,
    pub value: String,
    pub kind: config::ConfigKind,
}

#[derive(Clone, Debug)]
struct ColumnWidthCache {
    width: u16,
    cols: Vec<ColumnId>,
    rows_len: usize,
    at: SystemTime,
    widths: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct RouteInfo {
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub route: Option<String>,
    pub fetched_at: SystemTime,
}

#[derive(Clone, Copy, Debug)]
struct AircraftRate {
    last_messages: u64,
    last_time: SystemTime,
    last_rate_time: SystemTime,
    ema: Option<f64>,
    rate: Option<f64>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Metrics {
    alt_baro: Option<i64>,
    gs: Option<f64>,
}

#[derive(Clone, Copy, Debug)]
pub struct PerformanceSample {
    pub msg_rate: Option<f64>,
    pub flights: usize,
    pub rssi_avg: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct PerformanceSnapshot {
    pub msg_rate: Vec<u64>,
    pub flights: Vec<u64>,
    pub signal: Vec<u64>,
    pub latest_msg_rate: Option<f64>,
    pub latest_flights: usize,
    pub latest_signal: Option<f64>,
    pub latest_signal_rsi: Option<f64>,
}

pub struct App {
    pub(crate) url: String,
    pub(crate) refresh: Duration,
    pub(crate) data: ApiResponse,
    raw_data: ApiResponse,
    pub(crate) last_update: Option<SystemTime>,
    pub(crate) last_error: Option<String>,
    pub(crate) sort: SortMode,
    pub(crate) table_state: TableState,
    pub(crate) table_area: Option<Rect>,
    pub(crate) table_header_rows: u16,
    pub(crate) tick: u64,
    pub(crate) start_time: SystemTime,
    pub(crate) stale_secs: f64,
    pub(crate) hide_stale: bool,
    pub(crate) low_nic: i64,
    pub(crate) low_nac: i64,
    pub(crate) favorites: HashSet<String>,
    pub(crate) favorites_path: Option<PathBuf>,
    pub(crate) watchlist_enabled: bool,
    pub(crate) watchlist_path: Option<PathBuf>,
    pub(crate) watchlist: Vec<WatchEntry>,
    pub(crate) filter: String,
    pub(crate) filter_edit: String,
    pub(crate) input_mode: InputMode,
    pub(crate) layout_mode: LayoutMode,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) role_enabled: bool,
    pub(crate) role_highlight: bool,
    pub(crate) column_cache_enabled: bool,
    pub(crate) column_cache_ttl: Duration,
    column_width_cache: Option<ColumnWidthCache>,
    pub(crate) config_path: PathBuf,
    pub(crate) config_items: Vec<ConfigItem>,
    pub(crate) config_cursor: usize,
    pub(crate) config_edit: String,
    pub(crate) config_editing: bool,
    pub(crate) config_dirty: bool,
    pub(crate) config_status: Option<(String, SystemTime)>,
    pub(crate) watchlist_cursor: usize,
    pub(crate) trail_len: usize,
    pub(crate) site: Option<SiteLocation>,
    pub(crate) demo_mode: bool,
    pub(crate) radar_range_nm: f64,
    pub(crate) radar_aspect: f64,
    pub(crate) radar_renderer: RadarRenderer,
    pub(crate) radar_labels: bool,
    pub(crate) radar_blip: RadarBlip,
    pub(crate) columns: Vec<ColumnConfig>,
    pub(crate) column_cursor: usize,
    pub(crate) smooth_mode: bool,
    pub(crate) smooth_merge: bool,
    pub(crate) ui_interval: Duration,
    pub(crate) last_swap: Option<SystemTime>,
    pub(crate) selection_key: Option<String>,
    pub(crate) route_enabled: bool,
    pub(crate) route_tar1090: bool,
    pub(crate) route_ttl: Duration,
    pub(crate) route_refresh: Duration,
    pub(crate) route_batch: usize,
    pub(crate) altitude_trend_arrows: bool,
    pub(crate) track_arrows: bool,
    pub(crate) stats_metrics: [String; 3],
    #[allow(dead_code)]
    pub(crate) flags_enabled: bool,
    pub(crate) flag_style: FlagStyle,
    pub(crate) route_last_poll: Option<SystemTime>,
    pub(crate) route_cache: HashMap<String, RouteInfo>,
    pub(crate) route_last_request: HashMap<String, SystemTime>,
    pub(crate) route_backoff_until: Option<SystemTime>,
    pub(crate) route_backoff_attempts: u32,
    pub(crate) msg_rate: Option<f64>,
    msg_rate_display: Option<f64>,
    msg_rate_ema: Option<f64>,
    msg_rate_last_display: Option<SystemTime>,
    msg_rate_window: Duration,
    msg_rate_min_secs: f64,
    msg_samples: VecDeque<(SystemTime, u64)>,
    aircraft_rates: HashMap<String, AircraftRate>,
    avg_aircraft_rate: Option<f64>,
    total_aircraft_rate: Option<f64>,
    total_aircraft_rate_ema: Option<f64>,
    total_aircraft_rate_time: Option<SystemTime>,
    pub(crate) notify_radius_mi: f64,
    pub(crate) overpass_mi: f64,
    pub(crate) notify_cooldown: Duration,
    notified_recent: HashMap<String, SystemTime>,
    watch_notified_recent: HashMap<String, SystemTime>,
    pub(crate) notifications: Vec<Notification>,
    pub(crate) last_msg_total: Option<u64>,
    pub(crate) last_msg_time: Option<SystemTime>,
    pub(crate) seen_times: HashMap<String, SystemTime>,
    last_metrics: HashMap<String, Metrics>,
    pub(crate) trend_cache: HashMap<String, Trend>,
    pub(crate) trail_points: HashMap<String, Vec<TrailPoint>>,
    perf_samples: VecDeque<PerformanceSample>,
    perf_max_samples: usize,
    pub(crate) last_export: Option<(String, SystemTime)>,
    pub(crate) route_error: Option<(String, SystemTime)>,
    // Lookup modal state
    pub(crate) lookup_input: String,
    pub(crate) lookup_status: Option<String>,
    pub(crate) lookup_results: Option<Vec<Aircraft>>,
    pub(crate) lookup_busy: bool,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        url: String,
        refresh: Duration,
        stale_secs: f64,
        hide_stale: bool,
        low_nic: i64,
        low_nac: i64,
        favorites: HashSet<String>,
        filter: String,
        layout_mode: LayoutMode,
        theme_mode: ThemeMode,
        role_enabled: bool,
        role_highlight: bool,
        column_cache_enabled: bool,
        column_cache_ttl: Duration,
        config_path: PathBuf,
        trail_len: usize,
        favorites_path: Option<PathBuf>,
        site: Option<SiteLocation>,
        demo_mode: bool,
        radar_range_nm: f64,
        radar_aspect: f64,
        radar_renderer: RadarRenderer,
        radar_labels: bool,
        radar_blip: RadarBlip,
        route_enabled: bool,
        route_tar1090: bool,
        route_ttl: Duration,
        route_refresh: Duration,
        route_batch: usize,
        ui_fps: u64,
        smooth_mode: bool,
        smooth_merge: bool,
        rate_window: Duration,
        rate_min_secs: f64,
        notify_radius_mi: f64,
        overpass_mi: f64,
        notify_cooldown: Duration,
        altitude_trend_arrows: bool,
        track_arrows: bool,
        flags_enabled: bool,
        flag_style: FlagStyle,
        stats_metric_1: String,
        stats_metric_2: String,
        stats_metric_3: String,
        watchlist_enabled: bool,
        watchlist_path: Option<PathBuf>,
        watchlist: Vec<WatchEntry>,
    ) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let ui_interval = if ui_fps == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_millis((1000.0 / ui_fps.max(1) as f64) as u64)
        };
        let perf_max_samples = {
            let refresh_secs = refresh.as_secs_f64().max(0.1);
            ((300.0 / refresh_secs) as usize).clamp(60, 300)
        };
        Self {
            url,
            refresh,
            data: ApiResponse::default(),
            raw_data: ApiResponse::default(),
            last_update: None,
            last_error: None,
            sort: SortMode::LastSeen,
            table_state,
            table_area: None,
            table_header_rows: 1,
            tick: 0,
            start_time: SystemTime::now(),
            stale_secs,
            hide_stale,
            low_nic,
            low_nac,
            favorites,
            favorites_path,
            watchlist_enabled,
            watchlist_path,
            watchlist,
            filter,
            filter_edit: String::new(),
            input_mode: InputMode::Normal,
            layout_mode,
            theme_mode,
            role_enabled,
            role_highlight,
            column_cache_enabled,
            column_cache_ttl: if column_cache_ttl.is_zero() {
                Duration::from_millis(400)
            } else {
                column_cache_ttl
            },
            column_width_cache: None,
            config_path,
            config_items: Vec::new(),
            config_cursor: 0,
            config_edit: String::new(),
            config_editing: false,
            config_dirty: false,
            config_status: None,
            watchlist_cursor: 0,
            trail_len: trail_len.max(1),
            site,
            demo_mode,
            radar_range_nm: radar_range_nm.max(1.0),
            radar_aspect: radar_aspect.max(0.2),
            radar_renderer,
            radar_labels,
            radar_blip,
            columns: {
                let mut cols = default_columns();
                if let Some(flag_col) = cols.iter_mut().find(|c| c.id == ColumnId::Flag) {
                    flag_col.visible = flags_enabled;
                }
                if let Some(watch_col) = cols.iter_mut().find(|c| c.id == ColumnId::Watch) {
                    watch_col.visible = watchlist_enabled;
                }
                cols
            },
            column_cursor: 0,
            smooth_mode,
            smooth_merge,
            ui_interval,
            last_swap: None,
            selection_key: None,
            route_enabled,
            route_tar1090,
            route_ttl,
            route_refresh,
            route_batch: route_batch.max(1),
            altitude_trend_arrows,
            track_arrows,
            stats_metrics: [stats_metric_1, stats_metric_2, stats_metric_3],
            flags_enabled,
            flag_style,
            route_last_poll: None,
            route_cache: HashMap::new(),
            route_last_request: HashMap::new(),
            route_backoff_until: None,
            route_backoff_attempts: 0,
            msg_rate: None,
            msg_rate_display: None,
            msg_rate_ema: None,
            msg_rate_last_display: None,
            msg_rate_window: if rate_window.is_zero() {
                Duration::from_millis(300)
            } else {
                rate_window
            },
            msg_rate_min_secs: rate_min_secs.max(0.05),
            msg_samples: VecDeque::new(),
            aircraft_rates: HashMap::new(),
            avg_aircraft_rate: None,
            total_aircraft_rate: None,
            total_aircraft_rate_ema: None,
            total_aircraft_rate_time: None,
            notify_radius_mi: notify_radius_mi.max(0.1),
            overpass_mi: overpass_mi.max(0.05),
            notify_cooldown: if notify_cooldown.is_zero() {
                Duration::from_secs(120)
            } else {
                notify_cooldown
            },
            notified_recent: HashMap::new(),
            watch_notified_recent: HashMap::new(),
            notifications: Vec::new(),
            last_msg_total: None,
            last_msg_time: None,
            seen_times: HashMap::new(),
            last_metrics: HashMap::new(),
            trend_cache: HashMap::new(),
            trail_points: HashMap::new(),
            perf_samples: VecDeque::new(),
            perf_max_samples,
            last_export: None,
            route_error: None,
            lookup_input: String::new(),
            lookup_status: None,
            lookup_results: None,
            lookup_busy: false,
        }
    }

    pub fn apply_update(&mut self, data: ApiResponse) {
        debug!(
            "apply_update aircraft={} messages={:?}",
            data.aircraft.len(),
            data.messages
        );
        let now_time = data
            .now
            .and_then(|n| {
                let mut value = n;
                if value > 4_000_000_000 {
                    value /= 1000;
                }
                if value > 0 {
                    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(value as u64))
                } else {
                    None
                }
            })
            .unwrap_or_else(SystemTime::now);
        self.update_rate(&data, now_time);
        self.update_aircraft_rates(&data, now_time);
        self.update_performance_samples(&data, now_time);
        self.update_seen_times(&data, now_time);
        self.update_trends(&data);
        self.update_trails(&data, now_time);
        self.update_notifications(&data, now_time);
        self.update_watchlist_notifications(&data, now_time);

        self.raw_data = data;
        if !self.smooth_mode {
            self.swap_snapshot();
        }
        self.last_update = Some(now_time);
        self.last_error = None;
    }

    pub fn apply_error(&mut self, msg: String) {
        warn!("apply_error: {msg}");
        self.last_error = Some(msg);
    }

    pub fn advance_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn maybe_swap_snapshot(&mut self, now: SystemTime) {
        if !self.smooth_mode {
            return;
        }
        if self.ui_interval.as_secs() == 0 && self.ui_interval.subsec_nanos() == 0 {
            self.swap_snapshot();
            self.last_swap = Some(now);
            return;
        }
        match self.last_swap {
            Some(last) => {
                if now
                    .duration_since(last)
                    .map(|d| d >= self.ui_interval)
                    .unwrap_or(false)
                {
                    self.swap_snapshot();
                    self.last_swap = Some(now);
                }
            }
            None => {
                self.swap_snapshot();
                self.last_swap = Some(now);
            }
        }
    }

    pub fn toggle_sort(&mut self) {
        self.sort = self.sort.next();
        debug!("sort mode -> {}", self.sort.label());
    }

    pub fn toggle_theme(&mut self) {
        self.theme_mode = self.theme_mode.toggle();
        debug!("theme -> {}", self.theme_mode.label());
    }

    pub fn toggle_layout(&mut self) {
        self.layout_mode = self.layout_mode.toggle();
        debug!("layout -> {}", self.layout_mode.label());
    }

    pub fn set_layout(&mut self, layout_mode: LayoutMode) {
        if self.layout_mode != layout_mode {
            self.layout_mode = layout_mode;
            debug!("layout -> {}", self.layout_mode.label());
        }
    }

    pub fn toggle_radar_labels(&mut self) {
        self.radar_labels = !self.radar_labels;
        debug!(
            "radar labels -> {}",
            if self.radar_labels { "on" } else { "off" }
        );
    }

    pub fn open_columns(&mut self) {
        self.input_mode = InputMode::Columns;
        debug!("open columns");
    }

    pub fn close_columns(&mut self) {
        self.input_mode = InputMode::Normal;
        debug!("close columns");
    }

    pub fn open_help(&mut self) {
        self.input_mode = InputMode::Help;
        debug!("open help");
    }

    pub fn close_help(&mut self) {
        self.input_mode = InputMode::Normal;
        debug!("close help");
    }

    pub fn open_legend(&mut self) {
        self.config_cursor = 0;
        self.input_mode = InputMode::Legend;
        debug!("open legend");
    }

    pub fn close_legend(&mut self) {
        self.input_mode = InputMode::Help;
        debug!("close legend");
    }

    pub fn open_watchlist(&mut self) {
        self.watchlist_cursor = 0;
        self.input_mode = InputMode::Watchlist;
        debug!("open watchlist");
    }

    pub fn close_watchlist(&mut self) {
        self.input_mode = InputMode::Normal;
        debug!("close watchlist");
    }

    pub fn save_watchlist(&mut self) {
        let now = SystemTime::now();
        let Some(path) = self.watchlist_path.as_ref() else {
            warn!("watchlist save skipped: no file path");
            self.notifications.push(Notification {
                message: "WATCHLIST no file path".to_string(),
                at: now,
            });
            return;
        };
        match storage::save_watchlist(path, &self.watchlist) {
            Ok(_) => {
                info!("watchlist saved {}", path.display());
                self.notifications.push(Notification {
                    message: "WATCHLIST saved".to_string(),
                    at: now,
                });
            }
            Err(err) => {
                warn!("watchlist save failed: {err}");
                self.notifications.push(Notification {
                    message: format!("WATCHLIST ERR {err}"),
                    at: now,
                });
            }
        }
    }

    pub fn add_watchlist_from_selected(&mut self, indices: &[usize]) -> bool {
        if !self.watchlist_enabled {
            self.watchlist_enabled = true;
        }
        if let Some(path) = self.watchlist_path.as_ref() {
            if let Ok(created) = storage::ensure_watchlist_file(path) {
                if created {
                    self.notifications.push(Notification {
                        message: format!("WATCHLIST template created {}", path.display()),
                        at: SystemTime::now(),
                    });
                }
            }
        }
        let Some(selected) = self.table_state.selected() else {
            return false;
        };
        let Some(idx) = indices.get(selected) else {
            return false;
        };
        let Some(ac) = self.data.aircraft.get(*idx) else {
            return false;
        };
        let Some(entry) = make_watchlist_entry(ac) else {
            return false;
        };
        let entry_key = watchlist_entry_key(&entry);
        if self
            .watchlist
            .iter()
            .any(|existing| watchlist_entry_key(existing) == entry_key)
        {
            self.notifications.push(Notification {
                message: "WATCHLIST already exists".to_string(),
                at: SystemTime::now(),
            });
            return false;
        }
        self.watchlist.push(entry.clone());
        self.watchlist_cursor = self.watchlist.len().saturating_sub(1);
        self.save_watchlist();
        let entry_id = entry.entry_id();
        let label = entry
            .label
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(entry_id.as_str());
        self.notifications.push(Notification {
            message: format!("WATCHLIST added {label}"),
            at: SystemTime::now(),
        });
        true
    }

    pub fn toggle_watchlist_enabled_selected(&mut self) -> bool {
        if let Some(entry) = self.watchlist.get_mut(self.watchlist_cursor) {
            let entry_id = entry.entry_id();
            let next = !entry.is_enabled();
            entry.enabled = Some(next);
            self.save_watchlist();
            debug!("watchlist {} enabled={}", entry_id, next);
            return true;
        }
        false
    }

    pub fn toggle_watchlist_notify_selected(&mut self) -> bool {
        if let Some(entry) = self.watchlist.get_mut(self.watchlist_cursor) {
            let entry_id = entry.entry_id();
            let next = !entry.notify_enabled();
            entry.notify = Some(next);
            self.save_watchlist();
            debug!("watchlist {} notify={}", entry_id, next);
            return true;
        }
        false
    }

    pub fn delete_watchlist_selected(&mut self) -> bool {
        if self.watchlist.is_empty() {
            return false;
        }
        let entry_id = self
            .watchlist
            .get(self.watchlist_cursor)
            .map(|entry| entry.entry_id());
        if self.watchlist_cursor >= self.watchlist.len() {
            self.watchlist_cursor = self.watchlist.len() - 1;
        }
        self.watchlist.remove(self.watchlist_cursor);
        if self.watchlist_cursor >= self.watchlist.len() && !self.watchlist.is_empty() {
            self.watchlist_cursor = self.watchlist.len() - 1;
        }
        self.save_watchlist();
        if let Some(entry_id) = entry_id {
            debug!("watchlist delete {}", entry_id);
        }
        true
    }

    pub fn watchlist_len(&self) -> usize {
        self.watchlist.len()
    }

    pub fn next_watchlist_item(&mut self) {
        if self.watchlist.is_empty() {
            return;
        }
        self.watchlist_cursor = (self.watchlist_cursor + 1) % self.watchlist.len();
    }

    pub fn previous_watchlist_item(&mut self) {
        if self.watchlist.is_empty() {
            return;
        }
        if self.watchlist_cursor == 0 {
            self.watchlist_cursor = self.watchlist.len() - 1;
        } else {
            self.watchlist_cursor -= 1;
        }
    }

    pub fn watchlist_page_up(&mut self, window: usize) {
        if self.watchlist.is_empty() || window == 0 {
            return;
        }
        self.watchlist_cursor = self.watchlist_cursor.saturating_sub(window);
    }

    pub fn watchlist_page_down(&mut self, window: usize) {
        if self.watchlist.is_empty() || window == 0 {
            return;
        }
        let next = self.watchlist_cursor.saturating_add(window);
        self.watchlist_cursor = next.min(self.watchlist.len().saturating_sub(1));
    }

    pub fn open_config(&mut self) {
        let config_exists = self.config_path.exists();
        self.config_items = load_config_items(&self.config_path);
        self.config_cursor = 0;
        self.config_editing = false;
        self.config_edit.clear();
        self.config_dirty = !config_exists;
        self.config_status = None;
        self.input_mode = InputMode::Config;
        debug!("open config items={}", self.config_items.len());
    }

    pub fn close_config(&mut self) {
        if self.config_dirty && !self.save_config() {
            return;
        }
        self.input_mode = InputMode::Normal;
        self.config_editing = false;
        self.config_edit.clear();
        debug!("close config");
    }

    pub fn next_config_item(&mut self) {
        if self.config_items.is_empty() {
            return;
        }
        self.config_cursor = (self.config_cursor + 1) % self.config_items.len();
    }

    pub fn previous_config_item(&mut self) {
        if self.config_items.is_empty() {
            return;
        }
        if self.config_cursor == 0 {
            self.config_cursor = self.config_items.len() - 1;
        } else {
            self.config_cursor -= 1;
        }
    }

    pub fn start_config_edit(&mut self) {
        if let Some(item) = self.config_items.get(self.config_cursor) {
            self.config_edit = item.value.clone();
            self.config_editing = true;
            debug!("config edit start key={}", item.key);
        }
    }

    pub fn cancel_config_edit(&mut self) {
        self.config_editing = false;
        self.config_edit.clear();
        debug!("config edit cancel");
    }

    pub fn apply_config_edit(&mut self) {
        if let Some(item) = self.config_items.get_mut(self.config_cursor) {
            let next = self.config_edit.trim().to_string();
            if item.value != next {
                item.value = next;
                self.config_dirty = true;
            }
        }
        self.config_editing = false;
        self.config_edit.clear();
        debug!("config edit applied");
    }

    pub fn push_config_char(&mut self, ch: char) {
        self.config_edit.push(ch);
    }

    pub fn backspace_config(&mut self) {
        self.config_edit.pop();
    }

    pub fn save_config(&mut self) -> bool {
        if self.config_editing {
            self.apply_config_edit();
        }
        if let Some(parent) = self
            .config_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            if let Err(err) = fs::create_dir_all(parent) {
                warn!("config save failed: {err}");
                self.config_status = Some((format!("save failed: {err}"), SystemTime::now()));
                return false;
            }
        }
        let existing = fs::read_to_string(&self.config_path).unwrap_or_default();
        let mut doc = existing
            .parse::<DocumentMut>()
            .unwrap_or_else(|_| DocumentMut::new());
        let mut api_key_skipped = false;

        for item in &self.config_items {
            if item.key == "api_key" {
                if !item.value.trim().is_empty() {
                    api_key_skipped = true;
                }
                doc.remove(item.key.as_str());
                continue;
            }
            match parse_config_value(item.kind, item.value.trim()) {
                Ok(Some(value)) => {
                    doc[item.key.as_str()] = to_edit_value(value);
                }
                Ok(None) => {
                    doc.remove(item.key.as_str());
                }
                Err(err) => {
                    warn!("config save failed: {err}");
                    self.config_status = Some((err, SystemTime::now()));
                    return false;
                }
            }
        }

        let text = doc.to_string();
        if let Err(err) = fs::write(&self.config_path, text) {
            warn!("config save failed: {err}");
            self.config_status = Some((format!("save failed: {err}"), SystemTime::now()));
            return false;
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&self.config_path, fs::Permissions::from_mode(0o600));
            }
            info!("config saved {}", self.config_path.display());
            let mut message = format!("saved {} (restart to apply)", self.config_path.display());
            if api_key_skipped {
                message.push_str("; api_key not saved");
            }
            self.config_status = Some((message, SystemTime::now()));
            self.config_dirty = false;
        }
        true
    }

    pub fn next_column(&mut self) {
        if self.columns.is_empty() {
            return;
        }
        self.column_cursor = (self.column_cursor + 1) % self.columns.len();
    }

    pub fn previous_column(&mut self) {
        if self.columns.is_empty() {
            return;
        }
        if self.column_cursor == 0 {
            self.column_cursor = self.columns.len() - 1;
        } else {
            self.column_cursor -= 1;
        }
    }

    pub fn next_cursor(&mut self, len: usize) {
        if len == 0 {
            self.config_cursor = 0;
            return;
        }
        self.config_cursor = (self.config_cursor + 1) % len;
    }

    pub fn previous_cursor(&mut self, len: usize) {
        if len == 0 {
            self.config_cursor = 0;
            return;
        }
        if self.config_cursor == 0 {
            self.config_cursor = len - 1;
        } else {
            self.config_cursor -= 1;
        }
    }

    pub fn toggle_column(&mut self) {
        if self.columns.is_empty() {
            return;
        }
        let visible_count = self.columns.iter().filter(|col| col.visible).count();
        if let Some(col) = self.columns.get_mut(self.column_cursor) {
            if col.visible && visible_count <= 1 {
                return;
            }
            col.visible = !col.visible;
            debug!("column {} visible={}", col.label, col.visible);
        }
    }

    pub fn columns(&self) -> &[ColumnConfig] {
        &self.columns
    }

    pub fn column_cursor(&self) -> usize {
        self.column_cursor
    }

    pub fn column_cache_lookup(
        &self,
        width: u16,
        cols: &[ColumnId],
        rows_len: usize,
        now: SystemTime,
    ) -> Option<Vec<u16>> {
        if !self.column_cache_enabled {
            return None;
        }
        let cache = self.column_width_cache.as_ref()?;
        if cache.width != width || cache.rows_len != rows_len || cache.cols != cols {
            return None;
        }
        if now
            .duration_since(cache.at)
            .map(|d| d > self.column_cache_ttl)
            .unwrap_or(true)
        {
            return None;
        }
        Some(cache.widths.clone())
    }

    pub fn column_cache_store(
        &mut self,
        width: u16,
        cols: Vec<ColumnId>,
        rows_len: usize,
        widths: Vec<u16>,
        now: SystemTime,
    ) {
        if !self.column_cache_enabled {
            return;
        }
        self.column_width_cache = Some(ColumnWidthCache {
            width,
            cols,
            rows_len,
            at: now,
            widths,
        });
    }

    pub fn set_table_area(&mut self, area: Rect, header_rows: u16) {
        self.table_area = Some(area);
        self.table_header_rows = header_rows.max(1);
    }

    pub fn table_row_at(&self, y: u16) -> Option<usize> {
        let area = self.table_area?;
        if area.height < 3 {
            return None;
        }
        let top = area.y + 1;
        let data_top = top.saturating_add(self.table_header_rows);
        let data_bottom = area.y + area.height.saturating_sub(1);
        if y < data_top || y >= data_bottom {
            return None;
        }
        Some((y - data_top) as usize)
    }

    pub fn select_row(&mut self, row: usize, visible_len: usize) {
        if visible_len == 0 {
            self.table_state.select(None);
            return;
        }
        let clamped = row.min(visible_len.saturating_sub(1));
        self.table_state.select(Some(clamped));
    }

    pub fn next_row(&mut self, visible_len: usize) {
        if visible_len == 0 {
            return;
        }
        let idx = self.table_state.selected().unwrap_or(0);
        let next = if idx + 1 >= visible_len { 0 } else { idx + 1 };
        self.table_state.select(Some(next));
    }

    pub fn previous_row(&mut self, visible_len: usize) {
        if visible_len == 0 {
            return;
        }
        let idx = self.table_state.selected().unwrap_or(0);
        let prev = if idx == 0 { visible_len - 1 } else { idx - 1 };
        self.table_state.select(Some(prev));
    }

    pub fn clamp_selection_to(&mut self, visible_len: usize) {
        if visible_len == 0 {
            self.table_state.select(None);
        } else if let Some(selected) = self.table_state.selected() {
            if selected >= visible_len {
                self.table_state.select(Some(visible_len - 1));
            }
        } else {
            self.table_state.select(Some(0));
        }
    }

    pub fn restore_selection_by_key(&mut self, indices: &[usize]) {
        if let Some(key) = &self.selection_key {
            if let Some(pos) = indices.iter().position(|idx| {
                if let Some(ac) = self.data.aircraft.get(*idx) {
                    let stale = seen_seconds(ac)
                        .map(|s| s > self.stale_secs)
                        .unwrap_or(false);
                    if stale {
                        return false;
                    }
                    if let Some(hex) = ac.hex.as_deref() {
                        if normalize_hex(hex) == *key {
                            return true;
                        }
                    }
                    if let Some(flight) = ac.flight.as_deref() {
                        if normalize_callsign(flight) == *key {
                            return true;
                        }
                    }
                }
                false
            }) {
                self.table_state.select(Some(pos));
            } else {
                self.selection_key = None;
            }
        }
    }

    pub fn update_selection_key(&mut self, indices: &[usize]) {
        if let Some(selected) = self.table_state.selected() {
            if let Some(idx) = indices.get(selected) {
                if let Some(ac) = self.data.aircraft.get(*idx) {
                    if let Some(hex) = ac.hex.as_deref() {
                        self.selection_key = Some(normalize_hex(hex));
                        return;
                    }
                    if let Some(flight) = ac.flight.as_deref() {
                        self.selection_key = Some(normalize_callsign(flight));
                    }
                }
            }
        }
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .data
            .aircraft
            .iter()
            .enumerate()
            .filter(|(_, ac)| {
                if self.hide_stale {
                    let stale = seen_seconds(ac)
                        .map(|s| s > self.stale_secs)
                        .unwrap_or(false);
                    if stale {
                        return false;
                    }
                }
                self.matches_filter(ac)
            })
            .map(|(i, _)| i)
            .collect();

        indices.sort_by(|&a, &b| {
            let a_ac = &self.data.aircraft[a];
            let b_ac = &self.data.aircraft[b];

            let a_fav = self.is_favorite(a_ac);
            let b_fav = self.is_favorite(b_ac);
            if a_fav != b_fav {
                return b_fav.cmp(&a_fav);
            }

            match self.sort {
                SortMode::LastSeen => {
                    let a_seen = seen_seconds(a_ac);
                    let b_seen = seen_seconds(b_ac);
                    a_seen.partial_cmp(&b_seen).unwrap_or(Ordering::Equal)
                }
                SortMode::Altitude => {
                    let a_alt = a_ac.alt_baro.unwrap_or(-1);
                    let b_alt = b_ac.alt_baro.unwrap_or(-1);
                    b_alt.cmp(&a_alt)
                }
                SortMode::Speed => {
                    let a_spd = a_ac.gs.unwrap_or(-1.0);
                    let b_spd = b_ac.gs.unwrap_or(-1.0);
                    b_spd.partial_cmp(&a_spd).unwrap_or(Ordering::Equal)
                }
            }
        });

        indices
    }

    pub fn is_favorite(&self, ac: &Aircraft) -> bool {
        ac.hex
            .as_deref()
            .map(|hex| self.favorites.contains(&normalize_hex(hex)))
            .unwrap_or(false)
    }

    pub fn toggle_favorite_selected(&mut self, indices: &[usize]) -> bool {
        let selected = self.table_state.selected().and_then(|row| indices.get(row));
        if let Some(idx) = selected {
            let ac = &self.data.aircraft[*idx];
            if let Some(hex) = ac.hex.as_deref() {
                let key = normalize_hex(hex);
                if !self.favorites.insert(key.clone()) {
                    self.favorites.remove(&key);
                }
                debug!("favorite toggle {}", key);
                return true;
            }
        }
        false
    }

    pub fn favorites_path(&self) -> Option<&PathBuf> {
        self.favorites_path.as_ref()
    }

    pub fn watchlist_path(&self) -> Option<&PathBuf> {
        self.watchlist_path.as_ref()
    }

    pub fn is_watchlisted(&self, ac: &Aircraft) -> bool {
        self.watch_entry_for(ac).is_some()
    }

    pub fn watch_entry_for(&self, ac: &Aircraft) -> Option<&WatchEntry> {
        if !self.watchlist_enabled {
            return None;
        }
        let mut best: Option<&WatchEntry> = None;
        let mut best_priority = i64::MIN;
        for entry in &self.watchlist {
            if !entry.is_enabled() {
                continue;
            }
            if !watch_entry_matches(entry, ac, self.route_for(ac)) {
                continue;
            }
            let prio = entry.priority();
            if best.is_none() || prio > best_priority {
                best_priority = prio;
                best = Some(entry);
            }
        }
        best
    }

    pub fn site(&self) -> Option<SiteLocation> {
        if self.demo_mode {
            None
        } else {
            self.site
        }
    }

    pub fn route_for(&self, ac: &Aircraft) -> Option<&RouteInfo> {
        if let Some(callsign) = ac.flight.as_deref() {
            let key = normalize_callsign(callsign);
            if let Some(info) = self.route_cache.get(&key) {
                return Some(info);
            }
        }
        if let Some(hex) = ac.hex.as_deref() {
            let key = normalize_hex(hex);
            if let Some(info) = self.route_cache.get(&key) {
                return Some(info);
            }
        }
        None
    }

    pub fn route_enabled(&self) -> bool {
        self.route_enabled
    }

    pub fn route_tar1090(&self) -> bool {
        self.route_tar1090
    }

    pub fn route_pending(&self, callsign: &str, now: SystemTime) -> bool {
        let key = normalize_callsign(callsign);
        let window = self.route_pending_window();
        self.route_last_request
            .get(&key)
            .and_then(|last| now.duration_since(*last).ok())
            .map(|delta| delta <= window)
            .unwrap_or(false)
    }

    pub fn msg_rate_display(&self) -> Option<f64> {
        let now = SystemTime::now();
        let global_recent = self
            .last_msg_time
            .and_then(|t| now.duration_since(t).ok())
            .map(|d| d <= self.msg_rate_window + self.msg_rate_window)
            .unwrap_or(false);

        if global_recent {
            self.msg_rate
                .or(self.msg_rate_display)
                .or(self.total_aircraft_rate)
        } else {
            self.total_aircraft_rate
                .or(self.msg_rate)
                .or(self.msg_rate_display)
        }
    }

    pub fn avg_aircraft_rate(&self) -> Option<f64> {
        self.avg_aircraft_rate
    }

    pub fn performance_snapshot(&self) -> PerformanceSnapshot {
        let mut msg_rate = Vec::with_capacity(self.perf_samples.len());
        let mut flights = Vec::with_capacity(self.perf_samples.len());
        let mut signal = Vec::with_capacity(self.perf_samples.len());
        let to_rate = |value: Option<f64>| -> u64 { value.unwrap_or(0.0).max(0.0).round() as u64 };
        let to_signal = |value: Option<f64>| -> u64 {
            let min_db = -50.0;
            let max_db = 0.0;
            let clamped = value.unwrap_or(min_db).clamp(min_db, max_db);
            let norm = (clamped - min_db) / (max_db - min_db);
            (norm * 100.0).round() as u64
        };

        for sample in &self.perf_samples {
            msg_rate.push(to_rate(sample.msg_rate));
            flights.push(sample.flights as u64);
            signal.push(to_signal(sample.rssi_avg));
        }

        if msg_rate.is_empty() {
            msg_rate.push(0);
            flights.push(0);
            signal.push(0);
        }

        let (latest_msg_rate, latest_flights, latest_signal) = match self.perf_samples.back() {
            Some(sample) => (sample.msg_rate, sample.flights, sample.rssi_avg),
            None => (None, 0, None),
        };
        let latest_signal_rsi = rsi_from_series(&signal, 14);

        PerformanceSnapshot {
            msg_rate,
            flights,
            signal,
            latest_msg_rate,
            latest_flights,
            latest_signal,
            latest_signal_rsi,
        }
    }

    pub fn classify_aircraft(&self, ac: &Aircraft) -> AircraftRole {
        if !self.role_enabled {
            AircraftRole::Unknown
        } else {
            classify_aircraft(ac)
        }
    }

    pub fn route_refresh_due(&mut self, now: SystemTime) -> bool {
        if let Some(until) = self.route_backoff_until {
            if now.duration_since(until).is_err() {
                return false;
            }
            self.route_backoff_until = None;
        }
        if self.route_refresh.as_secs() == 0 && self.route_refresh.subsec_nanos() == 0 {
            return true;
        }
        match self.route_last_poll {
            Some(last) => now
                .duration_since(last)
                .map(|d| d >= self.route_refresh)
                .unwrap_or(false),
            None => true,
        }
    }

    pub fn mark_route_poll(&mut self, now: SystemTime) {
        self.route_last_poll = Some(now);
    }

    pub fn apply_routes(&mut self, results: Vec<crate::routes::RouteResult>) {
        let now = SystemTime::now();
        for route in results {
            let key = normalize_callsign(route.callsign.as_str());
            self.route_cache.insert(
                key,
                RouteInfo {
                    origin: route.origin,
                    destination: route.destination,
                    route: route.route,
                    fetched_at: now,
                },
            );
        }
        self.route_error = None;
        self.route_backoff_until = None;
        self.route_backoff_attempts = 0;
    }

    pub fn set_route_error(&mut self, message: String) {
        let now = SystemTime::now();
        self.route_error = Some((message.clone(), now));
        self.note_route_failure(&message, now);
    }

    pub fn collect_route_requests(
        &mut self,
        indices: &[usize],
        now: SystemTime,
    ) -> Vec<crate::routes::RouteRequest> {
        if !self.route_enabled {
            return Vec::new();
        }
        let mut requests = Vec::new();
        let batch_limit = self.route_batch.min(10);
        for idx in indices {
            if requests.len() >= batch_limit {
                break;
            }
            let ac = &self.data.aircraft[*idx];
            let callsign = ac.flight.as_deref().unwrap_or("").trim().to_string();
            if callsign.is_empty() {
                continue;
            }
            let key = normalize_callsign(callsign.as_str());
            if !(self.route_ttl.as_secs() == 0 && self.route_ttl.subsec_nanos() == 0) {
                if let Some(info) = self.route_cache.get(&key) {
                    if now
                        .duration_since(info.fetched_at)
                        .map(|d| d < self.route_ttl)
                        .unwrap_or(true)
                    {
                        continue;
                    }
                }
            }
            if !(self.route_refresh.as_secs() == 0 && self.route_refresh.subsec_nanos() == 0) {
                if let Some(last) = self.route_last_request.get(&key) {
                    if now
                        .duration_since(*last)
                        .map(|d| d < self.route_refresh)
                        .unwrap_or(true)
                    {
                        continue;
                    }
                }
            }
            self.route_last_request.insert(key.clone(), now);
            requests.push(crate::routes::RouteRequest {
                callsign: callsign.to_string(),
                lat: ac.lat.unwrap_or(0.0),
                lon: ac.lon.unwrap_or(0.0),
            });
        }
        requests
    }

    fn route_pending_window(&self) -> Duration {
        if self.route_refresh.is_zero() {
            Duration::from_secs(10)
        } else {
            self.route_refresh
                .min(Duration::from_secs(10))
                .max(Duration::from_secs(2))
        }
    }

    fn note_route_failure(&mut self, message: &str, now: SystemTime) {
        if !is_rate_limited_message(message) {
            return;
        }
        self.route_backoff_attempts = self.route_backoff_attempts.saturating_add(1);
        let retry_after = parse_retry_after_hint(message).unwrap_or(0);
        let backoff = route_backoff_duration(self.route_backoff_attempts)
            .max(Duration::from_secs(retry_after));
        self.route_backoff_until = Some(now + backoff);
        debug!(
            "route backoff {}s (attempt {}) due to {}",
            backoff.as_secs(),
            self.route_backoff_attempts,
            message
        );
    }

    pub fn start_filter(&mut self) {
        self.filter_edit = self.filter.clone();
        self.input_mode = InputMode::Filter;
        debug!("filter edit start");
    }

    pub fn apply_filter(&mut self) {
        self.filter = self.filter_edit.trim().to_string();
        self.input_mode = InputMode::Normal;
        debug!("filter applied len={}", self.filter.len());
    }

    pub fn cancel_filter(&mut self) {
        self.filter_edit.clear();
        self.input_mode = InputMode::Normal;
        debug!("filter edit cancel");
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        debug!("filter cleared");
    }

    pub fn push_filter_char(&mut self, ch: char) {
        self.filter_edit.push(ch);
    }

    pub fn backspace_filter(&mut self) {
        self.filter_edit.pop();
    }

    // Lookup modal helpers
    pub fn open_lookup(&mut self) {
        self.lookup_input.clear();
        self.lookup_results = None;
        self.lookup_status =
            Some("Enter: hex/callsign/reg/type/squawk/point/mil/ladd/pia".to_string());
        self.lookup_busy = false;
        self.input_mode = InputMode::Lookup;
        debug!("lookup modal opened");
    }

    pub fn cancel_lookup(&mut self) {
        self.lookup_input.clear();
        self.lookup_busy = false;
        self.input_mode = InputMode::Normal;
        debug!("lookup modal closed");
    }

    pub fn push_lookup_char(&mut self, ch: char) {
        self.lookup_input.push(ch);
    }

    pub fn backspace_lookup(&mut self) {
        self.lookup_input.pop();
    }

    pub fn prepare_lookup_request(&mut self) -> Option<LookupRequest> {
        if self.lookup_busy {
            self.lookup_status = Some("Busy...".to_string());
            return None;
        }
        let trimmed = self.lookup_input.trim();
        if trimmed.is_empty() {
            self.lookup_status = Some("Enter a query".to_string());
            return None;
        }
        match parse_lookup_input(trimmed) {
            Some(kind) => {
                self.lookup_busy = true;
                self.lookup_status = Some("Fetching...".to_string());
                Some(LookupRequest { kind })
            }
            None => {
                self.lookup_status = Some("Unrecognized query".to_string());
                None
            }
        }
    }

    pub fn apply_lookup_result(&mut self, data: ApiResponse) {
        let count = data.aircraft.len();
        self.lookup_results = Some(data.aircraft);
        self.lookup_status = Some(format!("{} result(s)", count));
        self.lookup_busy = false;
    }

    pub fn apply_lookup_error(&mut self, err: String) {
        self.lookup_status = Some(format!("Error: {err}"));
        self.lookup_busy = false;
    }

    pub fn trend_for(&self, ac: &Aircraft) -> Trend {
        if let Some(hex) = ac.hex.as_deref() {
            let key = normalize_hex(hex);
            if let Some(trend) = self.trend_cache.get(&key) {
                return *trend;
            }
        }
        Trend {
            alt: TrendDir::Unknown,
            gs: TrendDir::Unknown,
        }
    }

    pub fn trail_for(&self, ac: &Aircraft) -> Option<&[TrailPoint]> {
        let key = ac.hex.as_deref().map(normalize_hex)?;
        self.trail_points.get(&key).map(|v| v.as_slice())
    }

    pub fn set_last_export(&mut self, filename: String) {
        self.last_export = Some((filename, SystemTime::now()));
    }

    pub fn latest_notification(&self) -> Option<&Notification> {
        self.notifications.last()
    }

    fn update_performance_samples(&mut self, data: &ApiResponse, _now_time: SystemTime) {
        let flights = data.aircraft.len();
        let mut rssi_sum = 0.0;
        let mut rssi_count = 0usize;
        for ac in &data.aircraft {
            if let Some(rssi) = ac.rssi {
                rssi_sum += rssi;
                rssi_count += 1;
            }
        }
        let rssi_avg = if rssi_count > 0 {
            Some(rssi_sum / rssi_count as f64)
        } else {
            None
        };
        let sample = PerformanceSample {
            msg_rate: self.msg_rate_display(),
            flights,
            rssi_avg,
        };
        self.perf_samples.push_back(sample);
        while self.perf_samples.len() > self.perf_max_samples {
            self.perf_samples.pop_front();
        }
    }

    fn update_rate(&mut self, data: &ApiResponse, now_time: SystemTime) {
        if let Some(messages) = data.messages {
            if let Some(last_total) = self.last_msg_total {
                if messages < last_total {
                    self.msg_samples.clear();
                    self.msg_rate_ema = None;
                    self.msg_rate_display = None;
                }
                if messages == last_total {
                    self.apply_rate_decay(now_time);
                    return;
                }
            }

            self.last_msg_total = Some(messages);
            self.last_msg_time = Some(now_time);
            self.msg_samples.push_back((now_time, messages));

            while let Some((t, _)) = self.msg_samples.front() {
                if now_time
                    .duration_since(*t)
                    .map(|d| d > self.msg_rate_window)
                    .unwrap_or(false)
                {
                    self.msg_samples.pop_front();
                } else {
                    break;
                }
            }

            let window_rate = if let (Some((t0, m0)), Some((t1, m1))) =
                (self.msg_samples.front(), self.msg_samples.back())
            {
                if let Ok(delta_t) = t1.duration_since(*t0) {
                    let secs = delta_t.as_secs_f64().max(self.msg_rate_min_secs);
                    let delta_msgs = m1.saturating_sub(*m0) as f64;
                    Some((delta_msgs, secs))
                } else {
                    None
                }
            } else {
                None
            };

            let short_rate = if self.msg_samples.len() >= 2 {
                let last = self.msg_samples.len() - 1;
                if let (Some((t0, m0)), Some((t1, m1))) =
                    (self.msg_samples.get(last - 1), self.msg_samples.get(last))
                {
                    if let Ok(delta_t) = t1.duration_since(*t0) {
                        let secs = delta_t.as_secs_f64().max(self.msg_rate_min_secs * 0.5);
                        let delta_msgs = m1.saturating_sub(*m0) as f64;
                        Some((delta_msgs, secs))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let inst = match (short_rate, window_rate) {
                (Some((dm_s, s_s)), Some((dm_w, s_w))) => {
                    let rate_s = dm_s / s_s;
                    let rate_w = dm_w / s_w;
                    0.7 * rate_s + 0.3 * rate_w
                }
                (Some((dm_s, s_s)), None) => dm_s / s_s,
                (None, Some((dm_w, s_w))) => dm_w / s_w,
                _ => return,
            };

            if inst <= 0.0 {
                self.apply_rate_decay(now_time);
                return;
            }

            let ema = match self.msg_rate_ema {
                Some(prev) => 0.45 * inst + 0.55 * prev,
                None => inst,
            };
            self.msg_rate_ema = Some(ema);
            self.msg_rate = Some(ema);
            self.msg_rate_display = Some(ema);
            self.msg_rate_last_display = Some(now_time);
            trace!("msg_rate inst={inst:.2} ema={ema:.2}");
        }
        // Keep the last known rate indefinitely
    }

    fn apply_rate_decay(&mut self, now_time: SystemTime) {
        let Some(prev) = self.msg_rate_ema else {
            return;
        };
        let last = self.last_msg_time.unwrap_or(now_time);
        let dt = now_time
            .duration_since(last)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        let hold = self.msg_rate_window.as_secs_f64().max(2.0);
        if dt <= hold {
            return;
        }
        let tau = (self.msg_rate_window.as_secs_f64() * 4.0).max(3.0);
        let decay = (-dt / tau).exp();
        let ema = (prev * decay).max(0.0);
        self.msg_rate_ema = Some(ema);
        self.msg_rate = Some(ema);
        self.msg_rate_display = Some(ema);
    }

    fn update_aircraft_rates(&mut self, data: &ApiResponse, now_time: SystemTime) {
        let mut present = HashSet::new();
        let mut sum = 0.0;
        let mut count = 0usize;
        let mut total_delta_msgs: u64 = 0;
        let max_age = self.msg_rate_window + self.msg_rate_window;

        for ac in &data.aircraft {
            let key = if let Some(hex) = ac.hex.as_deref() {
                format!("hex:{}", normalize_hex(hex))
            } else if let Some(flight) = ac.flight.as_deref() {
                format!("flt:{}", normalize_callsign(flight))
            } else {
                continue;
            };
            present.insert(key.clone());

            let Some(messages) = ac.messages else {
                continue;
            };

            let entry = self
                .aircraft_rates
                .entry(key)
                .or_insert_with(|| AircraftRate {
                    last_messages: messages,
                    last_time: now_time,
                    last_rate_time: now_time,
                    ema: None,
                    rate: None,
                });

            if messages < entry.last_messages {
                entry.ema = None;
                entry.rate = None;
                entry.last_messages = messages;
                entry.last_time = now_time;
                entry.last_rate_time = now_time;
            } else if messages > entry.last_messages {
                if let Ok(delta_t) = now_time.duration_since(entry.last_time) {
                    if delta_t <= max_age {
                        total_delta_msgs =
                            total_delta_msgs.saturating_add(messages - entry.last_messages);
                    }
                    let secs = delta_t.as_secs_f64().max(self.msg_rate_min_secs);
                    let inst = (messages - entry.last_messages) as f64 / secs;
                    let ema = match entry.ema {
                        Some(prev) => 0.45 * inst + 0.55 * prev,
                        None => inst,
                    };
                    entry.ema = Some(ema);
                    entry.rate = Some(ema);
                }
                entry.last_messages = messages;
                entry.last_time = now_time;
                entry.last_rate_time = now_time;
            } else if let Some(prev) = entry.ema {
                let since_change = now_time
                    .duration_since(entry.last_time)
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0);
                let hold = self.msg_rate_window.as_secs_f64().max(2.0);
                if since_change > hold {
                    let dt = now_time
                        .duration_since(entry.last_rate_time)
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0);
                    if dt > 0.0 {
                        let tau = (self.msg_rate_window.as_secs_f64() * 4.0).max(3.0);
                        let decay = (-dt / tau).exp();
                        let ema = (prev * decay).max(0.0);
                        entry.ema = Some(ema);
                        entry.rate = Some(ema);
                        entry.last_rate_time = now_time;
                    }
                }
            }

            if let Some(rate) = entry.rate {
                sum += rate;
                count += 1;
            }
        }

        self.aircraft_rates.retain(|key, _| present.contains(key));
        self.avg_aircraft_rate = if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        };

        self.update_total_aircraft_rate(total_delta_msgs, now_time);
    }

    fn update_total_aircraft_rate(&mut self, delta_msgs: u64, now_time: SystemTime) {
        let hold = self.msg_rate_window.as_secs_f64().max(2.0);
        let tau = (self.msg_rate_window.as_secs_f64() * 4.0).max(3.0);
        if delta_msgs > 0 {
            let last_time = self.total_aircraft_rate_time.unwrap_or(now_time);
            let secs = now_time
                .duration_since(last_time)
                .map(|d| d.as_secs_f64())
                .unwrap_or(self.msg_rate_min_secs)
                .max(self.msg_rate_min_secs);
            let inst = delta_msgs as f64 / secs;
            let ema = match self.total_aircraft_rate_ema {
                Some(prev) => 0.45 * inst + 0.55 * prev,
                None => inst,
            };
            self.total_aircraft_rate_ema = Some(ema);
            self.total_aircraft_rate = Some(ema);
            self.total_aircraft_rate_time = Some(now_time);
            return;
        }

        if let Some(prev) = self.total_aircraft_rate_ema {
            let last_time = self.total_aircraft_rate_time.unwrap_or(now_time);
            let since = now_time
                .duration_since(last_time)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            if since > hold {
                let decay = (-since / tau).exp();
                let ema = (prev * decay).max(0.0);
                self.total_aircraft_rate_ema = Some(ema);
                self.total_aircraft_rate = Some(ema);
                self.total_aircraft_rate_time = Some(now_time);
            }
        }
    }

    fn update_seen_times(&mut self, data: &ApiResponse, now_time: SystemTime) {
        for ac in &data.aircraft {
            if let Some(hex) = ac.hex.as_deref() {
                self.seen_times.insert(normalize_hex(hex), now_time);
            }
        }
    }

    fn update_trends(&mut self, data: &ApiResponse) {
        for ac in &data.aircraft {
            if let Some(hex) = ac.hex.as_deref() {
                let key = normalize_hex(hex);
                let prev = self.last_metrics.get(&key).copied().unwrap_or_default();
                let current = Metrics {
                    alt_baro: ac.alt_baro,
                    gs: ac.gs,
                };
                let trend = Trend {
                    alt: compare_i64(prev.alt_baro, current.alt_baro),
                    gs: compare_f64(prev.gs, current.gs),
                };
                self.trend_cache.insert(key.clone(), trend);
                self.last_metrics.insert(key, current);
            }
        }
    }

    fn update_trails(&mut self, data: &ApiResponse, now_time: SystemTime) {
        let max_len = self.trail_len.max(1);
        for ac in &data.aircraft {
            if let (Some(hex), Some(lat), Some(lon)) = (ac.hex.as_deref(), ac.lat, ac.lon) {
                let key = normalize_hex(hex);
                let entry = self.trail_points.entry(key).or_default();
                if let Some(last) = entry.last().copied() {
                    let last_lat = last.lat;
                    let last_lon = last.lon;
                    if (last_lat - lat).abs() < 0.00001 && (last_lon - lon).abs() < 0.00001 {
                        continue;
                    }
                }
                entry.push(TrailPoint {
                    lat,
                    lon,
                    at: now_time,
                });
                if entry.len() > max_len {
                    let excess = entry.len() - max_len;
                    entry.drain(0..excess);
                }
            }
        }
    }

    fn update_notifications(&mut self, data: &ApiResponse, now: SystemTime) {
        let Some(site) = self.site() else {
            return;
        };
        let radius = self.notify_radius_mi;
        if radius <= 0.0 {
            return;
        }

        let max_age_secs = self.notify_cooldown.as_secs().saturating_mul(4).max(60);
        let max_age = Duration::from_secs(max_age_secs);
        self.notified_recent.retain(|_, last| {
            now.duration_since(*last)
                .map(|d| d <= max_age)
                .unwrap_or(true)
        });

        for ac in &data.aircraft {
            let (Some(lat), Some(lon)) = (ac.lat, ac.lon) else {
                continue;
            };
            let dist_mi = distance_mi(site.lat, site.lon, lat, lon);
            if dist_mi > radius {
                continue;
            }
            let key = if let Some(hex) = ac.hex.as_deref() {
                format!("hex:{}", normalize_hex(hex))
            } else if let Some(flight) = ac.flight.as_deref() {
                format!("flt:{}", normalize_callsign(flight))
            } else {
                continue;
            };
            let should_notify = match self.notified_recent.get(&key) {
                Some(last) => now
                    .duration_since(*last)
                    .map(|d| d >= self.notify_cooldown)
                    .unwrap_or(true),
                None => true,
            };
            if !should_notify {
                continue;
            }
            self.notified_recent.insert(key, now);

            let callsign = ac.flight.as_deref().unwrap_or("--").trim();
            let reg = ac.r.as_deref().unwrap_or("--");
            let prefix = if dist_mi <= self.overpass_mi {
                "OVER"
            } else {
                "NEAR"
            };
            let message = format!("{prefix} {callsign} {reg} {dist_mi:.1}mi");
            debug!("notify {message}");
            self.notifications.push(Notification { message, at: now });
        }

        if self.notifications.len() > 10 {
            let excess = self.notifications.len() - 10;
            self.notifications.drain(0..excess);
        }
    }

    fn update_watchlist_notifications(&mut self, data: &ApiResponse, now: SystemTime) {
        if !self.watchlist_enabled || self.watchlist.is_empty() {
            return;
        }

        let max_age_secs = self.notify_cooldown.as_secs().saturating_mul(4).max(60);
        let max_age = Duration::from_secs(max_age_secs);
        self.watch_notified_recent.retain(|_, last| {
            now.duration_since(*last)
                .map(|d| d <= max_age)
                .unwrap_or(true)
        });

        for ac in &data.aircraft {
            let Some(entry) = self.watch_entry_for(ac).cloned() else {
                continue;
            };
            if !entry.notify_enabled() {
                continue;
            }
            let key = if let Some(hex) = ac.hex.as_deref() {
                format!("hex:{}", normalize_hex(hex))
            } else if let Some(flight) = ac.flight.as_deref() {
                format!("flt:{}", normalize_callsign(flight))
            } else {
                continue;
            };
            let entry_id = entry.entry_id();
            let notify_key = format!("watch:{entry_id}:{key}");
            let should_notify = match self.watch_notified_recent.get(&notify_key) {
                Some(last) => now
                    .duration_since(*last)
                    .map(|d| d >= self.notify_cooldown)
                    .unwrap_or(true),
                None => true,
            };
            if !should_notify {
                continue;
            }
            self.watch_notified_recent.insert(notify_key, now);

            let callsign = ac.flight.as_deref().unwrap_or("--").trim();
            let reg = ac.r.as_deref().unwrap_or("--");
            let label = entry
                .label
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(entry_id.as_str());
            let message = format!("WATCH {label} {callsign} {reg}");
            debug!("notify {message}");
            self.notifications.push(Notification { message, at: now });
        }

        if self.notifications.len() > 10 {
            let excess = self.notifications.len() - 10;
            self.notifications.drain(0..excess);
        }
    }

    fn swap_snapshot(&mut self) {
        let mut next = self.raw_data.clone();
        if self.smooth_merge {
            merge_api_response(&mut next, &self.data);
        }
        self.data = next;
    }

    fn matches_filter(&self, ac: &Aircraft) -> bool {
        let filter = self.filter.trim();
        if filter.is_empty() {
            return true;
        }
        let needle = filter.to_lowercase();
        let haystacks = [
            ac.flight.as_deref(),
            ac.r.as_deref(),
            ac.t.as_deref(),
            ac.desc.as_deref(),
            ac.own_op.as_deref(),
            ac.hex.as_deref(),
        ];

        haystacks.iter().any(|value| {
            value
                .map(|v| v.to_lowercase().contains(&needle))
                .unwrap_or(false)
        })
    }
}

fn normalize_hex(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_callsign(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_text(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn rsi_from_series(values: &[u64], period: usize) -> Option<f64> {
    if period == 0 || values.len() <= period {
        return None;
    }
    let start = values.len().saturating_sub(period + 1);
    let window = &values[start..];
    let mut gains = 0.0;
    let mut losses = 0.0;
    for pair in window.windows(2) {
        let prev = pair[0] as f64;
        let curr = pair[1] as f64;
        if curr > prev {
            gains += curr - prev;
        } else if prev > curr {
            losses += prev - curr;
        }
    }
    let period_f = period as f64;
    let avg_gain = gains / period_f;
    let avg_loss = losses / period_f;
    if avg_gain == 0.0 && avg_loss == 0.0 {
        return Some(50.0);
    }
    if avg_loss == 0.0 {
        return Some(100.0);
    }
    if avg_gain == 0.0 {
        return Some(0.0);
    }
    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}

fn is_rate_limited_message(message: &str) -> bool {
    let msg = message.to_ascii_lowercase();
    msg.contains(" 429")
        || msg.contains("429 ")
        || msg.contains("too many requests")
        || msg.contains("rate limit")
}

fn classify_aircraft(ac: &Aircraft) -> AircraftRole {
    let callsign = ac
        .flight
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_ascii_uppercase();
    let owner = ac.own_op.as_deref().unwrap_or("").to_ascii_lowercase();
    let ac_type = ac.t.as_deref().unwrap_or("").to_ascii_uppercase();
    let desc = ac.desc.as_deref().unwrap_or("").to_ascii_lowercase();
    let category = ac.category.as_deref().unwrap_or("").to_ascii_lowercase();

    if matches_prefix(&callsign, &military_callsign_prefixes()) {
        return AircraftRole::Military;
    }

    if matches_prefix(&callsign, &government_callsign_prefixes()) {
        return AircraftRole::Government;
    }

    if contains_any(&owner, &military_owner_keywords())
        || contains_any(&desc, &military_owner_keywords())
        || likely_military_type(&ac_type)
        || category.contains("mil")
    {
        return AircraftRole::Military;
    }

    if contains_any(&owner, &government_owner_keywords())
        || contains_any(&desc, &government_owner_keywords())
        || matches_prefix(&callsign, &government_callsign_prefixes())
    {
        return AircraftRole::Government;
    }

    AircraftRole::Commercial
}

fn matches_prefix(value: &str, prefixes: &[&str]) -> bool {
    if value.is_empty() {
        return false;
    }
    prefixes
        .iter()
        .any(|p| value.starts_with(p) && value.len() >= p.len())
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    let lower = value.to_ascii_lowercase();
    needles.iter().any(|n| lower.contains(n))
}

fn likely_military_type(ac_type: &str) -> bool {
    let prefixes = [
        "C17", "C-17", "C130", "C-130", "K35", "KC10", "KC-10", "KC46", "E3", "E-3", "E6", "E-6",
        "P8", "P-8", "P3", "P-3", "B52", "B-52", "F1", "F2", "F3", "F4", "F5", "F6", "F14", "F15",
        "F16", "F18", "F22", "F35",
    ];
    let upper = ac_type.to_ascii_uppercase();
    prefixes.iter().any(|p| upper.starts_with(p))
}

fn military_callsign_prefixes() -> Vec<&'static str> {
    vec![
        "RCH", "MC", "MAF", "NAF", "BAF", "GAF", "LAGR", "TEXAN", "PAT", "SAM", "SPAR", "ASF",
        "CFC", "CAF", "HK", "VV", "VM", "AF", "SEN", "CNV", "JENA", "KING",
    ]
}

fn government_callsign_prefixes() -> Vec<&'static str> {
    vec![
        "NATION", "GOV", "GVT", "POL", "RIDER", "EAG", "EAGLE", "COAST", "CST",
    ]
}

fn military_owner_keywords() -> Vec<&'static str> {
    vec![
        "air force",
        "navy",
        "marine",
        "marines",
        "army",
        "usaf",
        "usn",
        "usmc",
        "uscg",
        "raf",
        "rcaf",
        "luftwaffe",
        "bundeswehr",
        "aeronautica",
        "military",
        "defence",
        "defense",
        "armée",
        "ejército",
    ]
}

fn government_owner_keywords() -> Vec<&'static str> {
    vec![
        "government",
        "gov",
        "border",
        "customs",
        "police",
        "gendarmerie",
        "coast guard",
        "homeland",
        "cbp",
        "state",
        "federal",
        "department",
        "ministry",
    ]
}

fn parse_retry_after_hint(message: &str) -> Option<u64> {
    message
        .split(|c: char| [' ', '(', ')'].contains(&c))
        .find_map(|part| {
            if let Some(rest) = part.strip_prefix("retry-after=") {
                rest.trim_end_matches('s').parse::<u64>().ok()
            } else {
                None
            }
        })
}

fn route_backoff_duration(attempts: u32) -> Duration {
    if attempts == 0 {
        return Duration::from_secs(0);
    }
    let shift = attempts.saturating_sub(1).min(7);
    let secs = 2u64.saturating_mul(1u64 << shift);
    Duration::from_secs(secs.min(300))
}

fn match_text(needle: &str, haystack: &str, mode: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    match mode {
        "prefix" => haystack.starts_with(needle),
        "contains" => haystack.contains(needle),
        _ => haystack == needle,
    }
}

fn parse_lookup_input(input: &str) -> Option<LookupKind> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Allow prefix forms like "hex:abc" or tokenized "hex abc".
    let (head, tail_opt) = if let Some(idx) = trimmed.find(':') {
        let (h, rest) = trimmed.split_at(idx);
        (
            h.trim().to_ascii_lowercase(),
            Some(rest[1..].trim().to_string()),
        )
    } else {
        let mut parts = trimmed.split_whitespace();
        let h = parts.next()?.to_ascii_lowercase();
        let rest = parts.collect::<Vec<_>>().join(" ");
        (h, if rest.is_empty() { None } else { Some(rest) })
    };

    match head.as_str() {
        "hex" => Some(LookupKind::Hex(split_list(tail_opt?))),
        "callsign" | "call" | "cs" => Some(LookupKind::Callsign(split_list(tail_opt?))),
        "reg" | "registration" => Some(LookupKind::Reg(split_list(tail_opt?))),
        "type" => Some(LookupKind::Type(split_list(tail_opt?))),
        "squawk" | "sqk" => Some(LookupKind::Squawk(split_list(tail_opt?))),
        "point" => parse_point(tail_opt?),
        "mil" => Some(LookupKind::Mil),
        "ladd" => Some(LookupKind::Ladd),
        "pia" => Some(LookupKind::Pia),
        // If no prefix, treat free text as callsign lookup.
        _ => {
            if let Some(rest) = tail_opt {
                Some(LookupKind::Callsign(split_list(format!("{head} {rest}"))))
            } else {
                Some(LookupKind::Callsign(vec![head]))
            }
        }
    }
}

fn split_list(text: String) -> Vec<String> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_point(args: String) -> Option<LookupKind> {
    let parts: Vec<_> = args
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.trim().is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }
    let lat = parts[0].trim().parse::<f64>().ok()?;
    let lon = parts[1].trim().parse::<f64>().ok()?;
    let radius = parts[2].trim().parse::<f64>().ok()?;
    Some(LookupKind::Point { lat, lon, radius })
}

fn watch_entry_matches(entry: &WatchEntry, ac: &Aircraft, route: Option<&RouteInfo>) -> bool {
    let match_type = entry.match_type.trim().to_ascii_lowercase();
    let mode = entry.match_mode();
    let value = entry.value.trim();
    if value.is_empty() {
        return false;
    }
    match match_type.as_str() {
        "hex" | "icao" | "icao_hex" => ac
            .hex
            .as_deref()
            .map(|hex| match_text(&normalize_hex(value), &normalize_hex(hex), mode))
            .unwrap_or(false),
        "callsign" | "flight" | "cs" => ac
            .flight
            .as_deref()
            .map(|cs| match_text(&normalize_callsign(value), &normalize_callsign(cs), mode))
            .unwrap_or(false),
        "reg" | "registration" => {
            ac.r.as_deref()
                .map(|reg| match_text(&normalize_text(value), &normalize_text(reg), mode))
                .unwrap_or(false)
        }
        "type" => {
            ac.t.as_deref()
                .map(|t| match_text(&normalize_text(value), &normalize_text(t), mode))
                .unwrap_or(false)
        }
        "owner" | "operator" | "ownop" => ac
            .own_op
            .as_deref()
            .map(|owner| match_text(&normalize_text(value), &normalize_text(owner), mode))
            .unwrap_or(false),
        "category" => ac
            .category
            .as_deref()
            .map(|cat| match_text(&normalize_text(value), &normalize_text(cat), mode))
            .unwrap_or(false),
        "route" => {
            let Some(info) = route else { return false };
            let mut route_text = String::new();
            if let (Some(origin), Some(dest)) = (info.origin.as_ref(), info.destination.as_ref()) {
                if !origin.trim().is_empty() && !dest.trim().is_empty() {
                    route_text = format!("{origin}-{dest}");
                }
            }
            if route_text.is_empty() {
                if let Some(route) = info.route.as_ref() {
                    route_text = route.clone();
                }
            }
            match_text(&normalize_text(value), &normalize_text(&route_text), mode)
        }
        _ => false,
    }
}

fn make_watchlist_entry(ac: &Aircraft) -> Option<WatchEntry> {
    if let Some(callsign) = ac
        .flight
        .as_deref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        let label =
            ac.r.as_deref()
                .map(|reg| format!("{callsign} {reg}"))
                .unwrap_or_else(|| callsign.to_string());
        return Some(WatchEntry {
            id: None,
            label: Some(label),
            match_type: "callsign".to_string(),
            value: callsign.to_string(),
            enabled: Some(true),
            notify: Some(true),
            priority: Some(0),
            mode: Some("exact".to_string()),
            color: None,
        });
    }
    if let Some(hex) = ac
        .hex
        .as_deref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        return Some(WatchEntry {
            id: None,
            label: Some(format!("HEX {hex}")),
            match_type: "hex".to_string(),
            value: hex.to_string(),
            enabled: Some(true),
            notify: Some(true),
            priority: Some(0),
            mode: Some("exact".to_string()),
            color: None,
        });
    }
    if let Some(reg) = ac.r.as_deref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
        return Some(WatchEntry {
            id: None,
            label: Some(format!("REG {reg}")),
            match_type: "reg".to_string(),
            value: reg.to_string(),
            enabled: Some(true),
            notify: Some(true),
            priority: Some(0),
            mode: Some("exact".to_string()),
            color: None,
        });
    }
    if let Some(ac_type) = ac.t.as_deref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
        return Some(WatchEntry {
            id: None,
            label: Some(format!("TYPE {ac_type}")),
            match_type: "type".to_string(),
            value: ac_type.to_string(),
            enabled: Some(true),
            notify: Some(true),
            priority: Some(0),
            mode: Some("exact".to_string()),
            color: None,
        });
    }
    None
}

fn watchlist_entry_key(entry: &WatchEntry) -> String {
    let match_type = entry.match_type.trim().to_ascii_lowercase();
    let value = entry.value.trim();
    let normalized = match match_type.as_str() {
        "hex" | "icao" | "icao_hex" => normalize_hex(value),
        "callsign" | "flight" | "cs" => normalize_callsign(value),
        _ => normalize_text(value),
    };
    format!("{match_type}:{normalized}")
}

fn compare_i64(prev: Option<i64>, current: Option<i64>) -> TrendDir {
    match (prev, current) {
        (Some(p), Some(c)) if c > p => TrendDir::Up,
        (Some(p), Some(c)) if c < p => TrendDir::Down,
        (Some(_), Some(_)) => TrendDir::Flat,
        _ => TrendDir::Unknown,
    }
}

fn compare_f64(prev: Option<f64>, current: Option<f64>) -> TrendDir {
    match (prev, current) {
        (Some(p), Some(c)) if c > p => TrendDir::Up,
        (Some(p), Some(c)) if c < p => TrendDir::Down,
        (Some(_), Some(_)) => TrendDir::Flat,
        _ => TrendDir::Unknown,
    }
}

fn distance_mi(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_mi = 3958.8_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r_mi * c
}

fn merge_api_response(target: &mut ApiResponse, prev: &ApiResponse) {
    if target.now.is_none() {
        target.now = prev.now;
    }
    if target.messages.is_none() {
        target.messages = prev.messages;
    }

    let mut prev_map = HashMap::new();
    for ac in &prev.aircraft {
        if let Some(hex) = ac.hex.as_deref() {
            prev_map.insert(format!("hex:{}", normalize_hex(hex)), ac);
        } else if let Some(flight) = ac.flight.as_deref() {
            prev_map.insert(format!("flt:{}", normalize_callsign(flight)), ac);
        }
    }

    for ac in &mut target.aircraft {
        let key = if let Some(hex) = ac.hex.as_deref() {
            Some(format!("hex:{}", normalize_hex(hex)))
        } else {
            ac.flight
                .as_deref()
                .map(|flight| format!("flt:{}", normalize_callsign(flight)))
        };

        let Some(key) = key else { continue };
        let Some(prev_ac) = prev_map.get(&key) else {
            continue;
        };

        fill_string(&mut ac.hex, &prev_ac.hex);
        fill_string(&mut ac.kind, &prev_ac.kind);
        fill_string(&mut ac.flight, &prev_ac.flight);
        fill_string(&mut ac.r, &prev_ac.r);
        fill_string(&mut ac.t, &prev_ac.t);
        fill_string(&mut ac.desc, &prev_ac.desc);
        fill_string(&mut ac.own_op, &prev_ac.own_op);
        fill_string(&mut ac.year, &prev_ac.year);
        fill_string(&mut ac.category, &prev_ac.category);
        fill_string(&mut ac.sil_type, &prev_ac.sil_type);

        fill_copy(&mut ac.alt_baro, prev_ac.alt_baro);
        fill_copy(&mut ac.alt_geom, prev_ac.alt_geom);
        fill_copy(&mut ac.gs, prev_ac.gs);
        fill_copy(&mut ac.track, prev_ac.track);
        fill_copy(&mut ac.baro_rate, prev_ac.baro_rate);
        fill_copy(&mut ac.nav_qnh, prev_ac.nav_qnh);
        fill_copy(&mut ac.nav_altitude_mcp, prev_ac.nav_altitude_mcp);
        fill_copy(&mut ac.lat, prev_ac.lat);
        fill_copy(&mut ac.lon, prev_ac.lon);
        fill_copy(&mut ac.nic, prev_ac.nic);
        fill_copy(&mut ac.rc, prev_ac.rc);
        fill_copy(&mut ac.seen_pos, prev_ac.seen_pos);
        fill_copy(&mut ac.version, prev_ac.version);
        fill_copy(&mut ac.nic_baro, prev_ac.nic_baro);
        fill_copy(&mut ac.nac_p, prev_ac.nac_p);
        fill_copy(&mut ac.nac_v, prev_ac.nac_v);
        fill_copy(&mut ac.sil, prev_ac.sil);
        fill_copy(&mut ac.alert, prev_ac.alert);
        fill_copy(&mut ac.spi, prev_ac.spi);
        fill_copy(&mut ac.messages, prev_ac.messages);
        fill_copy(&mut ac.seen, prev_ac.seen);
        fill_copy(&mut ac.rssi, prev_ac.rssi);
    }
}

fn fill_string(target: &mut Option<String>, source: &Option<String>) {
    let missing = target
        .as_deref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true);
    if missing {
        if let Some(value) = source.as_deref() {
            let value = value.trim();
            if !value.is_empty() {
                *target = Some(value.to_string());
            }
        }
    }
}

fn fill_copy<T: Copy>(target: &mut Option<T>, source: Option<T>) {
    if target.is_none() {
        *target = source;
    }
}

fn default_columns() -> Vec<ColumnConfig> {
    vec![
        ColumnConfig {
            id: ColumnId::Flag,
            label: "FLAG",
            width: 2,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Fav,
            label: "*",
            width: 1,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Watch,
            label: "W",
            width: 1,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Flight,
            label: "FLIGHT",
            width: 8,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Reg,
            label: "REG",
            width: 8,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Type,
            label: "TYPE",
            width: 5,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Route,
            label: "ROUTE",
            width: 9,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Alt,
            label: "ALT",
            width: 7,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Gs,
            label: "GS",
            width: 6,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Trk,
            label: "TRK",
            width: 5,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Lat,
            label: "LAT",
            width: 9,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Lon,
            label: "LON",
            width: 9,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Dist,
            label: "DIST",
            width: 6,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Brg,
            label: "BRG",
            width: 5,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Seen,
            label: "SEEN",
            width: 6,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Msgs,
            label: "MSGS",
            width: 6,
            visible: true,
        },
        ColumnConfig {
            id: ColumnId::Hex,
            label: "HEX",
            width: 6,
            visible: true,
        },
    ]
}

fn load_config_items(path: &PathBuf) -> Vec<ConfigItem> {
    let file_value = fs::read_to_string(path)
        .ok()
        .and_then(|content| toml::from_str::<Value>(&content).ok());
    let table = file_value.and_then(|value| value.as_table().cloned());

    let mut items = default_config_items();
    if let Some(map) = table.as_ref() {
        let mut index = HashMap::new();
        for (idx, item) in items.iter().enumerate() {
            index.insert(item.key.clone(), idx);
        }
        let mut extras = Vec::new();
        for (key, value) in map {
            let kind = match value {
                Value::String(_) => config::ConfigKind::Str,
                Value::Integer(_) => config::ConfigKind::Int,
                Value::Float(_) => config::ConfigKind::Float,
                Value::Boolean(_) => config::ConfigKind::Bool,
                _ => config::ConfigKind::Str,
            };
            let value = toml_value_to_string(value).unwrap_or_else(|| value.to_string());
            if let Some(idx) = index.get(key) {
                if let Some(item) = items.get_mut(*idx) {
                    item.value = value;
                    item.kind = kind;
                }
            } else {
                extras.push(ConfigItem {
                    key: key.to_string(),
                    value,
                    kind,
                });
            }
        }
        if !extras.is_empty() {
            extras.sort_by(|a, b| a.key.cmp(&b.key));
            items.extend(extras);
        }
    }

    items
}

fn default_config_items() -> Vec<ConfigItem> {
    config::config_specs()
        .iter()
        .map(|spec| ConfigItem {
            key: spec.key.to_string(),
            value: spec.default_string(),
            kind: spec.kind,
        })
        .collect()
}

fn toml_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.to_string()),
        Value::Integer(i) => Some(i.to_string()),
        Value::Float(f) => Some(format!("{f}")),
        Value::Boolean(b) => Some(b.to_string()),
        _ => None,
    }
}

fn parse_config_value(kind: config::ConfigKind, raw: &str) -> Result<Option<Value>, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Ok(None);
    }
    match kind {
        config::ConfigKind::Str => Ok(Some(Value::String(text.to_string()))),
        config::ConfigKind::Bool => {
            let lowered = text.to_ascii_lowercase();
            if matches!(lowered.as_str(), "1" | "true" | "yes" | "on") {
                Ok(Some(Value::Boolean(true)))
            } else if matches!(lowered.as_str(), "0" | "false" | "no" | "off") {
                Ok(Some(Value::Boolean(false)))
            } else {
                Err(format!("invalid bool: {text}"))
            }
        }
        config::ConfigKind::Int => match text.parse::<i64>() {
            Ok(value) => Ok(Some(Value::Integer(value))),
            Err(_) => Err(format!("invalid int for {}", text)),
        },
        config::ConfigKind::Float => match text.parse::<f64>() {
            Ok(value) => Ok(Some(Value::Float(value))),
            Err(_) => Err(format!("invalid float for {}", text)),
        },
    }
}

fn to_edit_value(value: Value) -> toml_edit::Item {
    match value {
        Value::String(s) => toml_edit::value(s),
        Value::Integer(i) => toml_edit::value(i),
        Value::Float(f) => toml_edit::value(f),
        Value::Boolean(b) => toml_edit::value(b),
        Value::Array(arr) => {
            let mut array = toml_edit::Array::new();
            for item in arr {
                if let Some(val) = toml_value_to_edit(item) {
                    array.push(val);
                }
            }
            toml_edit::Item::Value(toml_edit::Value::Array(array))
        }
        _ => toml_edit::Item::None,
    }
}

fn toml_value_to_edit(value: Value) -> Option<toml_edit::Value> {
    match value {
        Value::String(s) => Some(toml_edit::Value::from(s)),
        Value::Integer(i) => Some(toml_edit::Value::from(i)),
        Value::Float(f) => Some(toml_edit::Value::from(f)),
        Value::Boolean(b) => Some(toml_edit::Value::from(b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compare_f64, compare_i64, distance_mi, load_config_items, parse_config_value,
        watch_entry_matches, AircraftRole, App, PerformanceSample, RadarBlip, RadarRenderer,
        RouteInfo, TrendDir, WatchEntry,
    };
    use crate::config::ConfigKind;
    use crate::model::Aircraft;
    use std::collections::{HashSet, VecDeque};
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    fn sample_aircraft() -> Aircraft {
        Aircraft {
            hex: Some("ac6668".to_string()),
            flight: Some("SWA3576 ".to_string()),
            r: Some("N8987Q".to_string()),
            t: Some("B38M".to_string()),
            own_op: Some("SOUTHWEST AIRLINES CO".to_string()),
            category: Some("A3".to_string()),
            ..Aircraft::default()
        }
    }

    fn make_app(role_enabled: bool, role_highlight: bool) -> App {
        App::new(
            "http://example".to_string(),
            Duration::from_secs(1),
            60.0,
            false,
            5,
            8,
            HashSet::new(),
            String::new(),
            crate::app::LayoutMode::Full,
            crate::app::ThemeMode::Default,
            role_enabled,
            role_highlight,
            true,
            Duration::from_millis(400),
            PathBuf::from("adsb-tui.toml"),
            3,
            None,
            None,
            false,
            200.0,
            1.0,
            crate::app::RadarRenderer::Canvas,
            false,
            crate::app::RadarBlip::Dot,
            false,
            false,
            Duration::from_secs(1),
            Duration::from_secs(1),
            1,
            10,
            true,
            true,
            Duration::from_millis(300),
            0.2,
            10.0,
            0.5,
            Duration::from_secs(60),
            true,
            true,
            true,
            crate::app::FlagStyle::Emoji,
            "msg_rate_total".to_string(),
            "kbps_total".to_string(),
            "msg_rate_avg".to_string(),
            true,
            Some(PathBuf::from("adsb-watchlist.toml")),
            Vec::new(),
        )
    }

    #[test]
    fn watch_entry_matching_modes() {
        let ac = sample_aircraft();
        let route = RouteInfo {
            origin: Some("KJFK".to_string()),
            destination: Some("KMIA".to_string()),
            route: None,
            fetched_at: SystemTime::now(),
        };

        let entry = WatchEntry {
            id: None,
            label: None,
            match_type: "hex".to_string(),
            value: "AC6668".to_string(),
            enabled: Some(true),
            notify: Some(true),
            priority: None,
            mode: Some("exact".to_string()),
            color: None,
        };
        assert!(watch_entry_matches(&entry, &ac, None));

        let entry = WatchEntry {
            match_type: "callsign".to_string(),
            value: "SWA".to_string(),
            mode: Some("prefix".to_string()),
            ..entry.clone()
        };
        assert!(watch_entry_matches(&entry, &ac, None));

        let entry = WatchEntry {
            match_type: "reg".to_string(),
            value: "8987".to_string(),
            mode: Some("contains".to_string()),
            ..entry.clone()
        };
        assert!(watch_entry_matches(&entry, &ac, None));

        let entry = WatchEntry {
            match_type: "type".to_string(),
            value: "B38M".to_string(),
            mode: Some("exact".to_string()),
            ..entry.clone()
        };
        assert!(watch_entry_matches(&entry, &ac, None));

        let entry = WatchEntry {
            match_type: "owner".to_string(),
            value: "SOUTHWEST".to_string(),
            mode: Some("contains".to_string()),
            ..entry.clone()
        };
        assert!(watch_entry_matches(&entry, &ac, None));

        let entry = WatchEntry {
            match_type: "category".to_string(),
            value: "A3".to_string(),
            mode: Some("exact".to_string()),
            ..entry.clone()
        };
        assert!(watch_entry_matches(&entry, &ac, None));

        let entry = WatchEntry {
            match_type: "route".to_string(),
            value: "KJFK-KMIA".to_string(),
            mode: Some("exact".to_string()),
            ..entry.clone()
        };
        assert!(watch_entry_matches(&entry, &ac, Some(&route)));

        let entry = WatchEntry {
            match_type: "unknown".to_string(),
            value: "X".to_string(),
            mode: Some("exact".to_string()),
            ..entry.clone()
        };
        assert!(!watch_entry_matches(&entry, &ac, None));
    }

    #[test]
    fn watchlist_priority_pick() {
        let watchlist = vec![
            WatchEntry {
                id: Some("low".to_string()),
                label: None,
                match_type: "hex".to_string(),
                value: "ac6668".to_string(),
                enabled: Some(true),
                notify: Some(true),
                priority: Some(1),
                mode: Some("exact".to_string()),
                color: None,
            },
            WatchEntry {
                id: Some("high".to_string()),
                label: None,
                match_type: "hex".to_string(),
                value: "ac6668".to_string(),
                enabled: Some(true),
                notify: Some(true),
                priority: Some(5),
                mode: Some("exact".to_string()),
                color: None,
            },
        ];

        let app = App::new(
            "http://example".to_string(),
            Duration::from_secs(1),
            60.0,
            false,
            5,
            8,
            HashSet::new(),
            String::new(),
            crate::app::LayoutMode::Full,
            crate::app::ThemeMode::Default,
            true,
            true,
            true,
            Duration::from_millis(400),
            PathBuf::from("adsb-tui.toml"),
            3,
            None,
            None,
            false,
            200.0,
            1.0,
            crate::app::RadarRenderer::Canvas,
            false,
            crate::app::RadarBlip::Dot,
            false,
            false,
            Duration::from_secs(1),
            Duration::from_secs(1),
            1,
            10,
            true,
            true,
            Duration::from_millis(300),
            0.2,
            10.0,
            0.5,
            Duration::from_secs(60),
            true,
            true,
            true,
            crate::app::FlagStyle::Emoji,
            "msg_rate_total".to_string(),
            "kbps_total".to_string(),
            "msg_rate_avg".to_string(),
            true,
            Some(PathBuf::from("adsb-watchlist.toml")),
            watchlist,
        );

        let ac = sample_aircraft();
        let entry = app.watch_entry_for(&ac).unwrap();
        assert_eq!(entry.entry_id(), "high");
    }

    #[test]
    fn role_disabled_masks_classification() {
        let mut ac = sample_aircraft();
        ac.flight = Some("RCH123".to_string()); // military prefix

        let app = make_app(false, true);
        assert!(matches!(app.classify_aircraft(&ac), AircraftRole::Unknown));
    }

    #[test]
    fn performance_snapshot_scales_signal_strength() {
        let mut app = make_app(true, true);
        let mut samples = VecDeque::new();
        for i in 0..15 {
            let rssi = -50.0 + (i as f64) * (50.0 / 14.0);
            samples.push_back(PerformanceSample {
                msg_rate: Some(12.0 + i as f64),
                flights: 3 + i as usize,
                rssi_avg: Some(rssi),
            });
        }
        app.perf_samples = samples;

        let snapshot = app.performance_snapshot();
        assert_eq!(snapshot.msg_rate.last().copied(), Some(26));
        assert_eq!(snapshot.flights.last().copied(), Some(17));
        assert_eq!(snapshot.signal.len(), 15);
        assert_eq!(snapshot.signal.first().copied(), Some(0));
        assert_eq!(snapshot.signal.last().copied(), Some(100));
        let rsi = snapshot.latest_signal_rsi.unwrap_or_default();
        assert!(rsi >= 99.0);
    }

    #[test]
    fn role_enabled_classifies_military() {
        let mut ac = sample_aircraft();
        ac.flight = Some("RCH123".to_string());

        let app = make_app(true, true);
        assert!(matches!(app.classify_aircraft(&ac), AircraftRole::Military));
    }

    #[test]
    fn compare_trends() {
        assert_eq!(compare_i64(Some(10), Some(20)), TrendDir::Up);
        assert_eq!(compare_i64(Some(20), Some(10)), TrendDir::Down);
        assert_eq!(compare_i64(Some(10), Some(10)), TrendDir::Flat);
        assert_eq!(compare_i64(None, Some(10)), TrendDir::Unknown);
        assert_eq!(compare_f64(Some(1.0), Some(2.0)), TrendDir::Up);
    }

    #[test]
    fn parse_config_value_cases() {
        assert!(parse_config_value(ConfigKind::Str, "abc")
            .unwrap()
            .is_some());
        assert_eq!(
            parse_config_value(ConfigKind::Bool, "true").unwrap(),
            Some(toml::Value::Boolean(true))
        );
        assert_eq!(
            parse_config_value(ConfigKind::Bool, "0").unwrap(),
            Some(toml::Value::Boolean(false))
        );
        assert!(parse_config_value(ConfigKind::Int, "42").unwrap().is_some());
        assert!(parse_config_value(ConfigKind::Float, "1.5")
            .unwrap()
            .is_some());
        assert!(parse_config_value(ConfigKind::Str, " ").unwrap().is_none());
        assert!(parse_config_value(ConfigKind::Bool, "maybe").is_err());
    }

    fn write_temp_config(contents: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        path.push(format!("adsb-tui-test-{nanos}.toml"));
        std::fs::write(&path, contents).expect("write temp config");
        path
    }

    #[test]
    fn load_config_items_merges_defaults() {
        let path =
            write_temp_config("refresh_secs = 5\nrole_enabled = false\nunknown_key = \"abc\"\n");

        let items = load_config_items(&path);
        let refresh = items.iter().find(|item| item.key == "refresh_secs");
        let role = items.iter().find(|item| item.key == "role_enabled");
        let url = items.iter().find(|item| item.key == "url");
        let extra = items.iter().find(|item| item.key == "unknown_key");

        assert!(refresh.is_some());
        assert_eq!(refresh.unwrap().value, "5");
        assert!(role.is_some());
        assert_eq!(role.unwrap().value, "false");
        assert!(url.is_some());
        assert!(extra.is_some());
        assert_eq!(extra.unwrap().value, "abc");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn distance_same_point_is_zero() {
        let dist = distance_mi(26.0, -80.0, 26.0, -80.0);
        assert!(dist.abs() < 0.0001);
    }

    #[test]
    fn radar_blip_parses() {
        assert_eq!(RadarBlip::from_str("dot"), RadarBlip::Dot);
        assert_eq!(RadarBlip::from_str("block"), RadarBlip::Block);
        assert_eq!(RadarBlip::from_str("cube"), RadarBlip::Block);
        assert_eq!(RadarBlip::from_str("plane"), RadarBlip::Plane);
        assert_eq!(RadarBlip::from_str("aircraft"), RadarBlip::Plane);
    }

    #[test]
    fn radar_renderer_parses() {
        assert_eq!(RadarRenderer::from_str("ascii"), RadarRenderer::Ascii);
        assert_eq!(RadarRenderer::from_str("canvas"), RadarRenderer::Canvas);
        assert_eq!(RadarRenderer::from_str("other"), RadarRenderer::Canvas);
    }
}
