use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Champ de saisie simple
pub struct InputField {
    buffer: String,
    cursor_pos: usize,
}

impl InputField {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor_pos: 0,
        }
    }

    pub fn input(&mut self, c: char) {
        self.buffer.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.buffer.remove(self.cursor_pos);
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor_pos = 0;
    }

    pub fn get_value(&self) -> &str {
        &self.buffer
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let line = Line::from(Span::styled(
            format!("> {}", self.buffer),
            Style::default().fg(Color::Cyan),
        ));

        let paragraph = Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL).title("Input"));

        f.render_widget(paragraph, area);
    }
}