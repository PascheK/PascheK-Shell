//! Text User Interface (TUI) for PascheK Shell
//!
//! This module implements a full-screen terminal UI built with ratatui and crossterm.
//! The TUI offers multiple screens:
//! - Home: landing page with quick actions
//! - Shell: a simple terminal-like input/output pane, with optional logs side panel
//! - Explorer: a file browser limited to a root directory
//! - Editor: a basic text editor with ropey for efficient editing
//! - Workspace: a split view combining Explorer and Editor with focus switching
//!
//! Interaction model:
//! - Global overlay for Help (ephemeral, closes on next key)
//! - Status bar with contextual hints
//! - Shell supports TUI commands prefixed with ':' (e.g., :q, :l, :h, :fs, :e <path>)
//! - TerminalPane supports input editing, history navigation, and cursor movement
//!
//! Error handling is user-friendly: most failures surface as messages in the
//! TerminalPane output or the Logs panel rather than panicking.

mod command_mode;
mod components;
mod state;

use crate::shell::{prompt::Theme, tui::state::Focus};
use command_mode::TuiCommandHandler;
use components::{
    editor::EditorView,
    explorer::FileExplorerView,
    home::HomeView,
    logs::LogPanel,
    status::StatusBar,
    terminal::TerminalPane,
};
use state::{EditorMode, Overlay, Screen, TuiState};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Starts the PascheK Shell TUI event loop.
///
/// Lifecycle:
/// 1. Enters alternate screen and enables raw mode
/// 2. Initializes TUI state and components
/// 3. Renders the current screen and processes input in a loop
/// 4. Restores the terminal on exit
///
/// Returns an io::Result so terminal errors are propagated to the caller.
pub fn start_tui() -> io::Result<()> {
    // Passage en mode TUI (écran alternatif + raw mode)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- État & composants ---
    let mut state = TuiState::default();
    // Démarrage sur la page d'accueil
    state.screen = Screen::Home;
    // Le focus sera appliqué quand on entrera en Workspace
    state.focus = Focus::Explorer;

    // Définir la racine: HOME (sinon fallback sur CWD)
    let home_root = home::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    state.explorer.root = home_root.clone();
    // Démarrer dans la racine
    state.explorer.cwd = state.explorer.root.clone();
    // (re)charger le listing
    FileExplorerView::refresh(&mut state.explorer);

    let mut status = StatusBar::new(Theme::default());
    let mut term = TerminalPane::new();
    let mut logs = LogPanel::new();
    let home = HomeView::default();

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    while state.running {
        terminal.draw(|f| {
            let area = f.area();

            // Layout vertical = zone principale + status
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(3)])
                .split(area);

            // --- Rendu par écran ---
            match state.screen {
                Screen::Home => {
                    home.render(f, chunks[0]);
                    // Hints par défaut
                    let hints = "[1] Shell  [2] Shell+Logs  [3] Aide  [5] Workspace  [4/q] Quitter";
                    status.set_hint(hints);
                    status.render(f, chunks[1]);
                }
                Screen::Workspace => {
                    // Split horizontal: explorer (30%) | editor (70%)
                    let cols = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                        .split(chunks[0]);

                    // Styles de bordure selon le focus
                    let explorer_focused = state.focus == Focus::Explorer;
                    let editor_focused = state.focus == Focus::Editor;

                    let explorer_border = if explorer_focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    let editor_border = if editor_focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };

                    // Marqueur dirty du fichier courant (onglet actif) pour l’explorer
                    let dirty = state
                        .tabs
                        .current()
                        .and_then(|ed| ed.path.as_ref().map(|p| (p.clone(), ed.dirty)));

                    // Rendu Explorer + Editor
                    FileExplorerView::render_with_border(
                        f,
                        cols[0],
                        &state.explorer,
                        dirty,
                        explorer_border,
                    );

                    // Construire une barre d'onglets multi-lignes pour tout afficher
                    let tab_names: Vec<String> = if state.tabs.tabs.is_empty() {
                        vec![String::from("[No Tabs]")]
                    } else {
                        state
                            .tabs
                            .tabs
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                let mut name = t
                                    .state
                                    .path
                                    .as_ref()
                                    .and_then(|p| p.file_name())
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("[No Name]")
                                    .to_string();
                                if t.state.dirty { name = format!("● {}", name); }
                                if i == state.tabs.current { name = format!("[{}]", name); }
                                name
                            })
                            .collect()
                    };
                    // Pack tabs names into multiple lines to fit width
                    let editor_area = cols[1];
                    let maxw = editor_area.width.saturating_sub(2) as usize; // account border
                    let mut lines: Vec<Line> = Vec::new();
                    if tab_names.len() == 1 {
                        lines.push(Line::from(tab_names[0].clone()));
                    } else {
                        let mut current = String::new();
                        for (idx, name) in tab_names.iter().enumerate() {
                            let sep = if current.is_empty() { "" } else { "  " };
                            let candidate_len = current.len() + sep.len() + name.len();
                            if candidate_len > maxw && !current.is_empty() {
                                lines.push(Line::from(std::mem::take(&mut current)));
                                current.push_str(name);
                            } else {
                                if !sep.is_empty() { current.push_str(sep); }
                                current.push_str(name);
                            }
                            if idx == tab_names.len() - 1 && !current.is_empty() {
                                lines.push(Line::from(std::mem::take(&mut current)));
                            }
                        }
                    }

                    // Hauteur dynamique: contenu (1..3 lignes) + 2 pour les bordures
                    let content_lines: u16 = (lines.len().max(1).min(3)) as u16;
                    let tab_height: u16 = content_lines + 2;
                    let vchunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(tab_height), Constraint::Min(3)])
                        .split(cols[1]);

                    let tabs_title = format!("Tabs ({})", state.tabs.tabs.len());
                    let tabs_widget = Paragraph::new(lines)
                        .block(Block::default().borders(Borders::ALL).border_style(editor_border).title(tabs_title));
                    f.render_widget(tabs_widget, vchunks[0]);

                    if let Some(ed) = state.tabs.current() {
                        EditorView::render_with_border(f, vchunks[1], ed, editor_border);
                    } else {
                        let p = Paragraph::new(Line::from(
                            "Aucun fichier ouvert — sélectionne un fichier à gauche ou tape :e <path>",
                        ))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(editor_border)
                                .title("Editor"),
                        );
                        f.render_widget(p, vchunks[1]);
                    }

                    // Hints dynamiques dans la status bar
                    let hints = match state.focus {
                        Focus::Explorer => "[Tab] Éditeur  [Entrée] Ouvrir  [.] Cachés  [q] Accueil",
                        Focus::Editor => "[Tab] Explorer  [Ctrl+S] Sauver  [Ctrl+F] Rechercher  [Ctrl+G] Aller à la ligne",
                    };
                    status.set_hint(hints);

                    // Status en bas
                    status.render(f, chunks[1]);
                }
                Screen::Shell => {
                    if state.show_logs {
                        let cols = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                            .split(chunks[0]);
                        term.render(f, cols[0]);
                        logs.render(f, cols[1]);
                    } else {
                        term.render(f, chunks[0]);
                    }
                    status.set_hint(
                        "Tape :fs pour Workspace, :e <path> pour ouvrir, :h Aide, :l Logs, :q Quitter",
                    );
                    status.render(f, chunks[1]);
                }
                Screen::Explorer => {
                    FileExplorerView::render(f, chunks[0], &state.explorer, None);
                    status.set_hint("[Tab] Éditeur  [Entrée] Ouvrir  [.] Cachés  [q] Quitter");
                    status.render(f, chunks[1]);
                }
                Screen::Editor => {
                    // Barre d'onglets + éditeur, comme en Workspace
                    let editor_area = chunks[0];
                    let tab_names: Vec<String> = if state.tabs.tabs.is_empty() {
                        vec![String::from("[No Tabs]")]
                    } else {
                        state
                            .tabs
                            .tabs
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                let mut name = t
                                    .state
                                    .path
                                    .as_ref()
                                    .and_then(|p| p.file_name())
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("[No Name]")
                                    .to_string();
                                if t.state.dirty { name = format!("● {}", name); }
                                if i == state.tabs.current { name = format!("[{}]", name); }
                                name
                            })
                            .collect()
                    };
                    let maxw = editor_area.width.saturating_sub(2) as usize;
                    let mut lines: Vec<Line> = Vec::new();
                    if tab_names.len() == 1 {
                        lines.push(Line::from(tab_names[0].clone()));
                    } else {
                        let mut current = String::new();
                        for (idx, name) in tab_names.iter().enumerate() {
                            let sep = if current.is_empty() { "" } else { "  " };
                            let candidate_len = current.len() + sep.len() + name.len();
                            if candidate_len > maxw && !current.is_empty() {
                                lines.push(Line::from(std::mem::take(&mut current)));
                                current.push_str(name);
                            } else {
                                if !sep.is_empty() { current.push_str(sep); }
                                current.push_str(name);
                            }
                            if idx == tab_names.len() - 1 && !current.is_empty() {
                                lines.push(Line::from(std::mem::take(&mut current)));
                            }
                        }
                    }
                    // Hauteur dynamique: contenu (1..3 lignes) + 2 pour les bordures
                    let content_lines: u16 = (lines.len().max(1).min(3)) as u16;
                    let tab_height: u16 = content_lines + 2;
                    let vchunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(tab_height), Constraint::Min(3)])
                        .split(chunks[0]);
                    let tabs_title = format!("Tabs ({})", state.tabs.tabs.len());
                    let tabs_widget = Paragraph::new(lines)
                        .block(Block::default().borders(Borders::ALL).title(tabs_title));
                    f.render_widget(tabs_widget, vchunks[0]);

                    if let Some(ed) = state.tabs.current() {
                        EditorView::render(f, vchunks[1], ed);
                    } else {
                        let p = Paragraph::new(Line::from("Éditeur sans buffer — ouvrez un fichier."))
                            .block(Block::default().borders(Borders::ALL).title("Editor"));
                        f.render_widget(p, vchunks[1]);
                    }
                    status.set_hint("[Ctrl+S] Sauver  [Ctrl+F] Rechercher  [Ctrl+G] Aller à la ligne  [Tab] Explorer");
                    status.render(f, chunks[1]);
                }
            }

            // Overlay d'aide (éphémère) — se ferme à la prochaine touche
            if state.overlay == Overlay::Help {
                let popup = centered_rect(60, 40, area);

                f.render_widget(Clear, popup);
                let text = vec![
                    Line::from("PascheK TUI — Aide"),
                    Line::from(""),
                    Line::from(":q        → Quitter"),
                    Line::from(":l        → Ouvrir/fermer les logs (sticky)"),
                    Line::from(":h        → Ouvrir/fermer cette aide (éphémère)"),
                    Line::from(":fs       → Ouvrir l’espace de travail (Explorer + Editeur)"),
                    Line::from(":e <path> → Ouvrir un fichier dans l’éditeur"),
                    Line::from(""),
                    Line::from("Cette fenêtre se fermera à la prochaine touche."),
                ];
                let p = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title("Help"));
                f.render_widget(p, popup);
            } else if state.overlay == Overlay::Input {
                let popup = centered_rect(60, 20, area);
                f.render_widget(Clear, popup);
                let label = state
                    .overlay_input
                    .as_ref()
                    .map(|i| match i.kind {
                        state::InputKind::NewEntry => "Nouveau (fichier ou dossier/) :",
                        state::InputKind::RenameEntry => "Renommer (nouveau nom) :",
                        state::InputKind::DeleteConfirm => "Confirmer suppression (tape 'y') :",
                        state::InputKind::SearchText => "Rechercher :",
                        state::InputKind::GotoLine => "Aller à la ligne :",
                    })
                    .unwrap_or("");
                let value = state
                    .overlay_input
                    .as_ref()
                    .map(|i| i.buffer.clone())
                    .unwrap_or_default();
                let text = vec![Line::from(label), Line::from(value)];
                let p = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title("Input"));
                f.render_widget(p, popup);
            }
        })?;

        // ----- Gestion des événements clavier -----
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_millis(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // 1) Accueil : navigation directe
                if state.screen == Screen::Home {
                    match key.code {
                        KeyCode::Char('1') => {
                            state.screen = Screen::Shell;
                        }
                        KeyCode::Char('2') => {
                            state.screen = Screen::Shell;
                            state.show_logs = true;
                        }
                        KeyCode::Char('3') => {
                            state.screen = Screen::Shell;
                            state.overlay = Overlay::Help;
                        }
                        KeyCode::Char('5') => {
                            state.screen = Screen::Workspace; // Workspace (pas Explorer)
                            state.focus = Focus::Explorer;
                        }
                        KeyCode::Char('4') | KeyCode::Char('q') => {
                            state.running = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                // 2) Overlay Help: se ferme à la prochaine touche
                if state.overlay == Overlay::Help {
                    state.overlay = Overlay::None;
                    continue;
                }

                // 2bis) Overlay Input: capter la saisie avant le reste
                if state.overlay == Overlay::Input {
                    match key.code {
                        KeyCode::Esc => { state.overlay = Overlay::None; state.overlay_input = None; }
                        KeyCode::Backspace => {
                            if let Some(inp) = state.overlay_input.as_mut() { inp.buffer.pop(); }
                        }
                        KeyCode::Enter => {
                            use std::fs;
                            if let Some(inp) = state.overlay_input.take() {
                                match inp.kind {
                                    state::InputKind::NewEntry => {
                                        let name = inp.buffer.trim();
                                        if !name.is_empty() {
                                            let path = state.explorer.cwd.join(name);
                                            let res = if name.ends_with('/') { fs::create_dir_all(&path) } else { fs::File::create(&path).map(|_| ()) };
                                            let _ = res; // Optionally handle errors
                                            FileExplorerView::refresh(&mut state.explorer);
                                        }
                                    }
                                    state::InputKind::RenameEntry => {
                                        if let Some(entry) = state.explorer.entries.get(state.explorer.selected) {
                                            if entry.name != ".." {
                                                let from = state.explorer.cwd.join(&entry.name);
                                                let to = state.explorer.cwd.join(inp.buffer.trim());
                                                let _ = std::fs::rename(from, to);
                                                FileExplorerView::refresh(&mut state.explorer);
                                            }
                                        }
                                    }
                                    state::InputKind::DeleteConfirm => {
                                        if inp.buffer.trim().eq_ignore_ascii_case("y") {
                                            if let Some(entry) = state.explorer.entries.get(state.explorer.selected) {
                                                if entry.name != ".." {
                                                    let path = state.explorer.cwd.join(&entry.name);
                                                    let _ = if entry.is_dir { std::fs::remove_dir_all(path) } else { std::fs::remove_file(path) };
                                                    FileExplorerView::refresh(&mut state.explorer);
                                                }
                                            }
                                        }
                                    }
                                    state::InputKind::SearchText => {
                                        let q = inp.buffer;
                                        if !q.is_empty() {
                                            if let Some(ed) = state.tabs.current_mut() {
                                                ed.last_search = Some(q.clone());
                                                // Cherche à partir de la position courante (ligne courante)
                                                let start_line = ed.cursor_row;
                                                let total = ed.buffer.len_lines();
                                                let mut found: Option<usize> = None;
                                                for row in start_line..total {
                                                    let mut txt = ed.buffer.line(row).to_string();
                                                    if txt.ends_with('\n') { txt.pop(); }
                                                    if txt.contains(&q) { found = Some(row); break; }
                                                }
                                                if found.is_none() {
                                                    for row in 0..start_line {
                                                        let mut txt = ed.buffer.line(row).to_string();
                                                        if txt.ends_with('\n') { txt.pop(); }
                                                        if txt.contains(&q) { found = Some(row); break; }
                                                    }
                                                }
                                                if let Some(row) = found {
                                                    ed.cursor_row = row;
                                                    ed.cursor_col = 0;
                                                    if ed.cursor_row < ed.scroll_row { ed.scroll_row = ed.cursor_row; }
                                                }
                                            }
                                        }
                                    }
                                    state::InputKind::GotoLine => {
                                        if let Ok(n) = inp.buffer.trim().parse::<usize>() {
                                            if let Some(ed) = state.tabs.current_mut() {
                                                let line = n.saturating_sub(1).min(ed.buffer.len_lines().saturating_sub(1));
                                                ed.cursor_row = line;
                                                ed.cursor_col = 0;
                                                if ed.cursor_row < ed.scroll_row { ed.scroll_row = ed.cursor_row; }
                                            }
                                        }
                                    }
                                }
                            }
                            state.overlay = Overlay::None;
                        }
                        KeyCode::Char(c) => {
                            if let Some(inp) = state.overlay_input.as_mut() { inp.buffer.push(c); }
                        }
                        _ => {}
                    }
                    continue;
                }

                // 3) Écran Explorer : navigation & ouverture
                if state.screen == Screen::Explorer {
                    use KeyCode::*;
                    match key.code {
                        Char('j') | Down => FileExplorerView::move_down(&mut state.explorer),
                        Char('k') | Up => FileExplorerView::move_up(&mut state.explorer),
                        Char('h') | Backspace => FileExplorerView::go_up(&mut state.explorer),
                        Char('N') => {
                            state.overlay = Overlay::Input;
                            state.overlay_input = Some(state::InputOverlay { kind: state::InputKind::NewEntry, buffer: String::new() });
                        }
                        Char('R') => {
                            state.overlay = Overlay::Input;
                            state.overlay_input = Some(state::InputOverlay { kind: state::InputKind::RenameEntry, buffer: String::new() });
                        }
                        Delete => {
                            state.overlay = Overlay::Input;
                            state.overlay_input = Some(state::InputOverlay { kind: state::InputKind::DeleteConfirm, buffer: String::new() });
                        }
                        Char('.') => {
                            state.explorer.show_hidden = !state.explorer.show_hidden;
                            FileExplorerView::refresh(&mut state.explorer);
                        }
                        Char('l') | Enter => {
                            if let Some(path) = FileExplorerView::activate(&mut state.explorer) {
                                match EditorView::open_path(path, &state.explorer.root) {
                                    Ok(ed) => {
                                        state.tabs.open_or_focus(ed);
                                        state.screen = Screen::Workspace; // bascule en Workspace
                                        state.focus = Focus::Editor;
                                    }
                                    Err(_e) => {
                                        // TODO: pousser un message dans logs/term
                                    }
                                }
                            }
                        }
                        Char('q') | Esc => {
                            state.screen = Screen::Home;
                        }
                        _ => {}
                    }
                    continue;
                }

                // 4) Écran Workspace : focus & raccourcis
                if state.screen == Screen::Workspace {
                    match state.focus {
                        Focus::Explorer => {
                            use crossterm::event::KeyCode::*;
                            match key.code {
                                KeyCode::Tab => {
                                    state.focus = Focus::Editor;
                                } // Tab -> focus à droite
                                Char('N') => {
                                    state.overlay = Overlay::Input;
                                    state.overlay_input = Some(state::InputOverlay { kind: state::InputKind::NewEntry, buffer: String::new() });
                                }
                                Char('R') => {
                                    state.overlay = Overlay::Input;
                                    state.overlay_input = Some(state::InputOverlay { kind: state::InputKind::RenameEntry, buffer: String::new() });
                                }
                                Delete => {
                                    state.overlay = Overlay::Input;
                                    state.overlay_input = Some(state::InputOverlay { kind: state::InputKind::DeleteConfirm, buffer: String::new() });
                                }
                                Char('j') | Down => FileExplorerView::move_down(&mut state.explorer),
                                Char('k') | Up => FileExplorerView::move_up(&mut state.explorer),
                                Char('h') | Backspace => FileExplorerView::go_up(&mut state.explorer),
                                Char('.') => {
                                    state.explorer.show_hidden = !state.explorer.show_hidden;
                                    FileExplorerView::refresh(&mut state.explorer);
                                }
                                Char('l') | Enter => {
                                    if let Some(path) = FileExplorerView::activate(&mut state.explorer) {
                                        match EditorView::open_path(path, &state.explorer.root) {
                                            Ok(ed) => {
                                                state.tabs.open_or_focus(ed);
                                                state.focus = Focus::Editor;
                                            }
                                            Err(_e) => { /* TODO: logs */ }
                                        }
                                    }
                                }
                                Char('q') | Esc => {
                                    // Quitter le Workspace -> revenir à l'accueil
                                    state.screen = Screen::Home;
                                }
                                _ => {}
                            }
                        }
                        Focus::Editor => {
                            use crossterm::event::{KeyCode::*, KeyModifiers};
                            let modifiers = key.modifiers;

                            if modifiers.contains(KeyModifiers::CONTROL) {
                                match key.code {
                                    Char('s') => {
                                        if let Some(ed) = state.tabs.current_mut() { let _ = EditorView::save(ed); }
                                    } // Ctrl+S
                                    Char('z') => { if let Some(ed) = state.tabs.current_mut() { EditorView::undo(ed); } } // Ctrl+Z
                                    Char('y') => { if let Some(ed) = state.tabs.current_mut() { EditorView::redo(ed); } } // Ctrl+Y
                                    Char('w') => {
                                        state.tabs.close_current();
                                        if state.tabs.is_empty() { state.focus = Focus::Explorer; }
                                    } // Ctrl+W
                                    PageDown => { state.tabs.next(); } // Ctrl+PageDown
                                    PageUp => { state.tabs.prev(); }   // Ctrl+PageUp
                                    KeyCode::Tab => { state.tabs.next(); } // Ctrl+Tab
                                    KeyCode::BackTab => { state.tabs.prev(); } // Ctrl+Shift+Tab
                                    _ => {}
                                }
                                continue;
                            }

                            // Fallback: Alt+Left/Right pour naviguer entre onglets sur macOS/terminaux qui ne reportent pas Ctrl+Tab
                            if modifiers.contains(KeyModifiers::ALT) {
                                match key.code {
                                    Left => { state.tabs.prev(); continue; }
                                    Right => { state.tabs.next(); continue; }
                                    _ => {}
                                }
                            }

                            // F-keys fallback (macOS Terminal friendly): F5 ← précédent, F6 → suivant
                            match key.code {
                                KeyCode::F(5) => { state.tabs.prev(); continue; }
                                KeyCode::F(6) => { state.tabs.next(); continue; }
                                _ => {}
                            }

                            if let Some(ed) = state.tabs.current_mut() {
                                match key.code {
                                    Left => EditorView::move_left(ed),
                                    Right => EditorView::move_right(ed),
                                    Up => EditorView::move_up(ed),
                                    Down => EditorView::move_down(ed),
                                    Backspace => EditorView::backspace(ed),
                                    Enter => EditorView::insert_newline(ed),
                                    KeyCode::Tab | Esc => {
                                        state.focus = Focus::Explorer;
                                    } // Tab/Esc → focus à gauche
                                    Char(c) => EditorView::insert_char(ed, c),
                                    _ => {}
                                }
                            } else if let KeyCode::Tab = key.code {
                                state.focus = Focus::Explorer;
                            }
                        }
                    }
                    continue;
                }

                // 5) Écran Editor : mêmes raccourcis que Workspace/Editor, mais sur l'onglet courant
                if state.screen == Screen::Editor {
                    use crossterm::event::{KeyCode::*, KeyModifiers};

                    // Raccourcis globaux (navigation onglets, save, close)
                    let modifiers = key.modifiers;
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        match key.code {
                            Char('s') => { if let Some(ed) = state.tabs.current_mut() { let _ = EditorView::save(ed); } }
                            Char('z') => { if let Some(ed) = state.tabs.current_mut() { EditorView::undo(ed); } }
                            Char('y') => { if let Some(ed) = state.tabs.current_mut() { EditorView::redo(ed); } }
                            Char('f') => { state.overlay = Overlay::Input; state.overlay_input = Some(state::InputOverlay { kind: state::InputKind::SearchText, buffer: String::new() }); }
                            Char('g') => { state.overlay = Overlay::Input; state.overlay_input = Some(state::InputOverlay { kind: state::InputKind::GotoLine, buffer: String::new() }); }
                            Char('w') => {
                                state.tabs.close_current();
                                if state.tabs.is_empty() { state.screen = Screen::Workspace; state.focus = Focus::Explorer; }
                            }
                            PageDown => { state.tabs.next(); }
                            PageUp => { state.tabs.prev(); }
                            KeyCode::Tab => { state.tabs.next(); }
                            KeyCode::BackTab => { state.tabs.prev(); }
                            _ => {}
                        }
                        continue;
                    }

                    // Alt+←/→ et F5/F6 (fallbacks pour macOS Terminal)
                    if modifiers.contains(KeyModifiers::ALT) {
                        match key.code { Left => { state.tabs.prev(); }, Right => { state.tabs.next(); }, _ => {} }
                        continue;
                    }
                    match key.code { KeyCode::F(5) => { state.tabs.prev(); continue; }, KeyCode::F(6) => { state.tabs.next(); continue; }, _ => {} }

                    // Édition du buffer de l'onglet courant
                    let mut open_path_req: Option<PathBuf> = None;
                    {
                        if let Some(ed) = state.tabs.current_mut() {
                        use KeyCode::*;
                        match ed.mode {
                            EditorMode::Normal => match key.code {
                                Char('i') => ed.mode = EditorMode::Insert,
                                Char(':') => { ed.mode = EditorMode::Command; ed.cmdline.clear(); }
                                Left => EditorView::move_left(ed),
                                Right => EditorView::move_right(ed),
                                Up => EditorView::move_up(ed),
                                Down => EditorView::move_down(ed),
                                Esc | KeyCode::Tab => { state.screen = Screen::Workspace; state.focus = Focus::Explorer; }
                                _ => {}
                            },
                            EditorMode::Insert => match key.code {
                                Esc => ed.mode = EditorMode::Normal,
                                Enter => EditorView::insert_newline(ed),
                                Backspace => EditorView::backspace(ed),
                                Left => EditorView::move_left(ed),
                                Right => EditorView::move_right(ed),
                                Up => EditorView::move_up(ed),
                                Down => EditorView::move_down(ed),
                                Char(c) => EditorView::insert_char(ed, c),
                                _ => {}
                            },
                            EditorMode::Command => match key.code {
                                Enter => {
                                    let cmd = ed.cmdline.trim();
                                    match cmd {
                                        "q" => { state.screen = Screen::Workspace; state.focus = Focus::Explorer; }
                                        "w" => { let _ = EditorView::save(ed); }
                                        "wq" => { let _ = EditorView::save(ed); state.screen = Screen::Workspace; state.focus = Focus::Explorer; }
                                        other if other.starts_with("e ") => {
                                            let p = PathBuf::from(other.trim_start_matches("e ").trim());
                                            open_path_req = Some(p);
                                        }
                                        _ => {}
                                    }
                                    ed.mode = EditorMode::Normal; ed.cmdline.clear();
                                }
                                Esc => { ed.mode = EditorMode::Normal; ed.cmdline.clear(); }
                                Backspace => { ed.cmdline.pop(); }
                                Char(c) => { ed.cmdline.push(c); }
                                _ => {}
                            },
                        }
                        }
                    }
                    if let Some(p) = open_path_req.take() {
                        if let Ok(new_ed) = EditorView::open_path(p, &state.explorer.root) { state.tabs.open_or_focus(new_ed); }
                    }
                    continue;
                }

                // 6) Écran Shell : édition / exécution
                match key.code {
                    KeyCode::Esc => state.running = false,

                    // Scroll du terminal (ou logs avec Shift)
                    KeyCode::PageUp => {
                        if state.show_logs && key.modifiers.contains(KeyModifiers::SHIFT) {
                            logs.scroll_up();
                        } else {
                            term.scroll_up();
                        }
                    }
                    KeyCode::PageDown => {
                        if state.show_logs && key.modifiers.contains(KeyModifiers::SHIFT) {
                            logs.scroll_down();
                        } else {
                            term.scroll_down();
                        }
                    }

                    // Édition de la ligne
                    KeyCode::Left => term.move_left(),
                    KeyCode::Right => term.move_right(),
                    KeyCode::Backspace => term.backspace(),
                    KeyCode::Delete => term.delete_forward(),
                    KeyCode::Home => term.move_to_start(),
                    KeyCode::End => term.move_to_end(),

                    // Historique (↑/↓)
                    KeyCode::Up => term.history_up(),
                    KeyCode::Down => term.history_down(),

                    // Validation
                    KeyCode::Enter => {
                        let line = term.current_line().trim().to_string();

                        if line.starts_with(':') {
                            // Commandes TUI (ex: :q, :l, :h) + raccourcis workspace/editor
                            if line == ":fs" || line == ":files" {
                                state.screen = Screen::Workspace;
                                state.focus = Focus::Explorer;
                            } else if let Some(rest) = line.strip_prefix(":e ") {
                                let path = PathBuf::from(rest.trim());
                                match EditorView::open_path(path, &state.explorer.root) {
                                    Ok(ed) => {
                                        state.tabs.open_or_focus(ed);
                                        state.screen = Screen::Workspace;
                                        state.focus = Focus::Editor;
                                    }
                                    Err(e) => {
                                        term.push_output(format!(":e error: {}", e));
                                    }
                                }
                            } else {
                                let mut handler = TuiCommandHandler { state: &mut state, logs: &mut logs };
                                handler.execute(&line);
                            }
                        } else if !line.is_empty() {
                            // Commande shell réelle (simple)
                            term.push_output(format!("$ {}", line));
                            term.push_history_if_new(&line);
                            run_shell_like(&line, &mut term, &mut logs);
                        }
                        term.clear_input();
                    }

                    // Saisie
                    KeyCode::Char(c) => term.insert_char(c),

                    _ => {}
                }

                // Raccourcis Ctrl-* (à traiter en dehors du match par code)
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('a') => term.move_to_start(), // Ctrl+A
                        KeyCode::Char('e') => term.move_to_end(),   // Ctrl+E
                        KeyCode::Char('l') => term.clear_output(),  // Ctrl+L
                        _ => {}
                    }
                }

            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Restauration du terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Compute a centered rectangle that takes `percent_x` by `percent_y` of the given area.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1]);
    h[1]
}

/// Minimal shell-like command execution used by the Shell screen.
///
/// Behavior:
/// - Implements a built-in `cd <path>` that changes process CWD
/// - Otherwise runs the command via PATH, capturing stdout/stderr
/// - Prints outputs to the Terminal pane; logs failed execution
fn run_shell_like(line: &str, term: &mut TerminalPane, logs: &mut LogPanel) {
    let mut parts = line.split_whitespace();
    if let Some(cmd) = parts.next() {
        let args: Vec<&str> = parts.collect();

        if cmd == "cd" {
            use std::env;
            if let Some(path) = args.get(0) {
                match env::set_current_dir(path) {
                    Ok(()) => term.push_output(format!("(cd) -> {}", path)),
                    Err(e) => term.push_output(format!("cd: {}: {}", path, e)),
                }
            } else {
                term.push_output("usage: cd <path>");
            }
            return;
        }

        use std::process::Command;
        match Command::new(cmd).args(&args).output() {
            Ok(out) => {
                if !out.stdout.is_empty() {
                    term.push_output(String::from_utf8_lossy(&out.stdout).to_string());
                }
                if !out.stderr.is_empty() {
                    term.push_output(String::from_utf8_lossy(&out.stderr).to_string());
                }
            }
            Err(e) => {
                term.push_output(format!("command not found: {} ({})", cmd, e));
                logs.add(format!("exec error: {} {:?}", cmd, e));
            }
        }
    }
}
