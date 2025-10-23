//! File Explorer component: lists directory entries under a fixed root, with selection.
//!
//! Features:
//! - Root confinement: prevents leaving a configured root path
//! - Optional display of hidden files (dotfiles)
//! - Sorted entries: directories first, then files, case-insensitive by name
//! - Special ".." entry to go up (hidden at root)
use std::fs;
use std::path::{Path, PathBuf};

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::shell::tui::state::{DirEntryView, FileExplorerState};

/// Stateless explorer renderer and helper actions (refresh, navigate, activate).
pub struct FileExplorerView;

/// Check whether `path` stays within `root` boundary (canonicalized).
fn within_root(root: &Path, path: &Path) -> bool {
    let r = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    p.starts_with(&r)
}

/// Pretty-print a path relative-ish to root, replacing home prefix with `~` and truncating.
fn short_path(p: &Path, _root: &Path) -> String {
    let display = p.display().to_string();
    if let Some(home) = home::home_dir() {
        if let (Ok(cp), Ok(ch)) = (p.canonicalize(), home.canonicalize()) {
            if cp.starts_with(&ch) {
                return display.replacen(&ch.display().to_string(), "~", 1);
            }
        }
    }
    // Tronque si trop long
    if display.len() > 60 {
        let tail = &display[display.len().saturating_sub(60)..];
        format!("…{}", tail)
    } else {
        display
    }
}

impl FileExplorerView {
    /// Refresh the entries for the current working directory, applying filters and sorting.
    pub fn refresh(state: &mut FileExplorerState) {
        let cwd = if state.cwd.as_os_str().is_empty() {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        } else {
            state.cwd.clone()
        };

        let mut entries: Vec<DirEntryView> = Vec::new();

        // N'ajoute ".." que si on n'est pas à la racine
        if cwd != state.root {
            entries.push(DirEntryView {
                name: String::from(".."),
                is_dir: true,
            });
        }

        if let Ok(rd) = fs::read_dir(&cwd) {
            for e in rd.flatten() {
                let meta = e.metadata().ok();
                let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let name = e.file_name().to_string_lossy().to_string();

                if !state.show_hidden && name.starts_with('.') {
                    continue;
                }

                entries.push(DirEntryView { name, is_dir });
            }
        }

        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        state.cwd = cwd;
        state.entries = entries;
        if state.selected >= state.entries.len() {
            state.selected = state.entries.len().saturating_sub(1);
        }
    }

    /// Wrapper without custom border style for backwards compatibility.
    pub fn render(
        f: &mut Frame,
        area: Rect,
        state: &FileExplorerState,
        dirty: Option<(PathBuf, bool)>,
    ) {
        Self::render_with_border(f, area, state, dirty, Style::default())
    }

    /// Render explorer with a custom border style (used to show focus).
    pub fn render_with_border(
        f: &mut Frame,
        area: Rect,
        state: &FileExplorerState,
        dirty: Option<(PathBuf, bool)>,
        pane_border: Style,
    ) {
        let items: Vec<ListItem> = state
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let full_path = state.cwd.join(&e.name);
                let is_dirty_here = dirty
                    .as_ref()
                    .map(|(p, d)| *d && *p == full_path)
                    .unwrap_or(false);

                let mut label =
                    if e.is_dir { format!("📁 {}", e.name) } else { format!("📄 {}", e.name) };
                if is_dirty_here && !e.is_dir {
                    label = format!("● {}", label);
                }

                // Griser ".." si on est à la racine (normalement non affiché)
                let style = if e.name == ".." && state.cwd == state.root {
                    Style::default().fg(Color::DarkGray)
                } else if i == state.selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };

                ListItem::new(label).style(style)
            })
            .collect();

        let title = format!(
            "Explorer — {}  (root: {})",
            short_path(&state.cwd, &state.root),
            short_path(&state.root, &state.root)
        );

        let widget = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(pane_border)
                .title(title),
        );
        f.render_widget(widget, area);
    }

    pub fn move_up(state: &mut FileExplorerState) {
        if state.selected > 0 {
            state.selected -= 1;
        }
    }

    pub fn move_down(state: &mut FileExplorerState) {
        if state.selected + 1 < state.entries.len() {
            state.selected += 1;
        }
    }

    pub fn go_up(state: &mut FileExplorerState) {
        if let Some(parent) = state.cwd.parent() {
            if within_root(&state.root, parent) {
                state.cwd = parent.to_path_buf();
                Self::refresh(state);
            }
        }
    }

    /// Activate the currently selected entry.
    /// - If directory: enter it and refresh, returns None
    /// - If file: return its path (constrained to root)
    /// - If "..": go up and return None
    pub fn activate(state: &mut FileExplorerState) -> Option<PathBuf> {
        if state.entries.is_empty() {
            return None;
        }
        let entry = &state.entries[state.selected];

        if entry.name == ".." {
            Self::go_up(state);
            return None;
        }

        let path = state.cwd.join(&entry.name);
        if entry.is_dir {
            if within_root(&state.root, &path) {
                state.cwd = path;
                Self::refresh(state);
            }
            None
        } else if within_root(&state.root, &path) {
            Some(path)
        } else {
            None
        }
    }
}