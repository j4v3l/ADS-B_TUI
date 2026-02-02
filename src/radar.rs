use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::canvas::{Canvas, Circle, Line as CanvasLine, Points};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, LayoutMode, RadarBlip, RadarRenderer};
use crate::model::{seen_seconds, Aircraft};

const SWEEP_PERIOD_MS: u64 = 4500;
const MIN_RANGE_NM: f64 = 1.0;
const MIN_ASPECT: f64 = 0.2;
const LABEL_MAX_LEN: usize = 6;

#[derive(Clone, Copy)]
pub struct RadarTheme {
    pub accent: Color,
    pub dim: Color,
    pub fav: Color,
    pub warn: Color,
    pub highlight: Color,
    pub panel_bg: Color,
}

#[derive(Clone, Copy)]
pub struct RadarSettings {
    pub range_nm: f64,
    pub aspect: f64,
    pub renderer: RadarRenderer,
    pub blip: RadarBlip,
}

pub fn render(
    f: &mut Frame,
    area: Rect,
    app: &App,
    indices: &[usize],
    theme: RadarTheme,
    settings: RadarSettings,
) {
    let show_labels = app.radar_labels && matches!(app.layout_mode, LayoutMode::Radar);
    let data = match collect_data(app, indices, settings.range_nm, show_labels) {
        Some(data) => data,
        None => {
            render_empty(f, area, theme);
            return;
        }
    };

    let use_canvas = matches!(settings.renderer, RadarRenderer::Canvas);
    if use_canvas && area.width >= 8 && area.height >= 6 {
        render_canvas(f, area, &data, theme, settings);
    } else {
        render_ascii(f, area, &data, theme);
    }

    if matches!(app.layout_mode, LayoutMode::Radar) {
        if let Some(selection) = &data.selection {
            render_selection_panel(f, area, theme, selection);
        }
    }
}

#[derive(Clone, Copy)]
struct RadarPoint {
    x: f64,
    y: f64,
    track: Option<f64>,
    fav: bool,
    current: bool,
    seen_secs: Option<f64>,
}

struct RadarLabel {
    x: f64,
    y: f64,
    text: String,
    id: String,
    dist: f64,
    fav: bool,
    fresh: bool,
    selected: bool,
}

struct RadarData {
    points: Vec<RadarPoint>,
    range_nm: f64,
    selection: Option<RadarSelection>,
    labels: Vec<RadarLabel>,
}

struct RadarSelection {
    position: Option<(f64, f64)>,
    lines: Vec<String>,
}

fn collect_data(
    app: &App,
    indices: &[usize],
    range_nm: f64,
    collect_labels: bool,
) -> Option<RadarData> {
    struct RawPoint {
        lat: f64,
        lon: f64,
        track: Option<f64>,
        fav: bool,
        current: bool,
        seen_secs: Option<f64>,
        label: Option<LabelInfo>,
    }

    let selected_id = app
        .table_state
        .selected()
        .and_then(|row| indices.get(row))
        .and_then(|idx| label_info(&app.data.aircraft[*idx]).map(|info| info.id));

    let mut sum_lat = 0.0;
    let mut sum_lon = 0.0;
    let mut current_points = 0usize;
    let mut raw_points: Vec<RawPoint> = Vec::new();

    for idx in indices {
        let ac = &app.data.aircraft[*idx];
        if let (Some(lat), Some(lon)) = (ac.lat, ac.lon) {
            let label = if collect_labels { label_info(ac) } else { None };
            raw_points.push(RawPoint {
                lat,
                lon,
                track: ac.track,
                fav: app.is_favorite(ac),
                current: true,
                seen_secs: seen_seconds(ac),
                label,
            });
            sum_lat += lat;
            sum_lon += lon;
            current_points += 1;
        }
        if let Some(trail) = app.trail_for(ac) {
            for point in trail {
                raw_points.push(RawPoint {
                    lat: point.lat,
                    lon: point.lon,
                    track: None,
                    fav: app.is_favorite(ac),
                    current: false,
                    seen_secs: None,
                    label: None,
                });
            }
        }
    }

    if raw_points.is_empty() || current_points == 0 {
        return None;
    }

    let (center_lat, center_lon) = match app.site() {
        Some(site) => (site.lat, site.lon),
        None => (
            sum_lat / current_points as f64,
            sum_lon / current_points as f64,
        ),
    };

    let range_nm = range_nm.max(MIN_RANGE_NM);
    let mut points = Vec::with_capacity(raw_points.len());
    let mut labels = Vec::new();
    for raw in raw_points {
        let dist = distance_nm(center_lat, center_lon, raw.lat, raw.lon);
        if dist > range_nm {
            continue;
        }
        let bearing = bearing_deg(center_lat, center_lon, raw.lat, raw.lon).to_radians();
        let x = dist * bearing.sin();
        let y = dist * bearing.cos();
        points.push(RadarPoint {
            x,
            y,
            track: raw.track,
            fav: raw.fav,
            current: raw.current,
            seen_secs: raw.seen_secs,
        });
        if collect_labels {
            if let Some(info) = raw.label {
                let fresh = raw.seen_secs.map(|s| s <= 1.0).unwrap_or(false);
                let selected = selected_id
                    .as_ref()
                    .map(|id| id == &info.id)
                    .unwrap_or(false);
                labels.push(RadarLabel {
                    x,
                    y,
                    text: info.text,
                    id: info.id,
                    dist,
                    fav: raw.fav,
                    fresh,
                    selected,
                });
            }
        }
    }

    let selection = selected_aircraft(app, indices, center_lat, center_lon, range_nm);

    Some(RadarData {
        points,
        range_nm,
        selection,
        labels,
    })
}

fn render_empty(f: &mut Frame, area: Rect, theme: RadarTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title("RADAR");
    let paragraph = Paragraph::new(vec![TextLine::from(Span::styled(
        "No position data",
        Style::default().fg(theme.dim),
    ))])
    .block(block)
    .wrap(Wrap { trim: true })
    .style(Style::default().bg(theme.panel_bg));
    f.render_widget(paragraph, area);
}

fn render_canvas(
    f: &mut Frame,
    area: Rect,
    data: &RadarData,
    theme: RadarTheme,
    settings: RadarSettings,
) {
    let range = data.range_nm.max(MIN_RANGE_NM);
    let aspect = settings.aspect.max(MIN_ASPECT);
    let y_bounds = [-range * aspect, range * aspect];
    let x_bounds = [-range, range];

    let sweep_rad = sweep_angle(SWEEP_PERIOD_MS);
    let sweep_x = range * sweep_rad.sin();
    let sweep_y = range * sweep_rad.cos();

    let mut trail = Vec::new();
    let mut trail_fav = Vec::new();
    let mut current = Vec::new();
    let mut current_fresh = Vec::new();
    let mut current_fav = Vec::new();

    for point in &data.points {
        let coord = (point.x, point.y);
        if point.current {
            if point.fav {
                current_fav.push(coord);
            } else if point.seen_secs.map(|s| s <= 1.0).unwrap_or(false) {
                current_fresh.push(coord);
            } else {
                current.push(coord);
            }
        } else if point.fav {
            trail_fav.push(coord);
        } else {
            trail.push(coord);
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title("RADAR");
    let canvas = Canvas::default()
        .block(block)
        .x_bounds(x_bounds)
        .y_bounds(y_bounds)
        .background_color(theme.panel_bg)
        .marker(Marker::Braille)
        .paint(|ctx| {
            for i in 1..=4 {
                let radius = range * (i as f64 / 4.0);
                ctx.draw(&Circle {
                    x: 0.0,
                    y: 0.0,
                    radius,
                    color: theme.dim,
                });
            }
            ctx.draw(&CanvasLine {
                x1: -range,
                y1: 0.0,
                x2: range,
                y2: 0.0,
                color: theme.dim,
            });
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: -range,
                x2: 0.0,
                y2: range,
                color: theme.dim,
            });
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: 0.0,
                x2: sweep_x,
                y2: sweep_y,
                color: theme.warn,
            });
            if !trail.is_empty() {
                ctx.draw(&Points {
                    coords: &trail,
                    color: theme.dim,
                });
            }
            if !trail_fav.is_empty() {
                ctx.draw(&Points {
                    coords: &trail_fav,
                    color: theme.fav,
                });
            }

            match settings.blip {
                RadarBlip::Dot => {
                    if !current.is_empty() {
                        ctx.draw(&Points {
                            coords: &current,
                            color: theme.dim,
                        });
                    }
                    if !current_fresh.is_empty() {
                        ctx.draw(&Points {
                            coords: &current_fresh,
                            color: theme.accent,
                        });
                    }
                    if !current_fav.is_empty() {
                        ctx.draw(&Points {
                            coords: &current_fav,
                            color: theme.fav,
                        });
                    }
                }
                _ => {
                    for point in &data.points {
                        if !point.current {
                            continue;
                        }
                        let color = if point.fav {
                            theme.fav
                        } else if point.seen_secs.map(|s| s <= 1.0).unwrap_or(false) {
                            theme.accent
                        } else {
                            theme.dim
                        };
                        let glyph = blip_glyph(settings.blip, point.track);
                        ctx.print(
                            point.x,
                            point.y,
                            TextLine::from(Span::styled(
                                glyph.to_string(),
                                Style::default().fg(color),
                            )),
                        );
                    }
                }
            }
            if !data.labels.is_empty() {
                let label_offset = (range * 0.02).max(0.6).min(5.0);
                let mut selected_labels = Vec::new();
                let mut fav_labels = Vec::new();
                let mut other_labels = Vec::new();

                for label in &data.labels {
                    if label.selected {
                        selected_labels.push(label);
                    } else if label.fav {
                        fav_labels.push(label);
                    } else {
                        other_labels.push(label);
                    }
                }

                other_labels.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap_or(Ordering::Equal));

                let max_labels = label_capacity(area);
                let mut drawn = Vec::new();
                append_unique_labels(&mut drawn, &selected_labels);
                append_unique_labels(&mut drawn, &fav_labels);
                let remaining = max_labels.saturating_sub(drawn.len());
                if remaining > 0 {
                    append_unique_labels(
                        &mut drawn,
                        &other_labels[..remaining.min(other_labels.len())],
                    );
                }

                for label in drawn {
                    let color = if label.selected {
                        theme.highlight
                    } else if label.fav {
                        theme.fav
                    } else if label.fresh {
                        theme.accent
                    } else {
                        theme.dim
                    };
                    ctx.print(
                        label.x,
                        label.y + label_offset,
                        TextLine::from(Span::styled(
                            label.text.clone(),
                            Style::default().fg(color),
                        )),
                    );
                }
            }
            if let Some(selection) = &data.selection {
                if let Some((x, y)) = selection.position {
                    let marker = (range * 0.02).max(0.5).min(6.0);
                    ctx.draw(&Circle {
                        x,
                        y,
                        radius: marker,
                        color: theme.highlight,
                    });
                    ctx.draw(&CanvasLine {
                        x1: x - marker,
                        y1: y,
                        x2: x + marker,
                        y2: y,
                        color: theme.highlight,
                    });
                    ctx.draw(&CanvasLine {
                        x1: x,
                        y1: y - marker,
                        x2: x,
                        y2: y + marker,
                        color: theme.highlight,
                    });
                }
            }
        });
    f.render_widget(canvas, area);
}

fn render_ascii(f: &mut Frame, area: Rect, data: &RadarData, theme: RadarTheme) {
    let width = area.width.saturating_sub(2) as usize;
    let height = area.height.saturating_sub(2) as usize;
    if width == 0 || height == 0 {
        return;
    }

    let mut grid = vec![vec![('.', 0u8); width]; height];
    let cx = width / 2;
    let cy = height / 2;
    set_grid(&mut grid, cx, cy, '+', 1);

    let sweep_rad = sweep_angle(SWEEP_PERIOD_MS);
    let max_r = (width.min(height) as f64 / 2.0).max(1.0) as usize;
    for r in 0..=max_r {
        let x = (cx as f64 + r as f64 * sweep_rad.sin()).round() as isize;
        let y = (cy as f64 - r as f64 * sweep_rad.cos()).round() as isize;
        let xi = x.clamp(0, width.saturating_sub(1) as isize) as usize;
        let yi = y.clamp(0, height.saturating_sub(1) as isize) as usize;
        set_grid(&mut grid, xi, yi, ':', 0);
    }

    for point in &data.points {
        let dx = point.x / data.range_nm;
        let dy = point.y / data.range_nm;
        let x = ((dx + 1.0) * 0.5 * (width.saturating_sub(1)) as f64) as isize;
        let y = ((1.0 - (dy + 1.0) * 0.5) * (height.saturating_sub(1)) as f64) as isize;
        let xi = x.clamp(0, width.saturating_sub(1) as isize) as usize;
        let yi = y.clamp(0, height.saturating_sub(1) as isize) as usize;
        let (ch, prio) = match (point.fav, point.current) {
            (true, true) => ('F', 4),
            (false, true) => ('*', 3),
            (true, false) => ('f', 2),
            (false, false) => ('o', 1),
        };
        set_grid(&mut grid, xi, yi, ch, prio);
    }

    if let Some(selection) = &data.selection {
        if let Some((x, y)) = selection.position {
            let dx = x / data.range_nm;
            let dy = y / data.range_nm;
            let sx = ((dx + 1.0) * 0.5 * (width.saturating_sub(1)) as f64) as isize;
            let sy = ((1.0 - (dy + 1.0) * 0.5) * (height.saturating_sub(1)) as f64) as isize;
            let xi = sx.clamp(0, width.saturating_sub(1) as isize) as usize;
            let yi = sy.clamp(0, height.saturating_sub(1) as isize) as usize;
            set_grid(&mut grid, xi, yi, 'X', 5);
        }
    }

    let mut lines = Vec::with_capacity(height);
    for row in grid {
        let line: String = row.into_iter().map(|(ch, _)| ch).collect();
        lines.push(TextLine::from(Span::styled(
            line,
            Style::default().fg(theme.dim),
        )));
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

fn render_selection_panel(
    f: &mut Frame,
    area: Rect,
    theme: RadarTheme,
    selection: &RadarSelection,
) {
    if selection.lines.is_empty() || area.width < 12 || area.height < 6 {
        return;
    }
    let mut height = selection.lines.len() as u16 + 2;
    height = height.min(area.height.saturating_sub(2)).max(6);
    let mut width = 34u16.min(area.width.saturating_sub(2));
    if width < 12 {
        width = area.width.saturating_sub(2).max(12);
    }
    let x = area.x.saturating_add(1);
    let y = area
        .y
        .saturating_add(area.height.saturating_sub(height + 1));
    let panel = Rect {
        x,
        y,
        width,
        height,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("TARGET");
    let mut styled_lines = Vec::new();
    for (i, line) in selection.lines.iter().enumerate() {
        let styled = if i == 0 {
            TextLine::from(Span::styled(
                line.clone(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            TextLine::from(Span::raw(line.clone()))
        };
        styled_lines.push(styled);
    }
    let paragraph = Paragraph::new(styled_lines)
        .block(block)
        .style(Style::default().bg(theme.panel_bg))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, panel);
}

fn label_capacity(area: Rect) -> usize {
    let width = area.width.saturating_sub(2) as usize;
    let height = area.height.saturating_sub(2) as usize;
    let by_width = (width / 10).max(2);
    let by_height = (height / 3).max(2);
    by_width.min(by_height).min(12).max(2)
}

fn append_unique_labels<'a>(target: &mut Vec<&'a RadarLabel>, source: &[&'a RadarLabel]) {
    for label in source {
        if !target.iter().any(|existing| existing.id == label.id) {
            target.push(*label);
        }
    }
}

fn blip_glyph(style: RadarBlip, track: Option<f64>) -> &'static str {
    match style {
        RadarBlip::Plane => track.map(direction_glyph).unwrap_or("✈"),
        RadarBlip::Block => "■",
        RadarBlip::Dot => "•",
    }
}

fn direction_glyph(track: f64) -> &'static str {
    let mut heading = track % 360.0;
    if heading < 0.0 {
        heading += 360.0;
    }
    let idx = ((heading + 22.5) / 45.0).floor() as usize % 8;
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

fn set_grid(grid: &mut [Vec<(char, u8)>], x: usize, y: usize, ch: char, prio: u8) {
    if let Some(row) = grid.get_mut(y) {
        if let Some(cell) = row.get_mut(x) {
            if prio >= cell.1 {
                *cell = (ch, prio);
            }
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn sweep_angle(period_ms: u64) -> f64 {
    if period_ms == 0 {
        return 0.0;
    }
    let sweep_pos = (now_ms() % period_ms) as f64 / period_ms as f64;
    sweep_pos * std::f64::consts::TAU
}

fn distance_nm(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * a.sqrt().asin() * 3440.065
}

fn bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let mut deg = y.atan2(x).to_degrees();
    if deg < 0.0 {
        deg += 360.0;
    }
    deg
}

struct LabelInfo {
    id: String,
    text: String,
}

fn label_info(ac: &Aircraft) -> Option<LabelInfo> {
    let callsign = ac
        .flight
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(value) = callsign {
        let clean = value.to_ascii_uppercase();
        return Some(LabelInfo {
            id: clean.clone(),
            text: truncate_text(&clean, LABEL_MAX_LEN),
        });
    }
    let hex = ac.hex.as_deref().map(str::trim).filter(|v| !v.is_empty())?;
    let id = hex.to_ascii_lowercase();
    let text = truncate_text(&id.to_ascii_uppercase(), LABEL_MAX_LEN);
    Some(LabelInfo { id, text })
}

fn truncate_text(value: &str, max_len: usize) -> String {
    if max_len == 0 {
        return value.to_string();
    }
    value.chars().take(max_len).collect()
}

fn selected_aircraft(
    app: &App,
    indices: &[usize],
    center_lat: f64,
    center_lon: f64,
    range_nm: f64,
) -> Option<RadarSelection> {
    let selected_idx = app
        .table_state
        .selected()
        .and_then(|row| indices.get(row).copied());
    let Some(idx) = selected_idx else {
        return None;
    };
    let ac = &app.data.aircraft[idx];
    let callsign = ac
        .flight
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("--");
    let hex = ac.hex.as_deref().unwrap_or("--");
    let mut lines = Vec::new();
    lines.push(callsign.to_string());
    lines.push(format!("HEX      {hex}"));

    let mut position = None;
    if let (Some(lat), Some(lon)) = (ac.lat, ac.lon) {
        let dist = distance_nm(center_lat, center_lon, lat, lon);
        let brg = bearing_deg(center_lat, center_lon, lat, lon);
        let dist_text = format!("{dist:.1} nm");
        let brg_text = format!("{brg:.0}°");
        lines.push(format!("RNG/BRG  {dist_text} / {brg_text}"));
        let bearing = brg.to_radians();
        let x = dist * bearing.sin();
        let y = dist * bearing.cos();
        if dist <= range_nm {
            position = Some((x, y));
        }
    } else {
        lines.push("RNG/BRG  -- / --".to_string());
    }

    let alt = ac.alt_baro.or(ac.alt_geom);
    let alt_text = alt
        .map(|v| format!("{v} ft"))
        .unwrap_or_else(|| "--".to_string());
    let gs_text = ac
        .gs
        .map(|v| format!("{v:.0} kt"))
        .unwrap_or_else(|| "--".to_string());
    let trk_text = ac
        .track
        .map(|v| format!("{v:.0}°"))
        .unwrap_or_else(|| "--".to_string());
    let seen_text = seen_seconds(ac)
        .map(|v| format!("{v:.1}s"))
        .unwrap_or_else(|| "--".to_string());

    lines.push(format!("ALT/GS  {alt_text} / {gs_text}"));
    lines.push(format!("TRK/SE  {trk_text} / {seen_text}"));

    Some(RadarSelection { position, lines })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Aircraft;

    #[test]
    fn direction_glyph_maps_cardinals() {
        assert_eq!(direction_glyph(0.0), "↑");
        assert_eq!(direction_glyph(45.0), "↗");
        assert_eq!(direction_glyph(90.0), "→");
        assert_eq!(direction_glyph(135.0), "↘");
        assert_eq!(direction_glyph(180.0), "↓");
        assert_eq!(direction_glyph(225.0), "↙");
        assert_eq!(direction_glyph(270.0), "←");
        assert_eq!(direction_glyph(315.0), "↖");
        assert_eq!(direction_glyph(360.0), "↑");
    }

    #[test]
    fn label_info_prefers_callsign() {
        let ac = Aircraft {
            flight: Some("aal123".to_string()),
            hex: Some("ab12cd".to_string()),
            ..Default::default()
        };
        let info = label_info(&ac).expect("label info");
        assert_eq!(info.id, "AAL123");
        assert_eq!(info.text, "AAL123");
    }

    #[test]
    fn label_info_falls_back_to_hex() {
        let ac = Aircraft {
            flight: None,
            hex: Some("ab12cd".to_string()),
            ..Default::default()
        };
        let info = label_info(&ac).expect("label info");
        assert_eq!(info.id, "ab12cd");
        assert_eq!(info.text, "AB12CD");
    }

    #[test]
    fn label_info_truncates() {
        let ac = Aircraft {
            flight: Some("LONGCALLSIGN".to_string()),
            hex: None,
            ..Default::default()
        };
        let info = label_info(&ac).expect("label info");
        assert_eq!(info.text.len(), LABEL_MAX_LEN);
        assert_eq!(info.text, "LONGCA");
    }

    #[test]
    fn blip_glyph_plane_uses_heading() {
        assert_eq!(blip_glyph(RadarBlip::Plane, Some(90.0)), "→");
        assert_eq!(blip_glyph(RadarBlip::Plane, None), "✈");
    }
}
