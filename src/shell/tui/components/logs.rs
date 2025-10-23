use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Simple log panel that shows timestamped or raw entries, scrollable.
pub struct LogPanel {
    entries: Vec<String>,
    scroll: usize,
}

impl LogPanel {
    /// Create an empty log panel
    pub fn new() -> Self { Self { entries: vec![], scroll: 0 } }
    /// Append a log entry
    pub fn add<S: Into<String>>(&mut self, s: S) { self.entries.push(s.into()); }
    /// Remove all log entries
    pub fn clear(&mut self) { self.entries.clear(); }
    /// Scroll one step up (older)
    pub fn scroll_up(&mut self) {
        if self.scroll < self.entries.len().saturating_sub(1) { self.scroll += 1; }
    }
    /// Scroll one step down (newer)
    pub fn scroll_down(&mut self) { if self.scroll > 0 { self.scroll -= 1; } }

    /// Render the logs list in the given area
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let lines: Vec<Line> = self.entries
            .iter()
            .rev()
            .skip(self.scroll)
            .take(100)
            .rev()
            .map(|l| Line::from(Span::raw(l)))
            .collect();

        let p = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .style(Style::default().fg(Color::White));
        f.render_widget(p, area);
    }
}