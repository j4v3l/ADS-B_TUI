use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::canvas::{Canvas, Circle, Line as CanvasLine, Points};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, RadarRenderer};
use crate::model::seen_seconds;

const SWEEP_PERIOD_MS: u64 = 4500;
const MIN_RANGE_NM: f64 = 1.0;
const MIN_ASPECT: f64 = 0.2;

#[derive(Clone, Copy)]
pub struct RadarTheme {
    pub accent: Color,
    pub dim: Color,
    pub fav: Color,
    pub warn: Color,
    pub panel_bg: Color,
}

#[derive(Clone, Copy)]
pub struct RadarSettings {
    pub range_nm: f64,
    pub aspect: f64,
    pub renderer: RadarRenderer,
}

pub fn render(
    f: &mut Frame,
    area: Rect,
    app: &App,
    indices: &[usize],
    theme: RadarTheme,
    settings: RadarSettings,
) {
    let data = match collect_data(app, indices, settings.range_nm) {
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
}

#[derive(Clone, Copy)]
struct RadarPoint {
    x: f64,
    y: f64,
    fav: bool,
    current: bool,
    seen_secs: Option<f64>,
}

struct RadarData {
    points: Vec<RadarPoint>,
    range_nm: f64,
}

fn collect_data(app: &App, indices: &[usize], range_nm: f64) -> Option<RadarData> {
    let mut sum_lat = 0.0;
    let mut sum_lon = 0.0;
    let mut current_points = 0usize;
    let mut raw_points: Vec<(f64, f64, bool, bool, Option<f64>)> = Vec::new();

    for idx in indices {
        let ac = &app.data.aircraft[*idx];
        if let (Some(lat), Some(lon)) = (ac.lat, ac.lon) {
            raw_points.push((lat, lon, app.is_favorite(ac), true, seen_seconds(ac)));
            sum_lat += lat;
            sum_lon += lon;
            current_points += 1;
        }
        if let Some(trail) = app.trail_for(ac) {
            for point in trail {
                raw_points.push((point.lat, point.lon, app.is_favorite(ac), false, None));
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
    for (lat, lon, fav, current, seen_secs) in raw_points {
        let dist = distance_nm(center_lat, center_lon, lat, lon);
        if dist > range_nm {
            continue;
        }
        let bearing = bearing_deg(center_lat, center_lon, lat, lon).to_radians();
        let x = dist * bearing.sin();
        let y = dist * bearing.cos();
        points.push(RadarPoint {
            x,
            y,
            fav,
            current,
            seen_secs,
        });
    }

    Some(RadarData { points, range_nm })
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
