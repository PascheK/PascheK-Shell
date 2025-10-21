use chrono::Local;
use std::env;
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    layout::Rect,
    text::{Span, Line},
    style::{Color, Style},
    Frame,
};

use crate::shell::prompt::theme::Theme;

/// Données affichées dans la barre d’état.
pub struct StatusBar {
    pub theme: Theme,
}

impl StatusBar {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Rendu principal du panneau d’état.
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Récupère l’heure locale
        let time = Local::now().format("%H:%M:%S").to_string();

        // Répertoire courant (simplifié)
        let cwd = env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "/".to_string());

        // Thème actif (affiché par sa couleur shell)
        let theme_color = self.theme.shell_color.to_ansi_color();

        // Composition de la ligne
        let line = Line::from(vec![
            Span::styled(format!(" "), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("  PascheK Shell "), Style::default().fg(theme_color)),
            Span::styled("  ", Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(format!("📂 {cwd}"), Style::default().fg(Color::Cyan)),
            Span::raw("   "),
            Span::styled(format!("🕓 {time}"), Style::default().fg(Color::Yellow)),
        ]);

        let widget = Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL).title("Status"));

        f.render_widget(widget, area);
    }
}