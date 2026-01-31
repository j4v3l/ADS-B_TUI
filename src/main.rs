mod app;
mod config;
mod export;
mod model;
mod net;
mod routes;
mod runtime;
mod storage;
mod ui;

use anyhow::Result;
use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Duration;

use app::{App, LayoutMode, SiteLocation, ThemeMode};
use config::parse_args;
use net::spawn_fetcher;
use routes::spawn_route_fetcher;
use runtime::{init_terminal, restore_terminal, run_app, RouteChannels};
use std::path::PathBuf;
use storage::load_favorites;

fn main() -> Result<()> {
    let config = parse_args()?;
    let (tx, rx) = mpsc::channel();

    spawn_fetcher(config.url.clone(), config.refresh, config.insecure, tx);

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

    let layout_mode = LayoutMode::from_str(&config.layout);
    let theme_mode = ThemeMode::from_str(&config.theme);
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
        ),
        rx,
        route_channels,
    );
    restore_terminal(&mut terminal)?;

    if let Err(err) = res {
        eprintln!("{err}");
    }

    Ok(())
}
