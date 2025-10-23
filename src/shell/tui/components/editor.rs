//! Simple text editor component used in the Workspace and Editor screens.
//!
//! Features:
//! - Open and save files within a confined root path
//! - Rope-backed buffer for efficient editing
//! - Line numbers gutter and a basic status bar
//! - Minimal modes: Normal, Insert, Command (':' prompt)
use crate::shell::tui::state::{EditorMode, EditorState};
use anyhow::{Result, bail};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Ensure that a path resides under a given root (using canonical paths).
fn within_root(root: &Path, path: &Path) -> bool {
    let r = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    p.starts_with(&r)
}

/// Stateless view providing open/save and render helpers for EditorState.
pub struct EditorView;

impl EditorView {
    /// Open a file at `path` if it lies within `root` and return a new EditorState.
    pub fn open_path<P: AsRef<Path>>(path: P, root: &Path) -> Result<EditorState> {
        let p = path.as_ref();

        if !within_root(root, p) {
            bail!("Refusé: chemin en dehors de la racine autorisée");
        }

        let content = std::fs::read_to_string(p)?;
        let mut ed = EditorState::new_empty();
        ed.path = Some(p.to_path_buf());
        ed.buffer = ropey::Rope::from_str(&content);
        ed.cursor_row = 0;
        ed.cursor_col = 0;
        ed.scroll_row = 0;
        ed.dirty = false;
        Ok(ed)
    }

    /// Save current buffer to disk. Returns an error if no associated path or write fails.
    pub fn save(ed: &mut EditorState) -> std::io::Result<()> {
        let path = ed
            .path
            .clone()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No file path"))?;
        let mut f = fs::File::create(path)?;
        let s = ed.buffer.to_string();
        f.write_all(s.as_bytes())?;
        ed.dirty = false;
        Ok(())
    }

    /// Render editor with default border style.
    pub fn render(f: &mut Frame, area: Rect, ed: &EditorState) {
        Self::render_with_border(f, area, ed, Style::default());
    }

    /// Render editor with a custom border style (used to indicate focus).
    pub fn render_with_border(f: &mut Frame, area: Rect, ed: &EditorState, pane_border: Style) {
        // ---- même contenu que ton render actuel, en ajoutant .border_style(pane_border) ----
        let mut constraints = vec![Constraint::Min(3), Constraint::Length(1)];
        if matches!(ed.mode, EditorMode::Command) {
            constraints.push(Constraint::Length(1));
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Titre avec marqueur dirty ●
        let mut title = ed
            .path
            .as_ref()
            .map(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("[No Name]")
                    .to_string()
            })
            .unwrap_or_else(|| "[No Name]".into());
        if ed.dirty {
            title = format!("● {}", title);
        }

        // Lignes visibles + gouttière numérotée
        let height = chunks[0].height.saturating_sub(2) as usize;
        let start = ed.scroll_row;
        let end = usize::min(ed.buffer.len_lines(), start + height);
        let digits = ((ed.buffer.len_lines().max(1) as f64).log10().floor() as usize) + 1;

        let mut lines: Vec<Line> = Vec::with_capacity(end - start);
        let query = ed.last_search.clone().unwrap_or_default();
        for row in start..end {
            let mut text = ed.buffer.line(row).to_string();
            if text.ends_with('\n') { text.pop(); }

            // Gouttière
            let gutter = format!("{:>width$} │ ", row + 1, width = digits);
            let mut spans: Vec<Span> = Vec::new();
            spans.push(Span::raw(gutter));

            if !query.is_empty() {
                // Surlignage naïf des occurrences (ASCII sûr; approximation pour UTF-8)
                let mut last = 0usize;
                let mut idx = 0usize;
                while let Some(found) = text[last..].find(&query) {
                    let s = last + found;
                    let e = s + query.len();
                    if s > last {
                        spans.push(Span::raw(text[last..s].to_string()));
                    }
                    // Style du match courant si index correspond
                    let is_current = ed.search_index
                        .and_then(|i| ed.search_positions.get(i))
                        .map(|(r, c)| *r == row && *c == idx)
                        .unwrap_or(false);
                    let style = if is_current { Style::default().fg(Color::Black).bg(Color::Yellow) } else { Style::default().fg(Color::Yellow) };
                    spans.push(Span::styled(text[s..e].to_string(), style));
                    last = e;
                    idx += 1;
                }
                if last < text.len() {
                    spans.push(Span::raw(text[last..].to_string()));
                }
            } else {
                spans.push(Span::raw(text));
            }

            lines.push(Line::from(spans));
        }

        let text_widget = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(pane_border)
                .title(title),
        );
        f.render_widget(text_widget, chunks[0]);

        // Barre d’état
        let path_str = ed
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| String::from("[No Name]"));
        let status = format!(
            " {}  |  row {}, col {}  {}",
            path_str,
            ed.cursor_row + 1,
            ed.cursor_col + 1,
            if ed.dirty { "[+]" } else { "" }
        );
        let status_widget = Paragraph::new(Line::from(Span::styled(
            status,
            Style::default().fg(Color::LightBlue),
        )))
        .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status_widget, chunks[1]);

        if matches!(ed.mode, EditorMode::Command) {
            let cmd = Paragraph::new(Line::from(Span::raw(format!(":{}", ed.cmdline))))
                .block(Block::default().borders(Borders::ALL).title("Command"));
            f.render_widget(cmd, chunks[2]);
        }

        // ---- Curseur (décalé par la gouttière) ----
        let cursor_x = (digits as u16) + 3 /* espace + '│' + espace */ + (ed.cursor_col as u16) + chunks[0].x + 1;
        let cursor_y = (ed.cursor_row.saturating_sub(ed.scroll_row) as u16) + chunks[0].y + 1;
        let position: Position = Position {
            x: cursor_x,
            y: cursor_y,
        };
        f.set_cursor_position(position);
    }

    // Mouvements (navigation)
    pub fn move_left(ed: &mut EditorState) {
        if ed.cursor_col > 0 {
            ed.cursor_col -= 1;
        }
    }
    pub fn move_right(ed: &mut EditorState) {
        let line = ed.buffer.line(ed.cursor_row);
        let len = line.chars().count();
        if ed.cursor_col < len {
            ed.cursor_col += 1;
        }
    }
    pub fn move_up(ed: &mut EditorState) {
        if ed.cursor_row > 0 {
            ed.cursor_row -= 1;
        }
        Self::clamp_col(ed);
        if ed.cursor_row < ed.scroll_row {
            ed.scroll_row = ed.cursor_row;
        }
    }
    pub fn move_down(ed: &mut EditorState) {
        if ed.cursor_row + 1 < ed.buffer.len_lines() {
            ed.cursor_row += 1;
        }
        Self::clamp_col(ed);
        let visible_h = 20; // approx; on pourrait passer la hauteur
        if ed.cursor_row >= ed.scroll_row + visible_h {
            ed.scroll_row = ed.cursor_row.saturating_sub(visible_h - 1);
        }
    }
    fn clamp_col(ed: &mut EditorState) {
        let line_len = ed.buffer.line(ed.cursor_row).chars().count();
        if ed.cursor_col > line_len {
            ed.cursor_col = line_len;
        }
    }

    // Edition (INSERT)
    pub fn insert_char(ed: &mut EditorState, c: char) {
        ed.push_undo();
        let char_idx = Self::cursor_to_char_idx(ed);
        ed.buffer.insert_char(char_idx, c);
        ed.cursor_col += 1;
        ed.dirty = true;
        ed.search_positions.clear();
        ed.search_index = None;
    }
    pub fn backspace(ed: &mut EditorState) {
        ed.push_undo();
        let char_idx = Self::cursor_to_char_idx(ed);
        if char_idx > 0 {
            ed.buffer.remove(char_idx - 1..char_idx);
            if ed.cursor_col > 0 {
                ed.cursor_col -= 1;
            } else if ed.cursor_row > 0 {
                // si on supprime le \n précédent, recaler
                ed.cursor_row -= 1;
                ed.cursor_col = ed.buffer.line(ed.cursor_row).chars().count();
            }
            ed.dirty = true;
            ed.search_positions.clear();
            ed.search_index = None;
        }
    }
    pub fn insert_newline(ed: &mut EditorState) {
        ed.push_undo();
        let char_idx = Self::cursor_to_char_idx(ed);
        ed.buffer.insert(char_idx, "\n");
        ed.cursor_row += 1;
        ed.cursor_col = 0;
        ed.dirty = true;
        ed.search_positions.clear();
        ed.search_index = None;
    }

    fn cursor_to_char_idx(ed: &EditorState) -> usize {
        let line_start = ed.buffer.line_to_char(ed.cursor_row);
        line_start + ed.cursor_col
    }

    /// Undo last change if any
    pub fn undo(ed: &mut EditorState) {
        if let Some(prev) = ed.undo_stack.pop() {
            // push current to redo
            let current = super::super::state::EditorSnapshot {
                buffer: ed.buffer.clone(),
                cursor_row: ed.cursor_row,
                cursor_col: ed.cursor_col,
                scroll_row: ed.scroll_row,
                dirty: ed.dirty,
            };
            ed.redo_stack.push(current);
            // restore prev
            ed.buffer = prev.buffer;
            ed.cursor_row = prev.cursor_row;
            ed.cursor_col = prev.cursor_col;
            ed.scroll_row = prev.scroll_row;
            ed.dirty = prev.dirty;
        }
    }

    /// Redo next change if any
    pub fn redo(ed: &mut EditorState) {
        if let Some(next) = ed.redo_stack.pop() {
            // push current to undo
            ed.push_undo();
            // restore next
            ed.buffer = next.buffer;
            ed.cursor_row = next.cursor_row;
            ed.cursor_col = next.cursor_col;
            ed.scroll_row = next.scroll_row;
            ed.dirty = next.dirty;
        }
    }

    /// Recompute all search positions for last_search across the buffer
    pub fn recompute_search_positions(ed: &mut EditorState) {
        ed.search_positions.clear();
        ed.search_index = None;
        let Some(q) = ed.last_search.as_ref() else { return; };
        if q.is_empty() { return; }
        for row in 0..ed.buffer.len_lines() {
            let mut text = ed.buffer.line(row).to_string();
            if text.ends_with('\n') { text.pop(); }
            let mut last = 0usize;
            let mut idx = 0usize;
            while let Some(found) = text[last..].find(q) {
                let s = last + found;
                ed.search_positions.push((row, idx));
                last = s + q.len();
                idx += 1;
            }
        }
    }

    /// Jump to next search occurrence (wrap)
    pub fn search_next(ed: &mut EditorState) {
        if ed.search_positions.is_empty() {
            Self::recompute_search_positions(ed);
        }
        if ed.search_positions.is_empty() { return; }
        // Find current position index based on cursor
        let current = ed.search_index.unwrap_or_else(|| {
            // choose first occurrence after cursor
            let mut idx0 = 0usize;
            for (i, (row, _)) in ed.search_positions.iter().enumerate() {
                if *row > ed.cursor_row || (*row == ed.cursor_row && 0 >= ed.cursor_col) { idx0 = i; break; }
            }
            idx0
        });
        let next = (current + 1) % ed.search_positions.len();
        ed.search_index = Some(next);
        Self::jump_to_search(ed);
    }

    /// Jump to previous search occurrence (wrap)
    pub fn search_prev(ed: &mut EditorState) {
        if ed.search_positions.is_empty() {
            Self::recompute_search_positions(ed);
        }
        if ed.search_positions.is_empty() { return; }
        let current = ed.search_index.unwrap_or(0);
        let prev = if current == 0 { ed.search_positions.len() - 1 } else { current - 1 };
        ed.search_index = Some(prev);
        Self::jump_to_search(ed);
    }

    fn jump_to_search(ed: &mut EditorState) {
        if let Some(i) = ed.search_index {
            if let Some((row, _idx_in_row)) = ed.search_positions.get(i).copied() {
                ed.cursor_row = row;
                ed.cursor_col = 0;
                if ed.cursor_row < ed.scroll_row { ed.scroll_row = ed.cursor_row; }
            }
        }
    }
}
