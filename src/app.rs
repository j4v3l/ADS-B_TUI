use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use toml::Value;
use toml_edit::DocumentMut;

use crate::config;
use crate::model::{seen_seconds, Aircraft, ApiResponse};

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutMode {
    Full,
    Compact,
}

impl LayoutMode {
    pub fn toggle(self) -> Self {
        match self {
            LayoutMode::Full => LayoutMode::Compact,
            LayoutMode::Compact => LayoutMode::Full,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LayoutMode::Full => "FULL",
            LayoutMode::Compact => "COMPACT",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "compact" => LayoutMode::Compact,
            _ => LayoutMode::Full,
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
}

impl ThemeMode {
    pub fn toggle(self) -> Self {
        match self {
            ThemeMode::Default => ThemeMode::ColorBlind,
            ThemeMode::ColorBlind => ThemeMode::Amber,
            ThemeMode::Amber => ThemeMode::Ocean,
            ThemeMode::Ocean => ThemeMode::Matrix,
            ThemeMode::Matrix => ThemeMode::Default,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Default => "DEFAULT",
            ThemeMode::ColorBlind => "COLOR",
            ThemeMode::Amber => "AMBER",
            ThemeMode::Ocean => "OCEAN",
            ThemeMode::Matrix => "MATRIX",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "color" | "colorblind" | "cb" => ThemeMode::ColorBlind,
            "amber" | "gold" => ThemeMode::Amber,
            "ocean" | "blue" => ThemeMode::Ocean,
            "matrix" | "green" => ThemeMode::Matrix,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trend {
    pub alt: TrendDir,
    pub gs: TrendDir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnId {
    Fav,
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

#[derive(Clone, Copy, Debug)]
pub enum ConfigKind {
    Str,
    Bool,
    Int,
    Float,
}

#[derive(Clone, Debug)]
pub struct ConfigItem {
    pub key: &'static str,
    pub value: String,
    pub kind: ConfigKind,
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
    ema: Option<f64>,
    rate: Option<f64>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Metrics {
    alt_baro: Option<i64>,
    gs: Option<f64>,
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
    pub(crate) low_nic: i64,
    pub(crate) low_nac: i64,
    pub(crate) favorites: HashSet<String>,
    pub(crate) favorites_path: Option<PathBuf>,
    pub(crate) filter: String,
    pub(crate) filter_edit: String,
    pub(crate) input_mode: InputMode,
    pub(crate) layout_mode: LayoutMode,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) column_cache_enabled: bool,
    pub(crate) column_cache_ttl: Duration,
    column_width_cache: Option<ColumnWidthCache>,
    pub(crate) config_path: PathBuf,
    pub(crate) config_items: Vec<ConfigItem>,
    pub(crate) config_cursor: usize,
    pub(crate) config_edit: String,
    pub(crate) config_editing: bool,
    pub(crate) config_status: Option<(String, SystemTime)>,
    pub(crate) trail_len: usize,
    pub(crate) site: Option<SiteLocation>,
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
    #[allow(dead_code)]
    pub(crate) flags_enabled: bool,
    pub(crate) route_last_poll: Option<SystemTime>,
    pub(crate) route_cache: HashMap<String, RouteInfo>,
    pub(crate) route_last_request: HashMap<String, SystemTime>,
    pub(crate) msg_rate: Option<f64>,
    msg_rate_display: Option<f64>,
    msg_rate_ema: Option<f64>,
    msg_rate_last_display: Option<SystemTime>,
    msg_rate_window: Duration,
    msg_rate_min_secs: f64,
    msg_samples: VecDeque<(SystemTime, u64)>,
    aircraft_rates: HashMap<String, AircraftRate>,
    avg_aircraft_rate: Option<f64>,
    pub(crate) notify_radius_mi: f64,
    pub(crate) overpass_mi: f64,
    pub(crate) notify_cooldown: Duration,
    notified_recent: HashMap<String, SystemTime>,
    pub(crate) notifications: Vec<Notification>,
    pub(crate) last_msg_total: Option<u64>,
    pub(crate) last_msg_time: Option<SystemTime>,
    pub(crate) seen_times: HashMap<String, SystemTime>,
    last_metrics: HashMap<String, Metrics>,
    pub(crate) trend_cache: HashMap<String, Trend>,
    pub(crate) trail_points: HashMap<String, Vec<TrailPoint>>,
    pub(crate) last_export: Option<(String, SystemTime)>,
    pub(crate) route_error: Option<(String, SystemTime)>,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        url: String,
        refresh: Duration,
        stale_secs: f64,
        low_nic: i64,
        low_nac: i64,
        favorites: HashSet<String>,
        filter: String,
        layout_mode: LayoutMode,
        theme_mode: ThemeMode,
        column_cache_enabled: bool,
        column_cache_ttl: Duration,
        config_path: PathBuf,
        trail_len: usize,
        favorites_path: Option<PathBuf>,
        site: Option<SiteLocation>,
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
    ) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let ui_interval = if ui_fps == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_millis((1000.0 / ui_fps.max(1) as f64) as u64)
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
            low_nic,
            low_nac,
            favorites,
            favorites_path,
            filter,
            filter_edit: String::new(),
            input_mode: InputMode::Normal,
            layout_mode,
            theme_mode,
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
            config_status: None,
            trail_len: trail_len.max(1),
            site,
            columns: {
                let mut cols = default_columns();
                if let Some(flag_col) = cols.iter_mut().find(|c| c.id == ColumnId::Flag) {
                    flag_col.visible = flags_enabled;
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
            flags_enabled,
            route_last_poll: None,
            route_cache: HashMap::new(),
            route_last_request: HashMap::new(),
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
            notify_radius_mi: notify_radius_mi.max(0.1),
            overpass_mi: overpass_mi.max(0.05),
            notify_cooldown: if notify_cooldown.is_zero() {
                Duration::from_secs(120)
            } else {
                notify_cooldown
            },
            notified_recent: HashMap::new(),
            notifications: Vec::new(),
            last_msg_total: None,
            last_msg_time: None,
            seen_times: HashMap::new(),
            last_metrics: HashMap::new(),
            trend_cache: HashMap::new(),
            trail_points: HashMap::new(),
            last_export: None,
            route_error: None,
        }
    }

    pub fn apply_update(&mut self, data: ApiResponse) {
        let now_time = SystemTime::now();
        self.update_rate(&data, now_time);
        self.update_aircraft_rates(&data, now_time);
        self.update_seen_times(&data, now_time);
        self.update_trends(&data);
        self.update_trails(&data, now_time);
        self.update_notifications(&data, now_time);

        self.raw_data = data;
        if !self.smooth_mode {
            self.swap_snapshot();
        }
        self.last_update = Some(now_time);
        self.last_error = None;
    }

    pub fn apply_error(&mut self, msg: String) {
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
    }

    pub fn toggle_theme(&mut self) {
        self.theme_mode = self.theme_mode.toggle();
    }

    pub fn toggle_layout(&mut self) {
        self.layout_mode = self.layout_mode.toggle();
    }

    pub fn open_columns(&mut self) {
        self.input_mode = InputMode::Columns;
    }

    pub fn close_columns(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn open_help(&mut self) {
        self.input_mode = InputMode::Help;
    }

    pub fn close_help(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn open_config(&mut self) {
        self.config_items = load_config_items(&self.config_path);
        self.config_cursor = 0;
        self.config_editing = false;
        self.config_edit.clear();
        self.config_status = None;
        self.input_mode = InputMode::Config;
    }

    pub fn close_config(&mut self) {
        self.input_mode = InputMode::Normal;
        self.config_editing = false;
        self.config_edit.clear();
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
        }
    }

    pub fn cancel_config_edit(&mut self) {
        self.config_editing = false;
        self.config_edit.clear();
    }

    pub fn apply_config_edit(&mut self) {
        if let Some(item) = self.config_items.get_mut(self.config_cursor) {
            item.value = self.config_edit.trim().to_string();
        }
        self.config_editing = false;
        self.config_edit.clear();
    }

    pub fn push_config_char(&mut self, ch: char) {
        self.config_edit.push(ch);
    }

    pub fn backspace_config(&mut self) {
        self.config_edit.pop();
    }

    pub fn save_config(&mut self) {
        let existing = fs::read_to_string(&self.config_path).unwrap_or_default();
        let mut doc = existing
            .parse::<DocumentMut>()
            .unwrap_or_else(|_| DocumentMut::new());

        for item in &self.config_items {
            match parse_config_value(item.kind, item.value.trim()) {
                Ok(Some(value)) => {
                    doc[item.key] = to_edit_value(value);
                }
                Ok(None) => {
                    doc.remove(item.key);
                }
                Err(err) => {
                    self.config_status = Some((err, SystemTime::now()));
                    return;
                }
            }
        }

        let text = doc.to_string();
        if let Err(err) = fs::write(&self.config_path, text) {
            self.config_status = Some((format!("save failed: {err}"), SystemTime::now()));
        } else {
            self.config_status = Some((
                format!("saved {} (restart to apply)", self.config_path.display()),
                SystemTime::now(),
            ));
        }
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
            .filter(|(_, ac)| self.matches_filter(ac))
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
                return true;
            }
        }
        false
    }

    pub fn favorites_path(&self) -> Option<&PathBuf> {
        self.favorites_path.as_ref()
    }

    pub fn site(&self) -> Option<SiteLocation> {
        self.site
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

    pub fn msg_rate_display(&self) -> Option<f64> {
        self.msg_rate.or(self.msg_rate_display)
    }

    pub fn avg_aircraft_rate(&self) -> Option<f64> {
        self.avg_aircraft_rate
    }

    pub fn route_refresh_due(&mut self, now: SystemTime) -> bool {
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
    }

    pub fn set_route_error(&mut self, message: String) {
        self.route_error = Some((message, SystemTime::now()));
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
        for idx in indices {
            if requests.len() >= self.route_batch {
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

    pub fn start_filter(&mut self) {
        self.filter_edit = self.filter.clone();
        self.input_mode = InputMode::Filter;
    }

    pub fn apply_filter(&mut self) {
        self.filter = self.filter_edit.trim().to_string();
        self.input_mode = InputMode::Normal;
    }

    pub fn cancel_filter(&mut self) {
        self.filter_edit.clear();
        self.input_mode = InputMode::Normal;
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
    }

    pub fn push_filter_char(&mut self, ch: char) {
        self.filter_edit.push(ch);
    }

    pub fn backspace_filter(&mut self) {
        self.filter_edit.pop();
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

    fn update_rate(&mut self, data: &ApiResponse, now_time: SystemTime) {
        let hold = {
            let mut value = self.msg_rate_window + self.msg_rate_window;
            if value < Duration::from_secs(2) {
                value = Duration::from_secs(2);
            }
            value
        };

        if let Some(messages) = data.messages {
            if let Some(last_total) = self.last_msg_total {
                if messages < last_total {
                    self.msg_samples.clear();
                    self.msg_rate_ema = None;
                    self.msg_rate_display = None;
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
                if let Some(last) = self.msg_rate_last_display {
                    if now_time
                        .duration_since(last)
                        .map(|d| d < hold)
                        .unwrap_or(true)
                    {
                        return;
                    }
                }
                self.msg_rate = None;
                self.msg_rate_ema = None;
                self.msg_rate_display = None;
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
        }
        // Keep the last known rate indefinitely
    }

    fn update_aircraft_rates(&mut self, data: &ApiResponse, now_time: SystemTime) {
        let mut present = HashSet::new();
        let mut sum = 0.0;
        let mut count = 0usize;

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
                    ema: None,
                    rate: None,
                });

            if messages < entry.last_messages {
                entry.ema = None;
                entry.rate = None;
            } else if messages > entry.last_messages {
                if let Ok(delta_t) = now_time.duration_since(entry.last_time) {
                    let secs = delta_t.as_secs_f64().max(self.msg_rate_min_secs);
                    let inst = (messages - entry.last_messages) as f64 / secs;
                    let ema = match entry.ema {
                        Some(prev) => 0.45 * inst + 0.55 * prev,
                        None => inst,
                    };
                    entry.ema = Some(ema);
                    entry.rate = Some(ema);
                }
            }

            entry.last_messages = messages;
            entry.last_time = now_time;

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
        let Some(site) = self.site else {
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
            id: ColumnId::Fav,
            label: "*",
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
        ColumnConfig {
            id: ColumnId::Flag,
            label: "FLAG",
            width: 2,
            visible: true,
        },
    ]
}

fn load_config_items(path: &PathBuf) -> Vec<ConfigItem> {
    let file_value = fs::read_to_string(path)
        .ok()
        .and_then(|content| toml::from_str::<Value>(&content).ok());
    let table = file_value.and_then(|value| value.as_table().cloned());

    let mut items = Vec::new();
    let mut push_item = |key: &'static str, kind: ConfigKind, default: String| {
        let value = table
            .as_ref()
            .and_then(|t| t.get(key))
            .and_then(toml_value_to_string)
            .unwrap_or(default);
        items.push(ConfigItem { key, value, kind });
    };

    push_item("url", ConfigKind::Str, config::DEFAULT_URL.to_string());
    push_item(
        "refresh_secs",
        ConfigKind::Int,
        config::DEFAULT_REFRESH_SECS.to_string(),
    );
    push_item("insecure", ConfigKind::Bool, "false".to_string());
    push_item(
        "stale_secs",
        ConfigKind::Int,
        config::DEFAULT_STALE_SECS.to_string(),
    );
    push_item(
        "low_nic",
        ConfigKind::Int,
        config::DEFAULT_LOW_NIC.to_string(),
    );
    push_item(
        "low_nac",
        ConfigKind::Int,
        config::DEFAULT_LOW_NAC.to_string(),
    );
    push_item(
        "trail_len",
        ConfigKind::Int,
        config::DEFAULT_TRAIL_LEN.to_string(),
    );
    push_item(
        "favorites_file",
        ConfigKind::Str,
        config::DEFAULT_FAVORITES_FILE.to_string(),
    );
    push_item("filter", ConfigKind::Str, "".to_string());
    push_item("layout", ConfigKind::Str, "full".to_string());
    push_item("theme", ConfigKind::Str, "default".to_string());
    push_item("site_lat", ConfigKind::Float, "".to_string());
    push_item("site_lon", ConfigKind::Float, "".to_string());
    push_item("site_alt_m", ConfigKind::Float, "".to_string());
    push_item("route_enabled", ConfigKind::Bool, "true".to_string());
    push_item(
        "route_base",
        ConfigKind::Str,
        config::DEFAULT_ROUTE_BASE.to_string(),
    );
    push_item(
        "route_mode",
        ConfigKind::Str,
        config::DEFAULT_ROUTE_MODE.to_string(),
    );
    push_item(
        "route_path",
        ConfigKind::Str,
        config::DEFAULT_ROUTE_PATH.to_string(),
    );
    push_item(
        "route_ttl_secs",
        ConfigKind::Int,
        config::DEFAULT_ROUTE_TTL_SECS.to_string(),
    );
    push_item(
        "route_refresh_secs",
        ConfigKind::Int,
        config::DEFAULT_ROUTE_REFRESH_SECS.to_string(),
    );
    push_item(
        "route_batch",
        ConfigKind::Int,
        config::DEFAULT_ROUTE_BATCH.to_string(),
    );
    push_item(
        "route_timeout_secs",
        ConfigKind::Int,
        config::DEFAULT_ROUTE_TIMEOUT_SECS.to_string(),
    );
    push_item(
        "ui_fps",
        ConfigKind::Int,
        config::DEFAULT_UI_FPS.to_string(),
    );
    push_item(
        "smooth_mode",
        ConfigKind::Bool,
        config::DEFAULT_SMOOTH_MODE.to_string(),
    );
    push_item(
        "smooth_merge",
        ConfigKind::Bool,
        config::DEFAULT_SMOOTH_MERGE.to_string(),
    );
    push_item(
        "rate_window_ms",
        ConfigKind::Int,
        config::DEFAULT_RATE_WINDOW_MS.to_string(),
    );
    push_item(
        "rate_min_secs",
        ConfigKind::Float,
        config::DEFAULT_RATE_MIN_SECS.to_string(),
    );
    push_item(
        "notify_radius_mi",
        ConfigKind::Float,
        config::DEFAULT_NOTIFY_RADIUS_MI.to_string(),
    );
    push_item(
        "overpass_mi",
        ConfigKind::Float,
        config::DEFAULT_OVERPASS_MI.to_string(),
    );
    push_item(
        "notify_cooldown_secs",
        ConfigKind::Int,
        config::DEFAULT_NOTIFY_COOLDOWN_SECS.to_string(),
    );
    push_item(
        "altitude_trend_arrows",
        ConfigKind::Bool,
        config::DEFAULT_ALTITUDE_TREND_ARROWS.to_string(),
    );
    push_item(
        "track_arrows",
        ConfigKind::Bool,
        config::DEFAULT_TRACK_ARROWS.to_string(),
    );
    push_item(
        "column_cache",
        ConfigKind::Bool,
        config::DEFAULT_COLUMN_CACHE.to_string(),
    );
    push_item(
        "flags_enabled",
        ConfigKind::Bool,
        config::DEFAULT_FLAGS_ENABLED.to_string(),
    );

    items
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

fn parse_config_value(kind: ConfigKind, raw: &str) -> Result<Option<Value>, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Ok(None);
    }
    match kind {
        ConfigKind::Str => Ok(Some(Value::String(text.to_string()))),
        ConfigKind::Bool => {
            let lowered = text.to_ascii_lowercase();
            if matches!(lowered.as_str(), "1" | "true" | "yes" | "on") {
                Ok(Some(Value::Boolean(true)))
            } else if matches!(lowered.as_str(), "0" | "false" | "no" | "off") {
                Ok(Some(Value::Boolean(false)))
            } else {
                Err(format!("invalid bool: {text}"))
            }
        }
        ConfigKind::Int => match text.parse::<i64>() {
            Ok(value) => Ok(Some(Value::Integer(value))),
            Err(_) => Err(format!("invalid int for {}", text)),
        },
        ConfigKind::Float => match text.parse::<f64>() {
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
