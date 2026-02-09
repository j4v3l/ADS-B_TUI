use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, SystemTime};

use crate::app::{App, InputMode, LayoutMode};
use crate::export;
use crate::lookup::{LookupMessage, LookupRequest};
use crate::model::ApiResponse;
use crate::routes::{RouteMessage, RouteRequest};
use crate::storage;
use crate::ui;
use tracing::{debug, error, info};

pub fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
    rx: Receiver<Result<ApiResponse, String>>,
    routes: Option<RouteChannels>,
    lookup: Option<LookupChannels>,
) -> Result<()> {
    let tick_rate = Duration::from_millis(50);
    info!("runtime loop started");
    let mut last_draw: Option<SystemTime> = None;
    loop {
        let mut dirty = false;
        while let Ok(message) = rx.try_recv() {
            match message {
                Ok(data) => {
                    debug!("data update received");
                    app.apply_update(data);
                }
                Err(err) => {
                    error!("data error: {err}");
                    app.apply_error(err);
                }
            }
            dirty = true;
        }

        if let Some(routes) = &routes {
            while let Ok(message) = routes.res_rx.try_recv() {
                match message {
                    RouteMessage::Results(results) => {
                        debug!("route results received: {}", results.len());
                        app.apply_routes(results);
                    }
                    RouteMessage::Error(err) => {
                        error!("route error: {err}");
                        app.set_route_error(err);
                    }
                }
                dirty = true;
            }
        }

        if let Some(lookup) = &lookup {
            while let Ok(message) = lookup.res_rx.try_recv() {
                match message {
                    LookupMessage::Result(data) => app.apply_lookup_result(data),
                    LookupMessage::Error(err) => app.apply_lookup_error(err),
                }
                dirty = true;
            }
        }

        let now = SystemTime::now();
        app.maybe_swap_snapshot(now);

        let draw_due = is_draw_due(now, last_draw, app.ui_interval);
        let poll_timeout = if dirty || draw_due {
            Duration::from_millis(0)
        } else {
            tick_rate
        };

        let mut indices = app.visible_indices();
        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) => match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down => {
                            app.next_row(indices.len());
                            app.update_selection_key(&indices);
                        }
                        KeyCode::Up => {
                            app.previous_row(indices.len());
                            app.update_selection_key(&indices);
                        }
                        KeyCode::Char('s') => app.toggle_sort(),
                        KeyCode::Char('/') => app.start_filter(),
                        KeyCode::Char('c') => app.clear_filter(),
                        KeyCode::Char('f') => {
                            if app.toggle_favorite_selected(&indices) {
                                if let Some(path) = app.favorites_path() {
                                    match storage::save_favorites(path, &app.favorites) {
                                        Ok(_) => debug!("favorites saved {}", path.display()),
                                        Err(err) => error!("favorites save failed: {err}"),
                                    }
                                }
                            }
                        }
                        KeyCode::Char('t') => app.toggle_theme(),
                        KeyCode::Char('l') => app.toggle_layout(),
                        KeyCode::Char('R') | KeyCode::Char('r') => {
                            app.set_layout(LayoutMode::Radar);
                        }
                        KeyCode::Char('P') | KeyCode::Char('p') => {
                            app.set_layout(LayoutMode::Performance);
                        }
                        KeyCode::Char('b') | KeyCode::Char('B') => {
                            app.toggle_radar_labels();
                        }
                        KeyCode::Char('m') => app.open_columns(),
                        KeyCode::Char('C') => app.open_config(),
                        KeyCode::Char('a') => {
                            app.add_watchlist_from_selected(&indices);
                        }
                        KeyCode::Char('g') | KeyCode::Char('G') => app.open_lookup(),
                        KeyCode::Char('W') | KeyCode::Char('w') => app.open_watchlist(),
                        KeyCode::Char('?') | KeyCode::Char('h') => app.open_help(),
                        KeyCode::Char('e') => match export::export_csv(&app, &indices) {
                            Ok(path) => {
                                info!("export csv {}", path);
                                app.set_last_export(path);
                            }
                            Err(err) => error!("export csv failed: {err}"),
                        },
                        KeyCode::Char('E') => match export::export_json(&app) {
                            Ok(path) => {
                                info!("export json {}", path);
                                app.set_last_export(path);
                            }
                            Err(err) => error!("export json failed: {err}"),
                        },
                        _ => {}
                    },
                    InputMode::Filter => match key.code {
                        KeyCode::Enter => app.apply_filter(),
                        KeyCode::Esc => app.cancel_filter(),
                        KeyCode::Backspace => app.backspace_filter(),
                        KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if ch == 'u' {
                                app.filter_edit.clear();
                            }
                        }
                        KeyCode::Char(ch) => app.push_filter_char(ch),
                        _ => {}
                    },
                    InputMode::Columns => match key.code {
                        KeyCode::Esc => app.close_columns(),
                        KeyCode::Char('m') => app.close_columns(),
                        KeyCode::Up => app.previous_column(),
                        KeyCode::Down => app.next_column(),
                        KeyCode::Enter | KeyCode::Char(' ') => app.toggle_column(),
                        _ => {}
                    },
                    InputMode::Help => match key.code {
                        KeyCode::Esc => app.close_help(),
                        KeyCode::Char('?') | KeyCode::Char('h') => app.close_help(),
                        KeyCode::Char('L') | KeyCode::Char('l') => app.open_legend(),
                        _ => {}
                    },
                    InputMode::Legend => match key.code {
                        KeyCode::Esc => app.close_legend(),
                        KeyCode::Char('L') | KeyCode::Char('l') => app.close_legend(),
                        KeyCode::Up => app.previous_cursor(ui::legend_len()),
                        KeyCode::Down => app.next_cursor(ui::legend_len()),
                        _ => {}
                    },
                    InputMode::Watchlist => match key.code {
                        KeyCode::Esc => app.close_watchlist(),
                        KeyCode::Char('W') | KeyCode::Char('w') => app.close_watchlist(),
                        KeyCode::Up => app.previous_watchlist_item(),
                        KeyCode::Down => app.next_watchlist_item(),
                        KeyCode::PageUp => app.watchlist_page_up(10),
                        KeyCode::PageDown => app.watchlist_page_down(10),
                        KeyCode::Char('a') => {
                            app.add_watchlist_from_selected(&indices);
                        }
                        KeyCode::Char('e') => {
                            app.toggle_watchlist_enabled_selected();
                        }
                        KeyCode::Char('n') => {
                            app.toggle_watchlist_notify_selected();
                        }
                        KeyCode::Char('d') => {
                            app.delete_watchlist_selected();
                        }
                        KeyCode::Char('s') => {
                            app.save_watchlist();
                        }
                        _ => {}
                    },
                    InputMode::Config => match key.code {
                        KeyCode::Esc => {
                            if app.config_editing {
                                app.cancel_config_edit();
                            } else {
                                app.close_config();
                            }
                        }
                        KeyCode::Char('w') | KeyCode::Char('S') => {
                            app.save_config();
                        }
                        KeyCode::Up => {
                            if !app.config_editing {
                                app.previous_config_item();
                            }
                        }
                        KeyCode::Down => {
                            if !app.config_editing {
                                app.next_config_item();
                            }
                        }
                        KeyCode::Enter => {
                            if app.config_editing {
                                app.apply_config_edit();
                            } else {
                                app.start_config_edit();
                            }
                        }
                        KeyCode::Backspace => {
                            if app.config_editing {
                                app.backspace_config();
                            }
                        }
                        KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if ch == 'u' && app.config_editing {
                                app.config_edit.clear();
                            }
                            if ch == 's' {
                                app.save_config();
                            }
                        }
                        KeyCode::Char(ch) => {
                            if app.config_editing {
                                app.push_config_char(ch);
                            }
                        }
                        _ => {}
                    },
                    InputMode::Lookup => match key.code {
                        KeyCode::Esc => app.cancel_lookup(),
                        KeyCode::Enter => {
                            if let Some(req) = app.prepare_lookup_request() {
                                if let Some(channels) = &lookup {
                                    let _ = channels.req_tx.send(req);
                                } else {
                                    app.apply_lookup_error("Lookup unavailable".to_string());
                                }
                            }
                        }
                        KeyCode::Backspace => app.backspace_lookup(),
                        KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if ch == 'u' {
                                app.lookup_input.clear();
                            }
                        }
                        KeyCode::Char(ch) => app.push_lookup_char(ch),
                        _ => {}
                    },
                },
                Event::Mouse(mouse) => {
                    handle_mouse(&mut app, &indices, mouse);
                }
                _ => {}
            }
            dirty = true;
        }

        if dirty {
            indices = app.visible_indices();
        }
        app.restore_selection_by_key(&indices);
        app.clamp_selection_to(indices.len());
        app.update_selection_key(&indices);

        if let Some(routes) = &routes {
            let now = SystemTime::now();
            if app.route_enabled() && app.route_refresh_due(now) {
                if app.route_tar1090() {
                    let _ = routes.req_tx.send(Vec::new());
                    app.mark_route_poll(now);
                } else {
                    let requests = app.collect_route_requests(&indices, now);
                    if !requests.is_empty() {
                        let _ = routes.req_tx.send(requests);
                        app.mark_route_poll(now);
                    }
                }
            }
        }

        let now = SystemTime::now();
        let draw_due = is_draw_due(now, last_draw, app.ui_interval);
        if dirty || draw_due {
            terminal.draw(|f| ui::ui(f, &mut app, &indices))?;
            app.advance_tick();
            last_draw = Some(now);
        }
    }
}

fn is_draw_due(now: SystemTime, last_draw: Option<SystemTime>, interval: Duration) -> bool {
    if interval.is_zero() {
        return true;
    }
    match last_draw {
        Some(last) => now
            .duration_since(last)
            .map(|d| d >= interval)
            .unwrap_or(true),
        None => true,
    }
}

pub struct RouteChannels {
    pub req_tx: Sender<Vec<RouteRequest>>,
    pub res_rx: Receiver<RouteMessage>,
}

pub struct LookupChannels {
    pub req_tx: Sender<LookupRequest>,
    pub res_rx: Receiver<LookupMessage>,
}

fn handle_mouse(app: &mut App, indices: &[usize], mouse: MouseEvent) {
    if app.input_mode != InputMode::Normal {
        return;
    }
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            app.previous_row(indices.len());
            app.update_selection_key(indices);
        }
        MouseEventKind::ScrollDown => {
            app.next_row(indices.len());
            app.update_selection_key(indices);
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(row) = app.table_row_at(mouse.row) {
                app.select_row(row, indices.len());
                app.update_selection_key(indices);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::is_draw_due;
    use std::time::{Duration, SystemTime};

    #[test]
    fn draw_due_when_interval_zero() {
        assert!(is_draw_due(SystemTime::now(), None, Duration::from_secs(0)));
    }

    #[test]
    fn draw_due_when_elapsed() {
        let now = SystemTime::now();
        let last = now - Duration::from_secs(2);
        assert!(is_draw_due(now, Some(last), Duration::from_secs(1)));
        assert!(!is_draw_due(now, Some(last), Duration::from_secs(5)));
    }
}
