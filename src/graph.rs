use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Sparkline};
use ratatui::Frame;

use crate::app::App;

pub struct GraphTheme {
    pub accent: Color,
    pub warn: Color,
    pub panel_bg: Color,
}

pub fn render_performance_body(f: &mut Frame, area: Rect, app: &App, theme: &GraphTheme) {
    let snapshot = app.performance_snapshot();
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Min(7),
        ])
        .split(area);

    let msg_title = match snapshot.latest_msg_rate {
        Some(rate) => format!("MESSAGES {rate:.1}/s"),
        None => "MESSAGES --".to_string(),
    };
    render_sparkline(
        f,
        sections[0],
        &msg_title,
        &snapshot.msg_rate,
        theme.accent,
        theme.panel_bg,
    );

    let flights_title = format!("FLIGHTS {}", snapshot.latest_flights);
    render_sparkline(
        f,
        sections[1],
        &flights_title,
        &snapshot.flights,
        Color::Cyan,
        theme.panel_bg,
    );

    let signal_title = match (snapshot.latest_signal, snapshot.latest_signal_rsi) {
        (Some(rssi), Some(rsi)) => {
            format!("SIGNAL avg {rssi:.1} dB | RSI {rsi:.0} (scale -50..0)")
        }
        (Some(rssi), None) => format!("SIGNAL avg {rssi:.1} dB | RSI -- (scale -50..0)"),
        (None, Some(rsi)) => format!("SIGNAL avg -- | RSI {rsi:.0} (scale -50..0)"),
        (None, None) => "SIGNAL avg -- | RSI -- (scale -50..0)".to_string(),
    };
    render_sparkline(
        f,
        sections[2],
        &signal_title,
        &snapshot.signal,
        theme.warn,
        theme.panel_bg,
    );
}

fn render_sparkline(f: &mut Frame, area: Rect, title: &str, data: &[u64], fg: Color, bg: Color) {
    let (spark_data, spark_max) = sparkline_tail(data, area.width);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title);
    let graph = Sparkline::default()
        .block(block)
        .data(&spark_data)
        .max(spark_max)
        .style(Style::default().fg(fg).bg(bg));
    f.render_widget(graph, area);
}

fn sparkline_tail(data: &[u64], width: u16) -> (Vec<u64>, u64) {
    let width = width.saturating_sub(2).max(1) as usize;
    if data.is_empty() {
        return (vec![0], 1);
    }
    let start = data.len().saturating_sub(width);
    let mut tail: Vec<u64> = data[start..].to_vec();
    if tail.len() < width {
        let mut padded = vec![0; width - tail.len()];
        padded.append(&mut tail);
        tail = padded;
    }
    let max_value = tail.iter().copied().max().unwrap_or(1).max(1);
    (tail, max_value)
}

#[cfg(test)]
mod tests {
    use super::sparkline_tail;

    #[test]
    fn sparkline_tail_pads_and_limits_width() {
        let data = vec![1, 2, 3];
        let (tail, max) = sparkline_tail(&data, 6);
        assert_eq!(tail.len(), 4);
        assert_eq!(tail, vec![0, 1, 2, 3]);
        assert_eq!(max, 3);
    }

    #[test]
    fn sparkline_tail_trims_old_values() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let (tail, max) = sparkline_tail(&data, 4);
        assert_eq!(tail.len(), 2);
        assert_eq!(tail, vec![5, 6]);
        assert_eq!(max, 6);
    }
}
