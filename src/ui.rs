use chrono::{DateTime, Local, Utc};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap};
use ratatui::Frame;
use std::time::{Duration, SystemTime};

use crate::app::{App, ColumnId, InputMode, LayoutMode, SiteLocation, ThemeMode, TrendDir};
use crate::model::seen_seconds;

struct Theme {
    accent: Color,
    warn: Color,
    danger: Color,
    dim: Color,
    highlight_fg: Color,
    highlight_bg: Color,
    fav: Color,
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

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let count = app.data.aircraft.len();
    let msg_total = app.data.messages.unwrap_or(0);
    let msg_rate_total = app.msg_rate_display();
    let avg_rate = match (msg_rate_total, count) {
        (Some(rate), n) if n > 0 => Some(rate / n as f64),
        _ => None,
    };
    let kbps = avg_rate.map(|rate| rate * 112.0 / 1000.0);

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

    let spinner = ["|", "/", "-", "\\"][app.tick as usize % 4];
    let blink = app
        .last_update
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .map(|d| d.as_millis() < 900)
        .unwrap_or(false);
    let sync_style = if blink {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dim)
    };

    let rate_text = match (avg_rate, kbps) {
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
        Span::styled(
            format!("AVG {rate_text}"),
            Style::default().fg(theme.accent),
        ),
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
    spans.extend(alert_span("NEAR", near, theme.accent));
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
    let msg_rate = app.msg_rate_display();
    let kbps = msg_rate.map(|rate| rate * 112.0 / 1000.0);
    let uptime = now
        .duration_since(app.start_time)
        .map(format_duration)
        .unwrap_or_else(|_| "--".to_string());
    let last_update = app
        .last_update
        .and_then(|t| now.duration_since(t).ok())
        .map(|d| format!("{}s", d.as_secs()))
        .unwrap_or_else(|| "--".to_string());
    let site_alt = app
        .site()
        .map(|site| format!("{:.1} m", site.alt_m))
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

    let lines = vec![
        Line::from(vec![
            Span::styled("VISIBLE ", Style::default().fg(theme.dim)),
            Span::styled(
                format!("{visible}/{total}"),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("MSG RATE ", Style::default().fg(theme.dim)),
            Span::raw(match msg_rate {
                Some(value) => format!("{value:.1}/s"),
                None => "--".to_string(),
            }),
        ]),
        Line::from(vec![
            Span::styled("EST KBPS ", Style::default().fg(theme.dim)),
            Span::raw(match kbps {
                Some(value) => format!("{value:.1}"),
                None => "--".to_string(),
            }),
        ]),
        Line::from(vec![
            Span::styled("SEEN 1/5/15 ", Style::default().fg(theme.dim)),
            Span::raw(format!("{last_1}/{last_5}/{last_15}")),
        ]),
        Line::from(vec![
            Span::styled("UPTIME ", Style::default().fg(theme.dim)),
            Span::raw(uptime),
        ]),
        Line::from(vec![
            Span::styled("LAST UPD ", Style::default().fg(theme.dim)),
            Span::raw(last_update),
        ]),
        Line::from(vec![
            Span::styled("SITE ALT ", Style::default().fg(theme.dim)),
            Span::raw(site_alt),
        ]),
        Line::from(vec![
            Span::styled("ROUTE ERR ", Style::default().fg(theme.dim)),
            Span::raw(route_error),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title("STATS");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, area);
}

fn render_radar(f: &mut Frame, area: Rect, app: &App, indices: &[usize]) {
    let theme = theme(app.theme_mode);
    let mut points = Vec::new();
    let mut sum_lat = 0.0;
    let mut sum_lon = 0.0;
    let mut current_points = 0usize;

    for idx in indices {
        let ac = &app.data.aircraft[*idx];
        if let (Some(lat), Some(lon)) = (ac.lat, ac.lon) {
            points.push((lat, lon, app.is_favorite(ac), true));
            sum_lat += lat;
            sum_lon += lon;
            current_points += 1;
        }
        if let Some(trail) = app.trail_for(ac) {
            for point in trail {
                points.push((point.lat, point.lon, app.is_favorite(ac), false));
            }
        }
    }

    let mut lines = Vec::new();
    if points.is_empty() || current_points == 0 {
        lines.push(Line::from("No position data"));
    } else {
        let (center_lat, center_lon) = match app.site() {
            Some(site) => (site.lat, site.lon),
            None => (
                sum_lat / current_points as f64,
                sum_lon / current_points as f64,
            ),
        };
        let mut max_delta = 0.0001f64;
        for (lat, lon, _, _) in &points {
            let dlat = (lat - center_lat).abs();
            let dlon = (lon - center_lon).abs();
            max_delta = max_delta.max(dlat.max(dlon));
        }

        let width = area.width.saturating_sub(2) as usize;
        let height = area.height.saturating_sub(2) as usize;
        let width = width.max(1);
        let height = height.max(1);
        let mut grid = vec![vec![('.', 0u8); width]; height];
        let cx = width / 2;
        let cy = height / 2;
        set_grid(&mut grid, cx, cy, '+', 1);

        for (lat, lon, fav, current) in points {
            let dx = (lon - center_lon) / max_delta;
            let dy = (lat - center_lat) / max_delta;
            let x = ((dx + 1.0) * 0.5 * (width.saturating_sub(1)) as f64) as isize;
            let y = ((1.0 - (dy + 1.0) * 0.5) * (height.saturating_sub(1)) as f64) as isize;
            let xi = x.clamp(0, width.saturating_sub(1) as isize) as usize;
            let yi = y.clamp(0, height.saturating_sub(1) as isize) as usize;
            let (ch, prio) = match (fav, current) {
                (true, true) => ('F', 4),
                (false, true) => ('*', 3),
                (true, false) => ('f', 2),
                (false, false) => ('o', 2),
            };
            set_grid(&mut grid, xi, yi, ch, prio);
        }

        for row in grid {
            let line: String = row.into_iter().map(|(ch, _)| ch).collect();
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme.dim),
            )));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title("RADAR");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, area);
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

    let rows = indices.iter().enumerate().map(|(i, idx)| {
        let ac = &app.data.aircraft[*idx];
        let seen = seen_seconds(ac);
        let stale = seen.map(|s| s > app.stale_secs).unwrap_or(true);
        let favorite = app.is_favorite(ac);
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
        } else if stale {
            style = if app.tick % 2 == 0 {
                style.fg(theme.danger)
            } else {
                style.fg(theme.dim)
            };
        }

        let route = app.route_for(ac);
        let cells = columns.iter().zip(widths.iter()).map(|(col, width)| {
            cell_for_column(
                col.id,
                *width as usize,
                ac,
                favorite,
                seen,
                trend,
                route,
                &theme,
                app.site(),
                app.altitude_trend_arrows,
            )
        });

        Row::new(cells).style(style)
    });

    let constraints: Vec<Constraint> = widths
        .iter()
        .map(|width| Constraint::Length(*width))
        .collect();

    let table = Table::new(rows, constraints)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("AIRSPACE"),
        )
        .column_spacing(1)
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
        let track = fmt_f64(ac.track, 0, 0);
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
        let route = app
            .route_for(ac)
            .map(route_display)
            .unwrap_or("--".to_string());
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
                Span::raw(format!("{gs} kt / {track} deg")),
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
    let sweep_pos = if width == 0 {
        0
    } else {
        (app.tick as usize) % width
    };
    let mut sweep = String::with_capacity(width);
    for i in 0..width {
        if i == sweep_pos {
            sweep.push('*');
        } else {
            sweep.push('.');
        }
    }

    let mut help = "q quit  s sort  / filter  f favorite  c clear  t theme  l layout  m columns  e export  C config  ? help".to_string();
    let source = short_source(&app.url);
    help.push_str(&format!("  REF {}s  SRC {}", app.refresh.as_secs(), source));

    if let Some((name, when)) = &app.last_export {
        if let Ok(delta) = SystemTime::now().duration_since(*when) {
            if delta <= Duration::from_secs(5) {
                help.push_str(&format!("  saved {name}"));
            }
        }
    }

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
    let popup = centered_rect(64, 18, area);

    f.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            "Keys",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("q           Quit"),
        Line::from("Up/Down     Move selection"),
        Line::from("s           Sort (SEEN/ALT/SPD)"),
        Line::from("/           Filter (Enter apply, Esc cancel, Ctrl+U clear)"),
        Line::from("c           Clear filter"),
        Line::from("f           Toggle favorite (auto-saves)"),
        Line::from("m           Columns menu"),
        Line::from("t           Theme toggle"),
        Line::from("l           Layout toggle"),
        Line::from("e           Export CSV (visible rows)"),
        Line::from("E           Export JSON (raw feed)"),
        Line::from("? or h      Toggle help"),
        Line::from("C           Config editor"),
        Line::from("Mouse       Scroll to move • Click row to select"),
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

fn render_config_menu(f: &mut Frame, area: Rect, app: &App) {
    let theme = theme(app.theme_mode);
    let height = (app.config_items.len() + 6).min(24) as u16;
    let popup = centered_rect(72, height, area);

    f.render_widget(Clear, popup);

    let key_width = app
        .config_items
        .iter()
        .map(|item| item.key.len())
        .max()
        .unwrap_or(8);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("CONFIG {}", app.config_path.display()),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (i, item) in app.config_items.iter().enumerate() {
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
    if let Some((msg, when)) = &app.config_status {
        if SystemTime::now()
            .duration_since(*when)
            .map(|d| d.as_secs() <= 5)
            .unwrap_or(true)
        {
            lines.push(Line::from(Span::styled(
                msg,
                Style::default().fg(theme.warn),
            )));
        }
    }
    lines.push(Line::from(Span::styled(
        "Up/Down select • Enter edit/apply • w save • Esc close",
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
    if value.len() > width {
        value.truncate(width);
    }
    value
}

fn center_text(value: &str, width: usize) -> String {
    if width == 0 {
        return value.to_string();
    }
    let len = value.len();
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
        ColumnId::Fav,
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
        desired[i] = desired[i].max(col.label.len());
    }

    let sample_limit = indices.len().min(50);
    for idx in indices.iter().take(sample_limit) {
        let ac = &app.data.aircraft[*idx];
        let trend = app.trend_for(ac);
        let route = app.route_for(ac);
        for (i, col) in columns.iter().enumerate() {
            let value = match col.id {
                ColumnId::Fav => {
                    if app.is_favorite(ac) {
                        "*".to_string()
                    } else {
                        " ".to_string()
                    }
                }
                ColumnId::Flight => fmt_text(ac.flight.as_deref()),
                ColumnId::Reg => fmt_text(ac.r.as_deref()),
                ColumnId::Type => fmt_text(ac.t.as_deref()),
                ColumnId::Route => route.map(route_display).unwrap_or_else(|| "--".to_string()),
                ColumnId::Alt => {
                    fmt_i64_trend(ac.alt_baro, trend.alt, app.altitude_trend_arrows, 0)
                }
                ColumnId::Gs => fmt_f64_trend(ac.gs, trend.gs, 0, 0),
                ColumnId::Trk => fmt_f64(ac.track, 0, 0),
                ColumnId::Lat => fmt_f64(ac.lat, 0, 2),
                ColumnId::Lon => fmt_f64(ac.lon, 0, 2),
                ColumnId::Dist => fmt_distance(app.site(), ac, 0),
                ColumnId::Brg => fmt_bearing(app.site(), ac, 0),
                ColumnId::Seen => fmt_f64(seen_seconds(ac), 0, 0),
                ColumnId::Msgs => fmt_u64(ac.messages, 0),
                ColumnId::Hex => fmt_text(ac.hex.as_deref()),
            };
            desired[i] = desired[i].max(value.len());
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
    seen: Option<f64>,
    trend: crate::app::Trend,
    route: Option<&crate::app::RouteInfo>,
    theme: &Theme,
    site: Option<SiteLocation>,
    altitude_trend_arrows: bool,
) -> Cell<'static> {
    let mut text = match id {
        ColumnId::Fav => {
            if favorite {
                "*".to_string()
            } else {
                " ".to_string()
            }
        }
        ColumnId::Flight => fmt_text(ac.flight.as_deref()),
        ColumnId::Reg => fmt_text(ac.r.as_deref()),
        ColumnId::Type => fmt_text(ac.t.as_deref()),
        ColumnId::Route => route.map(route_display).unwrap_or_else(|| "--".to_string()),
        ColumnId::Alt => fmt_i64_trend(ac.alt_baro, trend.alt, altitude_trend_arrows, 0),
        ColumnId::Gs => fmt_f64_trend(ac.gs, trend.gs, 0, 0),
        ColumnId::Trk => fmt_f64(ac.track, 0, 0),
        ColumnId::Lat => fmt_f64(ac.lat, 0, 2),
        ColumnId::Lon => fmt_f64(ac.lon, 0, 2),
        ColumnId::Dist => fmt_distance(site, ac, 0),
        ColumnId::Brg => fmt_bearing(site, ac, 0),
        ColumnId::Seen => fmt_f64(seen, 0, 0),
        ColumnId::Msgs => fmt_u64(ac.messages, 0),
        ColumnId::Hex => fmt_text(ac.hex.as_deref()),
    };

    text = truncate_to_width(text, width);
    let text = center_text(&text, width);

    if id == ColumnId::Fav && favorite {
        Cell::from(text).style(Style::default().fg(theme.fav).add_modifier(Modifier::BOLD))
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

fn set_grid(grid: &mut [Vec<(char, u8)>], x: usize, y: usize, ch: char, prio: u8) {
    if let Some(row) = grid.get_mut(y) {
        if let Some(cell) = row.get_mut(x) {
            if prio >= cell.1 {
                *cell = (ch, prio);
            }
        }
    }
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
    if value.len() <= max {
        value.to_string()
    } else if max <= 3 {
        value[..max].to_string()
    } else {
        format!("{}...", &value[..(max - 3)])
    }
}

fn column_name(id: ColumnId) -> &'static str {
    match id {
        ColumnId::Fav => "FAVORITE",
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
            row_even_bg: Color::Rgb(0, 18, 0),
            row_odd_bg: Color::Rgb(0, 12, 0),
            header_bg: Color::Rgb(0, 22, 0),
            panel_bg: Color::Rgb(0, 16, 0),
        },
    }
}
