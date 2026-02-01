mod app;
mod config;
mod export;
mod model;
mod net;
mod routes;
mod runtime;
mod storage;
mod ui;
mod radar;
mod logging;
mod watchlist;

use anyhow::Result;
use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Duration;

use app::{App, LayoutMode, RadarBlip, RadarRenderer, SiteLocation, ThemeMode};
use config::parse_args;
use logging::init as init_logging;
use net::spawn_fetcher;
use routes::spawn_route_fetcher;
use runtime::{init_terminal, restore_terminal, run_app, RouteChannels};
use std::path::PathBuf;
use storage::{load_favorites, load_watchlist};
use tracing::{debug, info, warn};

fn main() -> Result<()> {
    let config = parse_args()?;
    let _log_guard = init_logging(&config);
    info!("adsb-tui starting");
    debug!("config path: {}", config.config_path.display());
    let (tx, rx) = mpsc::channel();

    let api_key = if config.api_key.trim().is_empty() {
        None
    } else {
        Some(config.api_key.clone())
    };
    let api_key_header = if config.api_key_header.trim().is_empty() {
        None
    } else {
        Some(config.api_key_header.clone())
    };

    spawn_fetcher(
        config.url.clone(),
        config.refresh,
        config.insecure,
        api_key,
        api_key_header,
        tx,
    );

    let mut favorites: HashSet<String> = config
        .favorites
        .into_iter()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .collect();

    let favorites_path = if config.favorites_file.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(config.favorites_file))
    };

    if let Some(path) = favorites_path.as_ref() {
        if let Ok(file_favs) = load_favorites(path) {
            favorites.extend(file_favs);
        }
    }

    let watchlist_path = if config.watchlist_file.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(config.watchlist_file))
    };
    let mut watchlist = Vec::new();
    if config.watchlist_enabled {
        if let Some(path) = watchlist_path.as_ref() {
            if let Ok(entries) = load_watchlist(path) {
                watchlist = entries;
            } else {
                warn!("failed to load watchlist from {}", path.display());
            }
        }
    }

    let layout_mode = LayoutMode::from_str(&config.layout);
    let theme_mode = ThemeMode::from_str(&config.theme);
    let radar_renderer = RadarRenderer::from_str(&config.radar_renderer);
    let radar_blip = RadarBlip::from_str(&config.radar_blip);
    let site = match (config.site_lat, config.site_lon) {
        (Some(lat), Some(lon)) => Some(SiteLocation {
            lat,
            lon,
            alt_m: config.site_alt_m.unwrap_or(0.0),
        }),
        _ => None,
    };

    let mut terminal = init_terminal()?;
    let route_channels = if config.route_enabled {
        let (route_req_tx, route_req_rx) = mpsc::channel();
        let (route_res_tx, route_res_rx) = mpsc::channel();
        spawn_route_fetcher(
            config.route_base.clone(),
            config.route_mode.clone(),
            config.route_path.clone(),
            config.insecure,
            Duration::from_secs(config.route_timeout_secs.max(2)),
            route_res_tx,
            route_req_rx,
        );
        Some(RouteChannels {
            req_tx: route_req_tx,
            res_rx: route_res_rx,
        })
    } else {
        None
    };

    let res = run_app(
        &mut terminal,
        App::new(
            config.url,
            config.refresh,
            config.stale_secs as f64,
            config.hide_stale,
            config.low_nic,
            config.low_nac,
            favorites,
            config.filter,
            layout_mode,
            theme_mode,
            config.column_cache,
            Duration::from_millis(400),
            config.config_path.clone(),
            config.trail_len as usize,
            favorites_path.clone(),
            site,
            config.radar_range_nm,
            config.radar_aspect,
            radar_renderer,
            config.radar_labels,
            radar_blip,
            config.route_enabled,
            config.route_mode.eq_ignore_ascii_case("tar1090"),
            Duration::from_secs(config.route_ttl_secs),
            Duration::from_secs(config.route_refresh_secs),
            config.route_batch as usize,
            config.ui_fps,
            config.smooth_mode,
            config.smooth_merge,
            Duration::from_millis(config.rate_window_ms),
            config.rate_min_secs,
            config.notify_radius_mi,
            config.overpass_mi,
            Duration::from_secs(config.notify_cooldown_secs),
            config.altitude_trend_arrows,
            config.track_arrows,
            config.flags_enabled,
            config.stats_metric_1.clone(),
            config.stats_metric_2.clone(),
            config.stats_metric_3.clone(),
            config.watchlist_enabled,
            watchlist_path.clone(),
            watchlist,
        ),
        rx,
        route_channels,
    );
    restore_terminal(&mut terminal)?;

    if let Err(err) = res {
        warn!("runtime error: {err}");
        eprintln!("{err}");
    }

    info!("adsb-tui exited");
    Ok(())
}
