use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::shell::prompt::Theme;

/// Status bar displayed at the bottom of every screen.
///
/// Left side shows the shell name and current time; right side displays
/// contextual hints controlled by the parent screen.
pub struct StatusBar {
    theme: Theme,
    right_hint: String,
}

impl StatusBar {
    /// Create a new status bar with the given prompt Theme.
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            right_hint: String::from(""),
        }
    }

    /// Update the right-hand hint text.
    pub fn set_hint<S: Into<String>>(&mut self, s: S) {
        self.right_hint = s.into();
    }

    /// Render the status bar into the provided area.
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let left = Paragraph::new(Line::from(format!(
            " PascheK Shell • {}",
            Local::now().format("%H:%M:%S")
        )))
        .block(Block::default().borders(Borders::ALL).title("Status"));

        let right = Paragraph::new(Line::from(self.right_hint.clone()))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(left, cols[0]);
        f.render_widget(right, cols[1]);
    }
}