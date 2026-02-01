use chrono::{DateTime, Local, Utc};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap};
use ratatui::Frame;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::app::{App, ColumnId, InputMode, LayoutMode, SiteLocation, ThemeMode, TrendDir};
use crate::model::seen_seconds;
use crate::radar::{self, RadarSettings, RadarTheme};

struct Theme {
    accent: Color,
    warn: Color,
    danger: Color,
    dim: Color,
    highlight_fg: Color,
    highlight_bg: Color,
    fav: Color,
    watch: Color,
    row_even_bg: Color,
    row_odd_bg: Color,
    header_bg: Color,
    panel_bg: Color,
}

pub fn ui(f: &mut Frame, app: &mut App, indices: &[usize]) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(size);

    render_header(f, chunks[0], app);
    render_alerts(f, chunks[1], app, indices);

    match app.layout_mode {
        LayoutMode::Full => render_full_body(f, chunks[2], app, indices),
        LayoutMode::Compact => render_compact_body(f, chunks[2], app, indices),
        LayoutMode::Radar => render_radar_body(f, chunks[2], app, indices),
    }

    render_footer(f, chunks[3], app);

    if app.input_mode == InputMode::Columns {
        render_columns_menu(f, size, app);
    }

    if app.input_mode == InputMode::Help {
        render_help_menu(f, size, app);
    }

    if app.input_mode == InputMode::Config {
        render_config_menu(f, size, app);
    }

    if app.input_mode == InputMode::Legend {
        render_legend_menu(f, size, app);
    }

    if app.input_mode == InputMode::Watchlist {
        render_watchlist_menu(f, size, app);
    }
}

fn render_full_body(f: &mut Frame, area: Rect, app: &mut App, indices: &[usize]) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(area);

    render_table(f, body[0], app, indices);

    let side = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Min(10),
        ])
        .split(body[1]);

    render_stats(f, side[0], app, indices);
    render_radar(f, side[1], app, indices);
    render_details(f, side[2], app, indices);
}

fn render_compact_body(f: &mut Frame, area: Rect, app: &mut App, indices: &[usize]) {
    render_table(f, area, app, indices);
}

fn render_radar_body(f: &mut Frame, area: Rect, app: &App, indices: &[usize]) {
    render_radar(f, area, app, indices);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let count = app.data.aircraft.len();
    let msg_total = app.data.messages.unwrap_or(0);
    let total_rate = app.msg_rate_display();
    let avg_rate = app.avg_aircraft_rate();
    let total_kbps = total_rate.map(|rate| rate * 112.0 / 1000.0);
    let avg_kbps = avg_rate.map(|rate| rate * 112.0 / 1000.0);

    let api_time = app
        .data
        .now
        .and_then(format_epoch)
        .unwrap_or_else(|| "--".to_string());

    let update_time = app
        .last_update
        .map(format_system_time)
        .unwrap_or_else(|| "--".to_string());

    let status = if let Some(err) = &app.last_error {
        format!("ERR: {err}")
    } else {
        "OK".to_string()
    };

    let status_color = if app.last_error.is_some() {
        theme.danger
    } else {
        Color::Green
    };

    let spinner = ["|", "/", "-", "\\"][phase_index(200, 4)];
    let since_update_ms = app
        .last_update
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .map(|d| d.as_millis() as u64);
    let sync_style = match since_update_ms {
        Some(ms) if ms < 1200 => {
            if phase_ms(500) {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.accent)
            }
        }
        _ => Style::default().fg(theme.dim),
    };

    let total_text = match (total_rate, total_kbps) {
        (Some(rate), Some(kbps)) => format!("{rate:.1}/s {kbps:.1}kbps"),
        (Some(rate), None) => format!("{rate:.1}/s"),
        _ => "--".to_string(),
    };
    let avg_text = match (avg_rate, avg_kbps) {
        (Some(rate), Some(kbps)) => format!("{rate:.1}/s {kbps:.1}kbps"),
        (Some(rate), None) => format!("{rate:.1}/s"),
        _ => "--".to_string(),
    };

    let line_top = Line::from(vec![
        Span::styled(
            "ADSB BOARD",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("AIRCRAFT {count}"),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" | "),
        Span::raw(format!("MSGS {msg_total}")),
        Span::raw(" | "),
        Span::styled(format!("TOT {total_text}"), Style::default().fg(theme.accent)),
        Span::raw(" | "),
        Span::styled(format!("AVG {avg_text}"), Style::default().fg(theme.dim)),
    ]);

    let line_bottom = Line::from(vec![
        Span::raw(format!("API {api_time}")),
        Span::raw(" | "),
        Span::raw(format!("SORT {}", app.sort.label())),
        Span::raw(" | "),
        Span::raw(format!("VIEW {}", app.layout_mode.label())),
        Span::raw(" | "),
        Span::raw(format!("THEME {}", app.theme_mode.label())),
        Span::raw(" | "),
        Span::raw(format!("LAST {update_time}")),
        Span::raw(" | "),
        Span::styled(format!("SYNC {spinner}"), sync_style),
        Span::raw(" | "),
        Span::styled(
            status,
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("MENU ", Style::default().fg(theme.dim)),
        Span::styled("[s]Sort ", Style::default().fg(theme.dim)),
        Span::styled("[/]Filter ", Style::default().fg(theme.dim)),
        Span::styled("[m]Cols ", Style::default().fg(theme.dim)),
        Span::styled("[C]Config ", Style::default().fg(theme.dim)),
        Span::styled("[W]Watch ", Style::default().fg(theme.dim)),
        Span::styled("[L]Legend ", Style::default().fg(theme.dim)),
        Span::styled("[?]Help", Style::default().fg(theme.dim)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("FEED");
    let paragraph = Paragraph::new(vec![line_top, line_bottom])
        .block(block)
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, area);
}

fn render_alerts(f: &mut Frame, area: Rect, app: &App, indices: &[usize]) {
    let theme = theme(app.theme_mode);
    let mut stale = 0usize;
    let mut no_pos = 0usize;
    let mut alert = 0usize;
    let mut spi = 0usize;
    let mut low_nic = 0usize;
    let mut low_nac = 0usize;
    let mut favs = 0usize;
    let mut near = 0usize;
    let mut route_err = 0usize;

    let site = app.site();
    let notify_radius = app.notify_radius_mi.max(0.0);
    for idx in indices {
        let ac = &app.data.aircraft[*idx];
        let seen = seen_seconds(ac).unwrap_or(f64::INFINITY);
        if seen > app.stale_secs {
            stale += 1;
        }
        if ac.lat.is_none() || ac.lon.is_none() {
            no_pos += 1;
        }
        if ac.alert.unwrap_or(0) > 0 {
            alert += 1;
        }
        if ac.spi.unwrap_or(0) > 0 {
            spi += 1;
        }
        if ac.nic.unwrap_or(99) < app.low_nic {
            low_nic += 1;
        }
        if ac.nac_p.unwrap_or(99) < app.low_nac {
            low_nac += 1;
        }
        if app.is_favorite(ac) {
            favs += 1;
        }
        if let (Some(site), Some(lat), Some(lon)) = (site, ac.lat, ac.lon) {
            if notify_radius > 0.0 {
                let dist_mi = distance_mi(site.lat, site.lon, lat, lon);
                if dist_mi <= notify_radius {
                    near += 1;
                }
            }
        }
    }

    if let Some((_, time)) = &app.route_error {
        if let Ok(delta) = SystemTime::now().duration_since(*time) {
            if delta.as_secs() <= 60 {
                route_err = 1;
            }
        }
    }

    let filter_text = if app.input_mode == InputMode::Filter {
        format!("/{}_", app.filter_edit)
    } else if app.filter.is_empty() {
        "none".to_string()
    } else {
        app.filter.clone()
    };

    let mut spans = Vec::new();
    spans.push(Span::styled(
        "ALERTS ",
        Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
    ));
    spans.extend(alert_span("STALE", stale, theme.danger));
    spans.extend(alert_span("NOPOS", no_pos, theme.warn));
    spans.extend(alert_span("ALERT", alert, theme.danger));
    spans.extend(alert_span("SPI", spi, theme.danger));
    spans.extend(alert_span("LOWNIC", low_nic, theme.warn));
    spans.extend(alert_span("LOWNAC", low_nac, theme.warn));
    spans.extend(alert_span("FAV", favs, theme.fav));
    let near_color = if phase_ms(800) {
        theme.accent
    } else {
        theme.warn
    };
    spans.extend(alert_span("NEAR", near, near_color));
    spans.extend(alert_span("RERR", route_err, theme.danger));

    let filter_style = if app.input_mode == InputMode::Filter {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dim)
    };

    spans.push(Span::raw("  "));
    spans.push(Span::styled("FILTER ", Style::default().fg(theme.dim)));
    spans.push(Span::styled(filter_text, filter_style));

    let paragraph = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, area);
}

fn alert_span(label: &str, count: usize, color: Color) -> Vec<Span<'static>> {
    let value_style = if count > 0 {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    vec![
        Span::styled(format!("{label} "), Style::default().fg(Color::DarkGray)),
        Span::styled(count.to_string(), value_style),
        Span::raw("  "),
    ]
}

fn render_stats(f: &mut Frame, area: Rect, app: &App, indices: &[usize]) {
    let theme = theme(app.theme_mode);
    let now = SystemTime::now();
    let visible = indices.len();
    let total = app.data.aircraft.len();
    let msg_rate_total = app.msg_rate_display();
    let msg_rate_avg = app.avg_aircraft_rate();
    let kbps_total = msg_rate_total.map(|rate| rate * 112.0 / 1000.0);
    let kbps_avg = msg_rate_avg.map(|rate| rate * 112.0 / 1000.0);
    let uptime = now
        .duration_since(app.start_time)
        .map(format_duration)
        .unwrap_or_else(|_| "--".to_string());
    let last_update = app
        .last_update
        .and_then(|t| now.duration_since(t).ok())
        .map(|d| format!("{}s", d.as_secs()))
        .unwrap_or_else(|| "--".to_string());
    let route_error = app
        .route_error
        .as_ref()
        .and_then(|(msg, when)| {
            now.duration_since(*when)
                .ok()
                .filter(|d| d.as_secs() <= 60)
                .map(|_| truncate(msg, 24))
        })
        .unwrap_or_else(|| "--".to_string());

    let mut last_1 = 0usize;
    let mut last_5 = 0usize;
    let mut last_15 = 0usize;
    for ac in &app.data.aircraft {
        if let Some(secs) = seen_seconds(ac) {
            let secs = secs.max(0.0) as u64;
            if secs <= 60 {
                last_1 += 1;
            }
            if secs <= 300 {
                last_5 += 1;
            }
            if secs <= 900 {
                last_15 += 1;
            }
        }
    }

    let ctx = StatsContext {
        visible,
        total,
        msg_total: app.data.messages.unwrap_or(0),
        msg_rate_total,
        msg_rate_avg,
        kbps_total,
        kbps_avg,
        last_1,
        last_5,
        last_15,
        uptime,
        last_update,
        route_error,
        site_alt: app
            .site()
            .map(|site| format!("{:.1} m", site.alt_m))
            .unwrap_or_else(|| "--".to_string()),
    };

    let mut lines = Vec::new();
    lines.push(stat_line("visible", &ctx, &theme, true));
    for key in &app.stats_metrics {
        lines.push(stat_line(key, &ctx, &theme, false));
    }
    lines.push(stat_line("seen_1_5_15", &ctx, &theme, false));
    lines.push(stat_line("uptime", &ctx, &theme, false));
    lines.push(stat_line("last_update", &ctx, &theme, false));
    lines.push(stat_line("route_err", &ctx, &theme, false));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title("STATS");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, area);
}

struct StatsContext {
    visible: usize,
    total: usize,
    msg_total: u64,
    msg_rate_total: Option<f64>,
    msg_rate_avg: Option<f64>,
    kbps_total: Option<f64>,
    kbps_avg: Option<f64>,
    last_1: usize,
    last_5: usize,
    last_15: usize,
    uptime: String,
    last_update: String,
    route_error: String,
    site_alt: String,
}

fn stat_line(key: &str, ctx: &StatsContext, theme: &Theme, emphasize: bool) -> Line<'static> {
    let key = key.trim().to_ascii_lowercase();
    let label = format!("{:<11}", stat_label(&key));
    let value = stat_value(&key, ctx);
    let label_style = if emphasize {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dim)
    };
    let value_style = if emphasize {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.accent)
    };
    Line::from(vec![
        Span::styled(label, label_style),
        Span::styled(value, value_style),
    ])
}

fn stat_label(key: &str) -> String {
    match key {
        "visible" => "VISIBLE".to_string(),
        "aircraft" => "AIRCRAFT".to_string(),
        "messages" => "MSGS".to_string(),
        "msg_rate_total" => "TOT MSG/S".to_string(),
        "msg_rate_avg" => "AVG MSG/S".to_string(),
        "kbps_total" => "TOT KBPS".to_string(),
        "kbps_avg" => "AVG KBPS".to_string(),
        "seen_1_5_15" => "SEEN 1/5/15".to_string(),
        "uptime" => "UPTIME".to_string(),
        "last_update" => "LAST UPD".to_string(),
        "site_alt" => "SITE ALT".to_string(),
        "route_err" => "ROUTE ERR".to_string(),
        _ => key.to_ascii_uppercase().replace('_', " "),
    }
}

fn stat_value(key: &str, ctx: &StatsContext) -> String {
    match key {
        "visible" => format!("{}/{}", ctx.visible, ctx.total),
        "aircraft" => ctx.total.to_string(),
        "messages" => ctx.msg_total.to_string(),
        "msg_rate_total" => fmt_rate(ctx.msg_rate_total),
        "msg_rate_avg" => fmt_rate(ctx.msg_rate_avg),
        "kbps_total" => fmt_kbps(ctx.kbps_total),
        "kbps_avg" => fmt_kbps(ctx.kbps_avg),
        "seen_1_5_15" => format!("{}/{}/{}", ctx.last_1, ctx.last_5, ctx.last_15),
        "uptime" => ctx.uptime.clone(),
        "last_update" => ctx.last_update.clone(),
        "site_alt" => ctx.site_alt.clone(),
        "route_err" => ctx.route_error.clone(),
        _ => "--".to_string(),
    }
}

fn fmt_rate(value: Option<f64>) -> String {
    match value {
        Some(rate) => format!("{rate:.1}/s"),
        None => "--".to_string(),
    }
}

fn fmt_kbps(value: Option<f64>) -> String {
    match value {
        Some(rate) => format!("{rate:.1}"),
        None => "--".to_string(),
    }
}

fn render_radar(f: &mut Frame, area: Rect, app: &App, indices: &[usize]) {
    let theme = theme(app.theme_mode);
    let radar_theme = RadarTheme {
        accent: theme.accent,
        dim: theme.dim,
        fav: theme.fav,
        warn: theme.warn,
        highlight: theme.highlight_bg,
        panel_bg: theme.panel_bg,
    };
    let settings = RadarSettings {
        range_nm: app.radar_range_nm,
        aspect: app.radar_aspect,
        renderer: app.radar_renderer,
        blip: app.radar_blip,
    };
    radar::render(f, area, app, indices, radar_theme, settings);
}

fn render_table(f: &mut Frame, area: Rect, app: &mut App, indices: &[usize]) {
    let theme = theme(app.theme_mode);
    app.set_table_area(area, 1);
    let available_width = area.width.saturating_sub(2);
    let columns = select_columns_for_width(app.columns(), available_width);
    if columns.is_empty() {
        return;
    }

    let now = SystemTime::now();
    let column_ids: Vec<ColumnId> = columns.iter().map(|col| col.id).collect();
    let widths = app
        .column_cache_lookup(available_width, &column_ids, indices.len(), now)
        .unwrap_or_else(|| compute_column_widths(app, &columns, indices, available_width));
    if app.column_cache_enabled {
        app.column_cache_store(
            available_width,
            column_ids,
            indices.len(),
            widths.clone(),
            now,
        );
    }
    let header_cells = columns.iter().zip(widths.iter()).map(|(col, width)| {
        let text = center_text(col.label, *width as usize);
        Cell::from(text).style(
            Style::default()
                .fg(theme.accent)
                .bg(theme.header_bg)
                .add_modifier(Modifier::BOLD),
        )
    });

    let header = Row::new(header_cells)
        .style(Style::default().bg(theme.header_bg))
        .height(1);

    let fresh_pulse = phase_ms(650);
    let stale_pulse = phase_ms(900);

    let rows = indices.iter().enumerate().map(|(i, idx)| {
        let ac = &app.data.aircraft[*idx];
        let seen = seen_seconds(ac);
        let stale = seen.map(|s| s > app.stale_secs).unwrap_or(true);
        let favorite = app.is_favorite(ac);
        let watchlisted = app.is_watchlisted(ac);
        let trend = app.trend_for(ac);
        let overpass = match (app.site(), ac.lat, ac.lon) {
            (Some(site), Some(lat), Some(lon)) => {
                distance_mi(site.lat, site.lon, lat, lon) <= app.overpass_mi
            }
            _ => false,
        };

        let mut style = if i % 2 == 0 {
            Style::default().bg(theme.row_even_bg)
        } else {
            Style::default().bg(theme.row_odd_bg)
        };

        if overpass {
            style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
        } else if seen.map(|s| s <= 1.0).unwrap_or(false) {
            style = if fresh_pulse {
                style
                    .fg(theme.accent)
                    .bg(theme.header_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                style.fg(theme.accent)
            };
        } else if stale {
            style = if stale_pulse {
                style.fg(theme.danger)
            } else {
                style.fg(theme.dim)
            };
        } else if watchlisted {
            style = style.fg(theme.watch).add_modifier(Modifier::BOLD);
        }

        let route = app.route_for(ac);
        let route_pending = route_pending_for(app, ac, route);
        let cells = columns.iter().zip(widths.iter()).map(|(col, width)| {
            cell_for_column(
                col.id,
                *width as usize,
                ac,
                favorite,
                watchlisted,
                seen,
                trend,
                route,
                route_pending,
                &theme,
                app.site(),
                app.altitude_trend_arrows,
                app.track_arrows,
            )
        });

        Row::new(cells).style(style)
    });

    let constraints: Vec<Constraint> = widths
        .iter()
        .map(|width| Constraint::Length(*width))
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title("AIRSPACE")
        .style(Style::default().bg(theme.panel_bg));

    let table = Table::new(rows, constraints)
        .header(header)
        .block(block)
        .column_spacing(1)
        .style(Style::default().bg(theme.panel_bg))
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg)
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_details(f: &mut Frame, area: Rect, app: &App, indices: &[usize]) {
    let theme = theme(app.theme_mode);
    let selected = app.table_state.selected().and_then(|row| indices.get(row));
    let lines = if let Some(idx) = selected {
        let ac = &app.data.aircraft[*idx];
        let flight = fit_str(ac.flight.as_deref(), 8).trim().to_string();
        let reg = ac.r.as_deref().unwrap_or("--");
        let hex = ac.hex.as_deref().unwrap_or("--");
        let ac_type = ac.t.as_deref().unwrap_or("--");
        let desc = ac.desc.as_deref().unwrap_or("--");
        let owner = ac.own_op.as_deref().unwrap_or("--");
        let year = ac.year.as_deref().unwrap_or("--");
        let alt_baro = fmt_i64(ac.alt_baro, 0);
        let alt_geom = fmt_i64(ac.alt_geom, 0);
        let gs = fmt_f64(ac.gs, 0, 0);
        let track = format_track_display(ac.track, app.track_arrows);
        let vs = fmt_i64(ac.baro_rate, 0);
        let qnh = fmt_f64(ac.nav_qnh, 0, 1);
        let mcp = fmt_i64(ac.nav_altitude_mcp, 0);
        let lat = fmt_f64(ac.lat, 0, 4);
        let lon = fmt_f64(ac.lon, 0, 4);
        let seen = fmt_f64(seen_seconds(ac), 0, 1);
        let msgs = fmt_u64(ac.messages, 0);
        let cat = ac.category.as_deref().unwrap_or("--");
        let nic = fmt_i64(ac.nic, 0);
        let nac_p = fmt_i64(ac.nac_p, 0);
        let nac_v = fmt_i64(ac.nac_v, 0);
        let sil = fmt_i64(ac.sil, 0);
        let rssi = fmt_f64(ac.rssi, 0, 1);
        let favorite = if app.is_favorite(ac) { "YES" } else { "NO" };
        let watch_text = if let Some(entry) = app.watch_entry_for(ac) {
            format!("YES {}", entry.entry_id())
        } else {
            "NO".to_string()
        };
        let route_info = app.route_for(ac);
        let route_pending = route_pending_for(app, ac, route_info);
        let route = if route_pending {
            route_pending_text().to_string()
        } else {
            route_info.map(route_display).unwrap_or("--".to_string())
        };
        let trail = app.trail_for(ac).unwrap_or(&[]);
        let trail_preview = trail
            .iter()
            .rev()
            .take(3)
            .map(|point| {
                format!(
                    "{} {:+.3},{:+.3}",
                    format_time_short(point.at),
                    point.lat,
                    point.lon
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        let (dist, brg) = match (app.site(), ac.lat, ac.lon) {
            (Some(site), Some(lat), Some(lon)) => (
                format!("{:.1} nm", distance_nm(site.lat, site.lon, lat, lon)),
                format!("{:.0}°", bearing_deg(site.lat, site.lon, lat, lon)),
            ),
            _ => ("--".to_string(), "--".to_string()),
        };

        vec![
            Line::from(vec![
                Span::styled("CALLSIGN ", Style::default().fg(theme.dim)),
                Span::styled(
                    flight,
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("REG      ", Style::default().fg(theme.dim)),
                Span::raw(reg),
                Span::raw("  "),
                Span::styled("HEX ", Style::default().fg(theme.dim)),
                Span::raw(hex),
            ]),
            Line::from(vec![
                Span::styled("TYPE     ", Style::default().fg(theme.dim)),
                Span::raw(ac_type),
            ]),
            Line::from(vec![
                Span::styled("DESC     ", Style::default().fg(theme.dim)),
                Span::raw(desc),
            ]),
            Line::from(vec![
                Span::styled("ROUTE    ", Style::default().fg(theme.dim)),
                Span::raw(route),
            ]),
            Line::from(vec![
                Span::styled("OPERATOR ", Style::default().fg(theme.dim)),
                Span::raw(owner),
            ]),
            Line::from(vec![
                Span::styled("YEAR     ", Style::default().fg(theme.dim)),
                Span::raw(year),
            ]),
            Line::from(vec![
                Span::styled("FAVORITE ", Style::default().fg(theme.dim)),
                Span::raw(favorite),
            ]),
            Line::from(vec![
                Span::styled("WATCH    ", Style::default().fg(theme.dim)),
                Span::raw(watch_text),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("ALT B/G  ", Style::default().fg(theme.dim)),
                Span::raw(format!("{alt_baro} / {alt_geom} ft")),
            ]),
            Line::from(vec![
                Span::styled("V/S      ", Style::default().fg(theme.dim)),
                Span::raw(format!("{vs} fpm")),
            ]),
            Line::from(vec![
                Span::styled("GS/TRK   ", Style::default().fg(theme.dim)),
                Span::raw(format!("{gs} kt / {track}")),
            ]),
            Line::from(vec![
                Span::styled("POS      ", Style::default().fg(theme.dim)),
                Span::raw(format!("{lat}, {lon}")),
            ]),
            Line::from(vec![
                Span::styled("DIST/BRG ", Style::default().fg(theme.dim)),
                Span::raw(format!("{dist} / {brg}")),
            ]),
            Line::from(vec![
                Span::styled("TRAILS   ", Style::default().fg(theme.dim)),
                Span::raw(format!("{} pts", trail.len())),
            ]),
            Line::from(vec![
                Span::styled("LAST POS ", Style::default().fg(theme.dim)),
                Span::raw(trail_preview),
            ]),
            Line::from(vec![
                Span::styled("QNH/MCP  ", Style::default().fg(theme.dim)),
                Span::raw(format!("{qnh} / {mcp} ft")),
            ]),
            Line::from(vec![
                Span::styled("SEEN     ", Style::default().fg(theme.dim)),
                Span::raw(format!("{seen} s")),
            ]),
            Line::from(vec![
                Span::styled("MSGS     ", Style::default().fg(theme.dim)),
                Span::raw(msgs),
            ]),
            Line::from(vec![
                Span::styled("CAT/NIC  ", Style::default().fg(theme.dim)),
                Span::raw(format!("{cat} / {nic}")),
            ]),
            Line::from(vec![
                Span::styled("NAC P/V  ", Style::default().fg(theme.dim)),
                Span::raw(format!("{nac_p} / {nac_v}")),
            ]),
            Line::from(vec![
                Span::styled("SIL/RSSI ", Style::default().fg(theme.dim)),
                Span::raw(format!("{sil} / {rssi} dB")),
            ]),
        ]
    } else {
        vec![Line::from("No aircraft selected.")]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("DETAILS");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let width = area.width.saturating_sub(40) as usize;
    let sweep_period_ms = 3500u64;
    let sweep_pos = if width == 0 {
        0
    } else {
        let ms = now_ms() % sweep_period_ms;
        ((ms as usize) * width) / sweep_period_ms as usize
    };
    let mut sweep = String::with_capacity(width);
    for i in 0..width {
        if i == sweep_pos {
            sweep.push('*');
        } else {
            sweep.push('.');
        }
    }

    let mut help = "q quit  s sort  / filter  f favorite  c clear  t theme  l layout  R radar  b labels  m columns  w watch  e export  C config  ? help".to_string();
    let source = short_source(&app.url);
    help.push_str(&format!("  REF {}s  SRC {}", app.refresh.as_secs(), source));

    let mut spans = vec![Span::styled(help, Style::default().fg(theme.dim))];
    if let Some(note) = app.latest_notification() {
        if let Ok(delta) = SystemTime::now().duration_since(note.at) {
            if delta <= Duration::from_secs(8) {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    format!("ALERT {}", note.message),
                    Style::default().fg(theme.warn).add_modifier(Modifier::BOLD),
                ));
            }
        }
    }
    if let Some((name, when)) = &app.last_export {
        if let Ok(delta) = SystemTime::now().duration_since(*when) {
            if delta <= Duration::from_secs(6) {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    format!("SAVED {}", name),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ));
            }
        }
    }
    spans.push(Span::raw("  "));
    spans.push(Span::styled("RADAR ", Style::default().fg(theme.accent)));
    spans.push(Span::styled(sweep, Style::default().fg(theme.dim)));
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, area);
}

fn render_columns_menu(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let columns = app.columns();
    let height = (columns.len() + 4).min(20) as u16;
    let popup = centered_rect(50, height, area);

    f.render_widget(Clear, popup);

    let mut lines = Vec::new();
    for (i, col) in columns.iter().enumerate() {
        let marker = if col.visible { "[x]" } else { "[ ]" };
        let text = format!(" {marker} {}", column_name(col.id));
        let line = if i == app.column_cursor() {
            Line::from(Span::styled(
                text,
                Style::default()
                    .fg(theme.highlight_fg)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(text, Style::default().fg(theme.dim)))
        };
        lines.push(line);
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Up/Down move • Space toggle • Esc close",
        Style::default().fg(theme.dim),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("COLUMNS");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, popup);
}

fn render_help_menu(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let popup = centered_rect(70, 20, area);

    f.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            "HELP",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Navigation",
            Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
        )),
        Line::from("  ↑/↓        Move selection"),
        Line::from("  Mouse      Scroll to move • Click row to select"),
        Line::from(""),
        Line::from(Span::styled(
            "Display",
            Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
        )),
        Line::from("  s          Sort (SEEN/ALT/SPD)"),
        Line::from("  l          Toggle layout (full/compact)"),
        Line::from("  R          Radar layout"),
        Line::from("  b          Toggle radar labels"),
        Line::from("  t          Toggle theme"),
        Line::from("  m          Columns menu"),
        Line::from("  w          Watchlist"),
        Line::from(""),
        Line::from(Span::styled(
            "Filter & Favorites",
            Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
        )),
        Line::from("  /          Filter (Enter apply, Esc cancel, Ctrl+U clear)"),
        Line::from("  c          Clear filter"),
        Line::from("  f          Toggle favorite (auto-saves)"),
        Line::from(""),
        Line::from(Span::styled(
            "Export & Config",
            Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
        )),
        Line::from("  e / E      Export CSV / JSON"),
        Line::from("  C          Config editor"),
        Line::from("  W          Watchlist menu"),
        Line::from(""),
        Line::from(Span::styled(
            "Quit",
            Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
        )),
        Line::from("  q          Quit"),
        Line::from("  ? / h      Toggle help"),
        Line::from("  L          Legend"),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc to close",
            Style::default().fg(theme.dim),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("HELP");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, popup);
}

fn render_legend_menu(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let popup = centered_rect(70, 22, area);
    f.render_widget(Clear, popup);

    let legend_items = legend_items();
    let total_items = legend_items.len();
    let reserved = 5;
    let items_height = popup.height.saturating_sub(reserved).max(1) as usize;
    let mut start = if total_items > items_height {
        app.config_cursor.saturating_sub(items_height / 2)
    } else {
        0
    };
    if start + items_height > total_items {
        start = total_items.saturating_sub(items_height);
    }
    let end = (start + items_height).min(total_items);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "LEGEND",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (i, item) in legend_items.iter().enumerate().take(end).skip(start) {
        let line = if i == app.config_cursor {
            Line::from(Span::styled(
                *item,
                Style::default()
                    .fg(theme.highlight_fg)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(*item, Style::default().fg(theme.dim)))
        };
        lines.push(line);
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "Up/Down scroll • L or Esc close  {}-{} / {}",
            if total_items == 0 { 0 } else { start + 1 },
            end,
            total_items
        ),
        Style::default().fg(theme.dim),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("LEGEND");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, popup);
}

fn render_config_menu(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let height = (app.config_items.len() + 6).min(24) as u16;
    let popup = centered_rect(72, height, area);

    f.render_widget(Clear, popup);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("CONFIG {}", app.config_path.display()),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let status_visible = app
        .config_status
        .as_ref()
        .and_then(|(_, when)| SystemTime::now().duration_since(*when).ok())
        .map(|d| d.as_secs() <= 5)
        .unwrap_or(false);
    let reserved = 4 + if status_visible { 1 } else { 0 };
    let items_height = popup.height.saturating_sub(reserved).max(1) as usize;
    let total_items = app.config_items.len();
    let mut start = if total_items > items_height {
        app.config_cursor.saturating_sub(items_height / 2)
    } else {
        0
    };
    if start + items_height > total_items {
        start = total_items.saturating_sub(items_height);
    }
    let end = (start + items_height).min(total_items);
    let key_width = app
        .config_items
        .iter()
        .skip(start)
        .take(items_height)
        .map(|item| item.key.len())
        .max()
        .unwrap_or(8)
        .min(24);

    for (i, item) in app.config_items.iter().enumerate().take(end).skip(start) {
        let mut value = item.value.clone();
        if app.config_editing && i == app.config_cursor {
            value = format!("{}_", app.config_edit);
        }
        let text = format!("{:width$} = {}", item.key, value, width = key_width);
        let line = if i == app.config_cursor {
            Line::from(Span::styled(
                text,
                Style::default()
                    .fg(theme.highlight_fg)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(text, Style::default().fg(theme.dim)))
        };
        lines.push(line);
    }

    lines.push(Line::from(""));
    if let Some((msg, _)) = &app.config_status {
        if status_visible {
            lines.push(Line::from(Span::styled(
                msg,
                Style::default().fg(theme.warn),
            )));
        }
    }
    lines.push(Line::from(Span::styled(
        format!(
            "Up/Down select • Enter edit/apply • w save • Esc close  {}-{} / {}",
            if total_items == 0 { 0 } else { start + 1 },
            end,
            total_items
        ),
        Style::default().fg(theme.dim),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("CONFIG");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, popup);
}

fn render_watchlist_menu(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let total_items = app.watchlist_len();
    let height = (total_items.max(1) + 6).min(24) as u16;
    let popup = centered_rect(72, height, area);

    f.render_widget(Clear, popup);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "WATCHLIST",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));
    let path_text = app
        .watchlist_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "--".to_string());
    lines.push(Line::from(Span::styled(
        format!("FILE {path_text}"),
        Style::default().fg(theme.dim),
    )));
    lines.push(Line::from(""));

    let reserved = 4;
    let items_height = popup.height.saturating_sub(reserved).max(1) as usize;
    let mut start = if total_items > items_height {
        app.watchlist_cursor.saturating_sub(items_height / 2)
    } else {
        0
    };
    if start + items_height > total_items {
        start = total_items.saturating_sub(items_height);
    }
    let end = (start + items_height).min(total_items);

    if total_items == 0 {
        lines.push(Line::from(Span::styled(
            "No watchlist entries.",
            Style::default().fg(theme.dim),
        )));
    } else {
        for (i, entry) in app.watchlist.iter().enumerate().take(end).skip(start) {
            let enabled = if entry.is_enabled() { "ON " } else { "OFF" };
            let notify = if entry.notify_enabled() { "N" } else { "-" };
            let entry_id = entry.entry_id();
            let label = entry
                .label
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(entry_id.as_str());
            let mode = entry.match_mode();
            let match_text = format!("{}={}", entry.match_type, entry.value);
            let text = format!(
                "{enabled} {notify}  {:<18}  {}  ({})",
                truncate(label, 18),
                truncate(&match_text, 28),
                mode
            );
            let line = if i == app.watchlist_cursor {
                Line::from(Span::styled(
                    text,
                    Style::default()
                        .fg(theme.highlight_fg)
                        .bg(theme.highlight_bg)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(text, Style::default().fg(theme.dim)))
            };
            lines.push(line);
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "e toggle enabled • n toggle notify • d delete • s save",
        Style::default().fg(theme.dim),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "Up/Down select • Esc close  {}-{} / {}",
            if total_items == 0 { 0 } else { start + 1 },
            end,
            total_items
        ),
        Style::default().fg(theme.dim),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("WATCHLIST");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, popup);
}

fn legend_items() -> Vec<&'static str> {
    vec![
        "Columns:",
        "  FLIGHT   Callsign (may be blank)",
        "  REG      Registration",
        "  TYPE     Aircraft type",
        "  ROUTE    Route (if available)",
        "  ALT      Baro altitude (trend arrows)",
        "  GS       Ground speed (kt)",
        "  TRK      Track/heading (deg + arrow)",
        "  LAT/LON  Position",
        "  DIST     Distance from site (nm)",
        "  BRG      Bearing from site (deg)",
        "  SEEN     Seconds since last seen",
        "  MSGS     Per‑aircraft message count",
        "  HEX      ICAO hex",
        "  W        Watchlist match",
        "Alerts:",
        "  STALE    Seen > stale_secs",
        "  NOPOS    Missing position",
        "  ALERT    Emergency flag",
        "  SPI      Special Position ID",
        "  LOWNIC   NIC below threshold",
        "  LOWNAC   NACp below threshold",
        "  FAV      Favorited aircraft",
        "  NEAR     Within notify_radius_mi",
        "  RERR     Route lookup error (recent)",
        "Stats:",
        "  MSG RATE Receiver msg/s (smoothed)",
        "  EST KBPS Estimated kbps (approx)",
        "Radar:",
        "  Canvas   Braille blips with sweep arm",
        "  * / o    ASCII fallback current/trail",
        "  F / f    ASCII favorite current/trail",
        "  X        Selected target (radar view)",
    ]
}

pub fn legend_len() -> usize {
    legend_items().len()
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let height = height.min(area.height.saturating_sub(2)).max(3);
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(height),
            Constraint::Min(1),
        ])
        .split(area);
    let vertical = popup_layout[1];
    let width = (vertical.width * percent_x / 100).max(20);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(width),
            Constraint::Min(1),
        ])
        .split(vertical);
    horizontal[1]
}

fn fit_str(value: Option<&str>, width: usize) -> String {
    let mut text = value.unwrap_or("--").trim().to_string();
    if text.len() > width {
        text.truncate(width);
    }
    format!("{text:width$}", width = width)
}

fn fmt_i64(value: Option<i64>, width: usize) -> String {
    match value {
        Some(v) if width > 0 => format!("{v:>width$}", width = width),
        Some(v) => v.to_string(),
        None if width > 0 => format!("{:>width$}", "--", width = width),
        None => "--".to_string(),
    }
}

fn fmt_u64(value: Option<u64>, width: usize) -> String {
    match value {
        Some(v) if width > 0 => format!("{v:>width$}", width = width),
        Some(v) => v.to_string(),
        None if width > 0 => format!("{:>width$}", "--", width = width),
        None => "--".to_string(),
    }
}

fn fmt_f64(value: Option<f64>, width: usize, precision: usize) -> String {
    match value {
        Some(v) if width > 0 => {
            format!(
                "{v:>width$.precision$}",
                width = width,
                precision = precision
            )
        }
        Some(v) => format!("{v:.precision$}", precision = precision),
        None if width > 0 => format!("{:>width$}", "--", width = width),
        None => "--".to_string(),
    }
}

fn fmt_i64_trend(value: Option<i64>, trend: TrendDir, show_trend: bool, width: usize) -> String {
    let base = match value {
        Some(v) => v.to_string(),
        None => "--".to_string(),
    };
    let arrow = if show_trend { trend_char(trend) } else { ' ' };
    let text = format!("{base}{arrow}");
    if width > 0 {
        format!("{text:>width$}", width = width)
    } else {
        text
    }
}

fn fmt_f64_trend(value: Option<f64>, trend: TrendDir, width: usize, precision: usize) -> String {
    let base = match value {
        Some(v) => format!("{v:.precision$}", precision = precision),
        None => "--".to_string(),
    };
    let arrow = trend_char(trend);
    let text = format!("{base}{arrow}");
    if width > 0 {
        format!("{text:>width$}", width = width)
    } else {
        text
    }
}

fn trend_char(trend: TrendDir) -> char {
    match trend {
        TrendDir::Up => '↑',
        TrendDir::Down => '↓',
        TrendDir::Flat => '→',
        TrendDir::Unknown => ' ',
    }
}

fn fmt_distance(site: Option<SiteLocation>, ac: &crate::model::Aircraft, width: usize) -> String {
    let value = match (site, ac.lat, ac.lon) {
        (Some(site), Some(lat), Some(lon)) => Some(distance_nm(site.lat, site.lon, lat, lon)),
        _ => None,
    };
    match value {
        Some(v) if width > 0 => format!("{v:>width$.1}", width = width),
        Some(v) => format!("{v:.1}"),
        None if width > 0 => format!("{:>width$}", "--", width = width),
        None => "--".to_string(),
    }
}

fn fmt_bearing(site: Option<SiteLocation>, ac: &crate::model::Aircraft, width: usize) -> String {
    let value = match (site, ac.lat, ac.lon) {
        (Some(site), Some(lat), Some(lon)) => Some(bearing_deg(site.lat, site.lon, lat, lon)),
        _ => None,
    };
    match value {
        Some(v) if width > 0 => format!("{v:>width$.0}", width = width),
        Some(v) => format!("{v:.0}"),
        None if width > 0 => format!("{:>width$}", "--", width = width),
        None => "--".to_string(),
    }
}

fn format_track_cell(track: Option<f64>, arrows: bool) -> String {
    format_track(track, false, arrows)
}

fn format_track_display(track: Option<f64>, arrows: bool) -> String {
    format_track(track, true, arrows)
}

fn format_track(track: Option<f64>, with_degree: bool, arrows: bool) -> String {
    let Some(track) = track else {
        return "--".to_string();
    };
    let deg = track.rem_euclid(360.0);
    let arrow = if arrows { track_arrow(deg) } else { "" };
    if with_degree {
        format!("{deg:03.0}°{arrow}")
    } else {
        format!("{deg:03.0}{arrow}")
    }
}

fn track_arrow(deg: f64) -> &'static str {
    let idx = ((deg + 22.5) / 45.0).floor() as i32 % 8;
    match idx {
        0 => "↑",
        1 => "↗",
        2 => "→",
        3 => "↘",
        4 => "↓",
        5 => "↙",
        6 => "←",
        _ => "↖",
    }
}

fn route_display(route: &crate::app::RouteInfo) -> String {
    match (&route.origin, &route.destination) {
        (Some(o), Some(d)) => format!("{o}-{d}"),
        _ => route.route.clone().unwrap_or_else(|| "--".to_string()),
    }
}

fn distance_nm(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_nm = 3440.065_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r_nm * c
}

fn distance_mi(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    distance_nm(lat1, lon1, lat2, lon2) * 1.15078
}

fn fmt_text(value: Option<&str>) -> String {
    let text = value.unwrap_or("--").trim();
    if text.is_empty() {
        "--".to_string()
    } else {
        text.to_string()
    }
}

fn truncate_to_width(mut value: String, width: usize) -> String {
    if width == 0 {
        return value;
    }
    if text_len(&value) > width {
        value = value.chars().take(width).collect();
    }
    value
}

fn center_text(value: &str, width: usize) -> String {
    if width == 0 {
        return value.to_string();
    }
    let len = text_len(value);
    if len >= width {
        return value.to_string();
    }
    let pad_left = (width - len) / 2;
    let pad_right = width - len - pad_left;
    let mut out = String::with_capacity(width);
    for _ in 0..pad_left {
        out.push(' ');
    }
    out.push_str(value);
    for _ in 0..pad_right {
        out.push(' ');
    }
    out
}

fn text_len(value: &str) -> usize {
    value.chars().count()
}

fn select_columns_for_width(
    columns: &[crate::app::ColumnConfig],
    available_width: u16,
) -> Vec<crate::app::ColumnConfig> {
    let mut cols: Vec<crate::app::ColumnConfig> =
        columns.iter().filter(|col| col.visible).cloned().collect();
    if cols.is_empty() {
        return cols;
    }

    let mut total_min = columns_min_width(&cols);
    let drop_order = [
        ColumnId::Brg,
        ColumnId::Lat,
        ColumnId::Lon,
        ColumnId::Route,
        ColumnId::Reg,
        ColumnId::Hex,
        ColumnId::Msgs,
        ColumnId::Dist,
        ColumnId::Trk,
        ColumnId::Gs,
        ColumnId::Alt,
        ColumnId::Type,
        ColumnId::Flight,
        ColumnId::Watch,
        ColumnId::Fav,
        ColumnId::Flag,
    ];

    for id in drop_order {
        if total_min <= available_width || cols.len() <= 3 {
            break;
        }
        if let Some(pos) = cols.iter().position(|c| c.id == id) {
            cols.remove(pos);
            total_min = columns_min_width(&cols);
        }
    }

    cols
}

fn columns_min_width(columns: &[crate::app::ColumnConfig]) -> u16 {
    let spacing = columns.len().saturating_sub(1) as u16;
    let sum: u16 = columns.iter().map(|c| c.width).sum();
    sum.saturating_add(spacing)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn phase_ms(period_ms: u64) -> bool {
    if period_ms == 0 {
        return true;
    }
    (now_ms() / period_ms) % 2 == 0
}

fn phase_index(period_ms: u64, frames: usize) -> usize {
    if frames == 0 || period_ms == 0 {
        return 0;
    }
    ((now_ms() / period_ms) as usize) % frames
}

fn route_pending_for(
    app: &App,
    ac: &crate::model::Aircraft,
    route: Option<&crate::app::RouteInfo>,
) -> bool {
    if !app.route_enabled() || route.is_some() {
        return false;
    }
    let callsign = ac.flight.as_deref().map(str::trim).unwrap_or("");
    if callsign.is_empty() {
        return false;
    }
    app.route_pending(callsign, SystemTime::now())
}

fn route_pending_text() -> &'static str {
    const FRAMES: [&str; 4] = [".", "..", "...", ".."];
    FRAMES[phase_index(350, FRAMES.len())]
}

fn compute_column_widths(
    app: &App,
    columns: &[crate::app::ColumnConfig],
    indices: &[usize],
    available_width: u16,
) -> Vec<u16> {
    if columns.is_empty() {
        return Vec::new();
    }

    let spacing = columns.len().saturating_sub(1);
    let available = available_width.saturating_sub(spacing as u16) as isize;
    if available <= 0 {
        return vec![1; columns.len()];
    }

    let mut desired: Vec<usize> = columns.iter().map(|col| col.width as usize).collect();

    for (i, col) in columns.iter().enumerate() {
        desired[i] = desired[i].max(text_len(col.label));
    }

    let sample_limit = indices.len().min(50);
    for idx in indices.iter().take(sample_limit) {
        let ac = &app.data.aircraft[*idx];
        let trend = app.trend_for(ac);
        let route = app.route_for(ac);
        let route_pending = route_pending_for(app, ac, route);
        for (i, col) in columns.iter().enumerate() {
            let value = match col.id {
                ColumnId::Fav => {
                    if app.is_favorite(ac) {
                        "*".to_string()
                    } else {
                        " ".to_string()
                    }
                }
                ColumnId::Watch => {
                    if app.is_watchlisted(ac) {
                        "W".to_string()
                    } else {
                        " ".to_string()
                    }
                }
                ColumnId::Flight => fmt_text(ac.flight.as_deref()),
                ColumnId::Reg => fmt_text(ac.r.as_deref()),
                ColumnId::Type => fmt_text(ac.t.as_deref()),
                ColumnId::Route => {
                    if route_pending {
                        "...".to_string()
                    } else {
                        route.map(route_display).unwrap_or_else(|| "--".to_string())
                    }
                }
                ColumnId::Alt => {
                    fmt_i64_trend(ac.alt_baro, trend.alt, app.altitude_trend_arrows, 0)
                }
                ColumnId::Gs => fmt_f64_trend(ac.gs, trend.gs, 0, 0),
                ColumnId::Trk => format_track_cell(ac.track, app.track_arrows),
                ColumnId::Lat => fmt_f64(ac.lat, 0, 2),
                ColumnId::Lon => fmt_f64(ac.lon, 0, 2),
                ColumnId::Dist => fmt_distance(app.site(), ac, 0),
                ColumnId::Brg => fmt_bearing(app.site(), ac, 0),
                ColumnId::Seen => fmt_f64(seen_seconds(ac), 0, 0),
                ColumnId::Msgs => fmt_u64(ac.messages, 0),
                ColumnId::Hex => fmt_text(ac.hex.as_deref()),
                ColumnId::Flag => get_flag(ac.r.as_deref()).to_string(),
            };
            desired[i] = desired[i].max(text_len(&value));
        }
    }

    let mut widths = desired;
    let mut sum = widths.iter().sum::<usize>() as isize;
    let min_widths: Vec<usize> = columns.iter().map(|c| c.width as usize).collect();
    let min_sum = min_widths.iter().sum::<usize>() as isize;

    if min_sum > available {
        let count = columns.len().max(1);
        let base = (available as usize / count).max(1);
        let mut out = vec![base; count];
        let mut remainder = available as usize - base * count;
        for w in &mut out {
            if remainder == 0 {
                break;
            }
            *w += 1;
            remainder -= 1;
        }
        return out.into_iter().map(|w| w as u16).collect();
    }

    if sum > available {
        while sum > available {
            if let Some((idx, _)) = widths
                .iter()
                .enumerate()
                .filter(|(i, w)| **w > min_widths[*i])
                .max_by_key(|(_, w)| *w)
            {
                widths[idx] = widths[idx].saturating_sub(1);
                sum -= 1;
            } else {
                break;
            }
        }
    } else if sum < available {
        let extra = (available - sum) as usize;
        let count = columns.len().max(1);
        let add_each = extra / count;
        let mut remainder = extra % count;
        for w in &mut widths {
            *w += add_each;
            if remainder > 0 {
                *w += 1;
                remainder -= 1;
            }
        }
    }

    widths.into_iter().map(|w| w as u16).collect()
}

fn bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let mut brg = y.atan2(x).to_degrees();
    if brg < 0.0 {
        brg += 360.0;
    }
    brg
}

#[allow(clippy::too_many_arguments)]
fn cell_for_column(
    id: ColumnId,
    width: usize,
    ac: &crate::model::Aircraft,
    favorite: bool,
    watchlisted: bool,
    seen: Option<f64>,
    trend: crate::app::Trend,
    route: Option<&crate::app::RouteInfo>,
    route_pending: bool,
    theme: &Theme,
    site: Option<SiteLocation>,
    altitude_trend_arrows: bool,
    track_arrows: bool,
) -> Cell<'static> {
    let mut text = match id {
        ColumnId::Fav => {
            if favorite {
                "*".to_string()
            } else {
                " ".to_string()
            }
        }
        ColumnId::Watch => {
            if watchlisted {
                "W".to_string()
            } else {
                " ".to_string()
            }
        }
        ColumnId::Flight => fmt_text(ac.flight.as_deref()),
        ColumnId::Reg => fmt_text(ac.r.as_deref()),
        ColumnId::Type => fmt_text(ac.t.as_deref()),
        ColumnId::Route => {
            if route_pending {
                route_pending_text().to_string()
            } else {
                route.map(route_display).unwrap_or_else(|| "--".to_string())
            }
        }
        ColumnId::Alt => fmt_i64_trend(ac.alt_baro, trend.alt, altitude_trend_arrows, 0),
        ColumnId::Gs => fmt_f64_trend(ac.gs, trend.gs, 0, 0),
        ColumnId::Trk => format_track_cell(ac.track, track_arrows),
        ColumnId::Lat => fmt_f64(ac.lat, 0, 2),
        ColumnId::Lon => fmt_f64(ac.lon, 0, 2),
        ColumnId::Dist => fmt_distance(site, ac, 0),
        ColumnId::Brg => fmt_bearing(site, ac, 0),
        ColumnId::Seen => fmt_f64(seen, 0, 0),
        ColumnId::Msgs => fmt_u64(ac.messages, 0),
        ColumnId::Hex => fmt_text(ac.hex.as_deref()),
        ColumnId::Flag => get_flag(ac.r.as_deref()).to_string(),
    };

    text = truncate_to_width(text, width);
    let text = center_text(&text, width);

    if id == ColumnId::Fav && favorite {
        Cell::from(text).style(Style::default().fg(theme.fav).add_modifier(Modifier::BOLD))
    } else if id == ColumnId::Watch && watchlisted {
        Cell::from(text).style(Style::default().fg(theme.watch).add_modifier(Modifier::BOLD))
    } else {
        Cell::from(text)
    }
}

fn format_epoch(ts: i64) -> Option<String> {
    let dt: DateTime<Utc> = DateTime::from_timestamp(ts, 0)?;
    Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn format_system_time(time: SystemTime) -> String {
    let dt: DateTime<Local> = time.into();
    dt.format("%H:%M:%S").to_string()
}

fn format_time_short(time: SystemTime) -> String {
    let dt: DateTime<Local> = time.into();
    dt.format("%H:%M:%S").to_string()
}

fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn short_source(url: &str) -> String {
    let mut text = url.trim().to_string();
    if let Some(pos) = text.find("://") {
        text = text[(pos + 3)..].to_string();
    }
    if let Some(pos) = text.find('/') {
        text.truncate(pos);
    }
    if text.len() > 24 {
        text.truncate(24);
    }
    if text.is_empty() {
        "--".to_string()
    } else {
        text
    }
}

fn truncate(value: &str, max: usize) -> String {
    if text_len(value) <= max {
        value.to_string()
    } else if max <= 3 {
        value.chars().take(max).collect()
    } else {
        let head: String = value.chars().take(max - 3).collect();
        format!("{head}...")
    }
}

fn column_name(id: ColumnId) -> &'static str {
    match id {
        ColumnId::Fav => "FAVORITE",
        ColumnId::Watch => "WATCHLIST",
        ColumnId::Flight => "FLIGHT",
        ColumnId::Reg => "REG",
        ColumnId::Type => "TYPE",
        ColumnId::Route => "ROUTE",
        ColumnId::Alt => "ALTITUDE",
        ColumnId::Gs => "GROUND SPD",
        ColumnId::Trk => "TRACK",
        ColumnId::Lat => "LATITUDE",
        ColumnId::Lon => "LONGITUDE",
        ColumnId::Dist => "DISTANCE",
        ColumnId::Brg => "BEARING",
        ColumnId::Seen => "SEEN",
        ColumnId::Msgs => "MESSAGES",
        ColumnId::Hex => "HEX",
        ColumnId::Flag => "FLAG",
    }
}

fn get_flag(registration: Option<&str>) -> &'static str {
    if let Some(reg) = registration {
        if reg.is_empty() {
            return "🏳️";
        }
        let two_char = if reg.len() >= 2 { &reg[0..2] } else { "" };
        let one_char = &reg[0..1];

        match two_char {
            // Two-letter prefixes (letter-based)
            "PH" => "🇳🇱",
            "SP" => "🇵🇱",
            "SU" => "🇪🇬",
            "SX" => "🇬🇷",
            "TF" => "🇮🇸",
            "TG" => "🇬🇹",
            "TI" => "🇨🇷",
            "YR" => "🇷🇴",
            "YU" => "🇷🇸",
            "YV" => "🇻🇪",
            "ZA" => "🇦🇱",
            "ZB" => "🇬🇮",
            "ZS" => "🇿🇦",
            "ZP" => "🇵🇾",
            "ZM" => "🇲🇳",
            "YA" => "🇦🇫",
            "YB" => "🇮🇩",
            "YI" => "🇮🇶",
            "YL" => "🇱🇻",
            "YN" => "🇳🇮",
            "YS" => "🇸🇻",
            "VT" => "🇮🇳",
            "VN" => "🇻🇳",
            "VR" => "🇸🇨",
            "CU" => "🇨🇺",
            "TX" => "🇧🇲",
            "ZF" => "🇰🇾",
            "ZI" => "🇰🇵",
            "4K" => "🇦🇿",
            "4L" => "🇬🇪",
            "5A" => "🇱🇾",
            "5B" => "🇨🇾",
            "5R" => "🇲🇬",
            "5T" => "🇲🇷",
            "5U" => "🇳🇪",
            "5V" => "🇹🇬",
            "5W" => "🇼🇸",
            "5X" => "🇺🇬",
            "5Y" => "🇰🇪",
            "5Z" => "🇹🇿",
            "6O" => "🇸🇴",
            "6V" => "🇸🇳",
            "6Y" => "🇯🇲",
            "7O" => "🇾🇪",
            "7P" => "🇱🇸",
            "7Q" => "🇲🇼",
            "7R" => "🇩🇿",
            "8P" => "🇧🇧",
            "8Q" => "🇲🇻",
            "8R" => "🇬🇾",
            "9A" => "🇭🇷",
            "9G" => "🇬🇭",
            "9J" => "🇿🇲",
            "9K" => "🇰🇼",
            "9L" => "🇸🇱",
            "9M" => "🇲🇾",
            "9N" => "🇳🇵",
            "9Q" => "🇨🇩",
            "9S" => "🇸🇹",
            "9U" => "🇧🇮",
            "9V" => "🇸🇬",
            "9X" => "🇷🇼",
            "9Y" => "🇹🇹",
            "HB" => "🇨🇭",
            "4X" => "🇮🇱",
            "A6" => "🇦🇪",
            "AP" => "🇵🇰",
            "C5" => "🇬🇲",
            "CR" => "🇵🇹",
            "D2" => "🇦🇴",
            "E3" => "🇪🇷",
            "E4" => "🇧🇭",
            "EC" => "🇪🇸",
            "EI" => "🇮🇪",
            "EP" => "🇮🇷",
            "ET" => "🇪🇹",
            "HA" => "🇭🇺",
            "HL" => "🇰🇷",
            "HS" => "🇹🇭",
            "JA" => "🇯🇵",
            "JY" => "🇯🇴",
            "LN" => "🇳🇴",
            "LZ" => "🇧🇬",
            "LY" => "🇱🇹",
            "OD" => "🇱🇧",
            "OE" => "🇦🇹",
            "OH" => "🇫🇮",
            "OK" => "🇨🇿",
            "OO" => "🇧🇪",
            "OY" => "🇩🇰",
            "P4" => "🇦🇼",
            "PK" => "🇵🇰",
            "S5" => "🇸🇮",
            "S9" => "🇸🇦",
            "ST" => "🇸🇩",
            "T7" => "🇸🇲",
            "T9" => "🇧🇴",
            "UR" => "🇺🇦",
            "V2" => "🇦🇬",
            "V3" => "🇧🇿",
            "V5" => "🇳🇦",
            "V6" => "🇲🇭",
            "V7" => "🇫🇯",
            "XA" => "🇲🇽",
            "XT" => "🇧🇼",
            "YJ" => "🇻🇺",
            "YK" => "🇸🇾",
            "Z3" => "🇲🇰",
            "Z8" => "🇸🇸",
            // Fallback to single-letter prefixes
            _ => match one_char {
                "A" => "🇦🇺",
                "B" => "🇧🇪",
                "C" => "🇨🇦",
                "D" => "🇩🇪",
                "E" => "🇪🇸",
                "F" => "🇫🇷",
                "G" => "🇬🇧",
                "H" => "🇭🇺",
                "I" => "🇮🇹",
                "J" => "🇯🇵",
                "K" => "🇰🇷",
                "L" => "🇱🇺",
                "M" => "🇲🇽",
                "N" => "🇺🇸",
                "O" => "🇦🇹",
                "P" => "🇧🇷",
                "Q" => "🇶🇦",
                "R" => "🇷🇺",
                "S" => "🇸🇪",
                "T" => "🇹🇷",
                "U" => "🇺🇦",
                "V" => "🇻🇳",
                "W" => "🇹🇼",
                "X" => "🇨🇳",
                "Y" => "🇳🇴",
                "Z" => "🇳🇿",
                _ => "🏳️",
            }
        }
    } else {
        "🏳️"
    }
}

fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Default => Theme {
            accent: Color::Yellow,
            warn: Color::Yellow,
            danger: Color::Red,
            dim: Color::DarkGray,
            highlight_fg: Color::Black,
            highlight_bg: Color::Rgb(200, 200, 200),
            fav: Color::Yellow,
            watch: Color::LightBlue,
            row_even_bg: Color::Rgb(20, 20, 24),
            row_odd_bg: Color::Rgb(12, 12, 16),
            header_bg: Color::Rgb(24, 24, 28),
            panel_bg: Color::Rgb(18, 18, 22),
        },
        ThemeMode::ColorBlind => Theme {
            accent: Color::Cyan,
            warn: Color::LightCyan,
            danger: Color::LightRed,
            dim: Color::DarkGray,
            highlight_fg: Color::Black,
            highlight_bg: Color::LightCyan,
            fav: Color::LightCyan,
            watch: Color::Yellow,
            row_even_bg: Color::Rgb(16, 22, 26),
            row_odd_bg: Color::Rgb(10, 16, 20),
            header_bg: Color::Rgb(20, 26, 30),
            panel_bg: Color::Rgb(14, 20, 24),
        },
        ThemeMode::Amber => Theme {
            accent: Color::Rgb(255, 191, 0),
            warn: Color::Rgb(255, 220, 120),
            danger: Color::LightRed,
            dim: Color::Rgb(140, 110, 40),
            highlight_fg: Color::Black,
            highlight_bg: Color::Rgb(255, 220, 120),
            fav: Color::Rgb(255, 191, 0),
            watch: Color::LightBlue,
            row_even_bg: Color::Rgb(28, 22, 12),
            row_odd_bg: Color::Rgb(20, 16, 10),
            header_bg: Color::Rgb(32, 24, 14),
            panel_bg: Color::Rgb(24, 18, 10),
        },
        ThemeMode::Ocean => Theme {
            accent: Color::Rgb(0, 200, 220),
            warn: Color::LightBlue,
            danger: Color::LightRed,
            dim: Color::Rgb(80, 120, 130),
            highlight_fg: Color::Black,
            highlight_bg: Color::Rgb(0, 200, 220),
            fav: Color::Rgb(0, 200, 220),
            watch: Color::LightYellow,
            row_even_bg: Color::Rgb(10, 20, 26),
            row_odd_bg: Color::Rgb(8, 16, 22),
            header_bg: Color::Rgb(12, 24, 30),
            panel_bg: Color::Rgb(10, 18, 24),
        },
        ThemeMode::Matrix => Theme {
            accent: Color::Green,
            warn: Color::LightGreen,
            danger: Color::LightRed,
            dim: Color::Rgb(0, 120, 0),
            highlight_fg: Color::Black,
            highlight_bg: Color::Green,
            fav: Color::Green,
            watch: Color::LightCyan,
            row_even_bg: Color::Rgb(0, 18, 0),
            row_odd_bg: Color::Rgb(0, 12, 0),
            header_bg: Color::Rgb(0, 22, 0),
            panel_bg: Color::Rgb(0, 16, 0),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        center_text, fmt_f64_trend, fmt_i64_trend, fmt_text, format_track_cell,
        format_track_display, get_flag, text_len, truncate_to_width, TrendDir,
    };

    #[test]
    fn test_get_flag() {
        // Test single-letter prefixes
        assert_eq!(get_flag(Some("N12345")), "🇺🇸"); // USA
        assert_eq!(get_flag(Some("GABCD")), "🇬🇧");  // UK
        assert_eq!(get_flag(Some("PH123")), "🇳🇱");  // Netherlands (two-letter)
        assert_eq!(get_flag(Some("")), "🏳️");       // Empty
        assert_eq!(get_flag(None), "🏳️");           // None
        assert_eq!(get_flag(Some("9K123")), "🇰🇼"); // Kuwait (two-letter starting with digit)

        // Test more country codes
        assert_eq!(get_flag(Some("DABCD")), "🇩🇪");  // Germany
        assert_eq!(get_flag(Some("FABCD")), "🇫🇷");  // France
        assert_eq!(get_flag(Some("JABCD")), "🇯🇵");  // Japan
        assert_eq!(get_flag(Some("C1234")), "🇨🇦");  // Canada
        assert_eq!(get_flag(Some("LY123")), "🇱🇹");  // Lithuania (two-letter)
        assert_eq!(get_flag(Some("ZS123")), "🇿🇦");  // South Africa (two-letter)
    }

    #[test]
    fn test_get_flag_edge_cases() {
        // Test various edge cases
        assert_eq!(get_flag(Some("A")), "🇦🇺");      // Single character
        assert_eq!(get_flag(Some("1")), "🏳️");      // Invalid single digit
        assert_eq!(get_flag(Some("123")), "🏳️");    // All digits
        assert_eq!(get_flag(Some("X9Y")), "🇨🇳");    // Single letter fallback
    }

    #[test]
    fn test_text_helpers() {
        assert_eq!(fmt_text(None), "--");
        assert_eq!(fmt_text(Some("   ")), "--");
        assert_eq!(fmt_text(Some("AB")), "AB");
        assert_eq!(truncate_to_width("ABCDE".to_string(), 3), "ABC");
        assert_eq!(truncate_to_width("ABCDE".to_string(), 0), "ABCDE");
        assert_eq!(center_text("A", 3), " A ");
        assert_eq!(center_text("AB", 2), "AB");
        assert_eq!(text_len("ABC"), 3);
    }

    #[test]
    fn test_track_formatting() {
        assert_eq!(format_track_display(Some(370.0), false), "010°");
        assert_eq!(format_track_cell(Some(90.0), true), "090→");
        assert_eq!(format_track_display(None, true), "--");
    }

    #[test]
    fn test_trend_formatting() {
        assert_eq!(fmt_i64_trend(Some(100), TrendDir::Up, true, 0), "100↑");
        assert_eq!(fmt_i64_trend(None, TrendDir::Unknown, true, 0), "-- ");
        assert_eq!(fmt_f64_trend(Some(1.5), TrendDir::Down, 0, 1), "1.5↓");
    }
}
