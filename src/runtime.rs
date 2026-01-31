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

use crate::app::{App, InputMode};
use crate::export;
use crate::routes::{RouteMessage, RouteRequest};
use crate::storage;
use crate::model::ApiResponse;
use crate::ui;

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
) -> Result<()> {
    let tick_rate = Duration::from_millis(50);
    loop {
        while let Ok(message) = rx.try_recv() {
            match message {
                Ok(data) => app.apply_update(data),
                Err(err) => app.apply_error(err),
            }
        }

        if let Some(routes) = &routes {
            while let Ok(message) = routes.res_rx.try_recv() {
                match message {
                    RouteMessage::Results(results) => app.apply_routes(results),
                    RouteMessage::Error(err) => app.set_route_error(err),
                }
            }
        }

        let now = SystemTime::now();
        app.maybe_swap_snapshot(now);

        let indices = app.visible_indices();
        app.restore_selection_by_key(&indices);
        app.clamp_selection_to(indices.len());
        app.update_selection_key(&indices);

        terminal.draw(|f| ui::ui(f, &mut app, &indices))?;
        app.advance_tick();

        if let Some(routes) = &routes {
            let now = SystemTime::now();
            if app.route_enabled() && app.route_refresh_due(now) {
                if app.route_tar1090() {
                    let _ = routes.req_tx.send(Vec::new());
                } else {
                    let requests = app.collect_route_requests(&indices, now);
                    if !requests.is_empty() {
                        let _ = routes.req_tx.send(requests);
                    }
                }
                app.mark_route_poll(now);
            }
        }

        if event::poll(tick_rate)? {
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
                                    let _ = storage::save_favorites(path, &app.favorites);
                                }
                            }
                        }
                        KeyCode::Char('t') => app.toggle_theme(),
                        KeyCode::Char('l') => app.toggle_layout(),
                        KeyCode::Char('m') => app.open_columns(),
                        KeyCode::Char('C') => app.open_config(),
                        KeyCode::Char('?') | KeyCode::Char('h') => app.open_help(),
                        KeyCode::Char('e') => {
                            if let Ok(path) = export::export_csv(&app, &indices) {
                                app.set_last_export(path);
                            }
                        }
                        KeyCode::Char('E') => {
                            if let Ok(path) = export::export_json(&app) {
                                app.set_last_export(path);
                            }
                        }
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
                        KeyCode::Char('w') | KeyCode::Char('S') => app.save_config(),
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
                },
                Event::Mouse(mouse) => {
                    handle_mouse(&mut app, &indices, mouse);
                }
                _ => {}
            }
        }
    }
}

pub struct RouteChannels {
    pub req_tx: Sender<Vec<RouteRequest>>,
    pub res_rx: Receiver<RouteMessage>,
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
