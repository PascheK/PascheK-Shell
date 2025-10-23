//! Terminal-like pane used by the PascheK Shell TUI.
//!
//! Responsibilities:
//! - Render a scrollable output area and an input line
//! - Provide simple input editing (left/right, backspace, delete)
//! - Maintain a command history navigable with Up/Down
//! - Expose helpers used by the TUI event loop (clear, scroll, etc.)

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Interactive terminal pane with output buffer, input editor, and command history.
pub struct TerminalPane {
    output: Vec<String>,
    scroll: usize,
    input: String,
    cursor: usize,
    // Command history (newest at the end)
    history: Vec<String>,
    // When navigating history: current index into history or None when editing fresh input
    history_pos: Option<usize>,
}

impl TerminalPane {
    /// Create a new terminal pane with a welcome message.
    pub fn new() -> Self {
        Self {
            output: vec![
                "Welcome to PascheK Shell TUI".into(),
                "Tape :h pour l’aide, :l pour les logs, :q pour quitter.".into(),
            ],
            scroll: 0,
            input: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_pos: None,
        }
    }

    /// Render the terminal output and input line with borders and titles.
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        let visible: Vec<Line> = self
            .output
            .iter()
            .rev()
            .skip(self.scroll)
            .take(200)
            .rev()
            .map(|l| Line::from(Span::raw(l)))
            .collect();

        let out = Paragraph::new(visible)
            .block(Block::default().borders(Borders::ALL).title("Terminal"));
        f.render_widget(out, chunks[0]);

        let prompted = format!("$ {}", self.input);
        let input_line = Paragraph::new(Line::from(Span::styled(
            prompted,
            Style::default().fg(Color::Cyan),
        )))
        .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input_line, chunks[1]);
    }

    // Input
    /// Insert a character at the cursor position (like typical terminals)
    pub fn insert_char(&mut self, c: char) { self.input.insert(self.cursor, c); self.cursor += 1; }
    /// Delete character before the cursor, if any
    pub fn backspace(&mut self) { if self.cursor > 0 { self.cursor -= 1; self.input.remove(self.cursor); } }
    /// Delete character under the cursor, if any
    pub fn delete_forward(&mut self) { if self.cursor < self.input.len() { self.input.remove(self.cursor); } }
    /// Move cursor one position left
    pub fn move_left(&mut self) { if self.cursor > 0 { self.cursor -= 1; } }
    /// Move cursor one position right
    pub fn move_right(&mut self) { if self.cursor < self.input.len() { self.cursor += 1; } }
    /// Move cursor to start of line
    pub fn move_to_start(&mut self) { self.cursor = 0; }
    /// Move cursor to end of line
    pub fn move_to_end(&mut self) { self.cursor = self.input.len(); }
    /// Clear input buffer and reset history navigation
    pub fn clear_input(&mut self) { self.input.clear(); self.cursor = 0; self.history_pos = None; }
    /// Borrow the current input line
    pub fn current_line(&self) -> &str { &self.input }
    /// Replace input line and set cursor at end
    fn set_input_from_history(&mut self, s: String) { self.input = s; self.cursor = self.input.len(); }

    // Output
    /// Append a line to the terminal output
    pub fn push_output<S: Into<String>>(&mut self, s: S) { self.output.push(s.into()); }
    /// Clear all output lines
    pub fn clear_output(&mut self) { self.output.clear(); }
    /// Scroll output one step up (older messages)
    pub fn scroll_up(&mut self) { if self.scroll < self.output.len().saturating_sub(1) { self.scroll += 1; } }
    /// Scroll output one step down (newer messages)
    pub fn scroll_down(&mut self) { if self.scroll > 0 { self.scroll -= 1; } }

    // History
    /// Push the executed command to history if not empty and not a duplicate of the last entry
    pub fn push_history_if_new(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() { return; }
        if self.history.last().map(|s| s.as_str()) != Some(trimmed) {
            self.history.push(trimmed.to_string());
        }
        self.history_pos = None;
    }
    /// Navigate one step up in history (older command). If starting fresh, jump to last.
    pub fn history_up(&mut self) {
        if self.history.is_empty() { return; }
        match self.history_pos {
            None => {
                let i = self.history.len() - 1;
                self.history_pos = Some(i);
                self.set_input_from_history(self.history[i].clone());
            }
            Some(i) => {
                if i > 0 {
                    let ni = i - 1;
                    self.history_pos = Some(ni);
                    self.set_input_from_history(self.history[ni].clone());
                }
            }
        }
    }
    /// Navigate one step down in history (newer command). Past the newest resets to empty input.
    pub fn history_down(&mut self) {
        if let Some(i) = self.history_pos {
            if i + 1 < self.history.len() {
                let ni = i + 1;
                self.history_pos = Some(ni);
                self.set_input_from_history(self.history[ni].clone());
            } else {
                // Exited history back to fresh input
                self.history_pos = None;
                self.clear_input();
            }
        }
    }
}