use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    text::{Span, Line}, // ✅ ici
    style::{Style, Color},
};
use std::io::{self};
use std::time::{Duration, Instant};

pub fn start_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut logs: Vec<String> = vec![];
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let size = f.area(); // ✅
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            let status = Paragraph::new("PascheK Shell — TUI mode")
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status, chunks[0]);

            // ✅ Correction ici
            let log_text: Vec<Line> = logs.iter().map(|l| Line::from(Span::raw(l.as_str()))).collect();
            let log_block = Paragraph::new(log_text)
                .block(Block::default().borders(Borders::ALL).title("Logs"));
            f.render_widget(log_block, chunks[1]);

            let input = Paragraph::new(Line::from(Span::styled(
                "Press 'q' to quit, 'l' to add a log",
                Style::default().fg(Color::Cyan),
            )))
            .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, chunks[2]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('l') => logs.push("New log entry!".into()),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}