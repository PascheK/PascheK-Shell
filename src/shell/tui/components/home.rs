use ratatui::{
    layout::{Layout, Constraint, Direction, Rect},
    style::{Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Default)]
/// Landing page view with quick key hints.
pub struct HomeView;

impl HomeView {
    /// Render the centered homepage panel with navigation hints.
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // centre un rectangle pour le contenu
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(area)[1];

        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(outer)[1];

        let lines = vec![
            Line::from(Span::styled("PascheK Shell — Accueil", Style::default().fg(Color::LightCyan))),
            Line::from(""),
            Line::from("1) Démarrer le shell"),
            Line::from("2) Ouvrir les logs"),
            Line::from("3) Aide"),
            Line::from("4) Quitter"),
            Line::from(""),
            Line::from("Astuce : vous pouvez aussi taper :l, :h, :q dans le shell."),
        ];

        let p = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Accueil"));

        f.render_widget(p, inner);
    }
}