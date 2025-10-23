//! Shared state structures for the PascheK Shell TUI.
//!
//! Contains enums and structs that describe the current screen, overlays,
//! panel focus, and the state for the File Explorer and Editor.
//! The goal is to keep UI rendering functions stateless and pure, while
//! this module represents the mutable state manipulated by input handlers.

use std::path::PathBuf;
use ropey::Rope;

/// Current main screen displayed by the TUI.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Shell,
    Explorer,
    Editor,
    Workspace, // si tu l'utilises pour le split Explorer | Editor
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Home // ou Screen::Workspace si tu veux démarrer en IDE
    }
}

/// Overlays displayed above the current screen.
/// Help is ephemeral (closes on next key). Input carries a small stateful prompt.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    Help,
    Input,
}

impl Default for Overlay {
    fn default() -> Self { Overlay::None }
}

/// Which pane currently has keyboard focus (used in Workspace split view)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Explorer,
    Editor,
}

impl Default for Focus {
    fn default() -> Self { Focus::Editor }
}

/// File explorer state (root, cwd, entries, selection, hidden toggle)
#[derive(Default)]
pub struct FileExplorerState {
    pub cwd: PathBuf,
    pub root: PathBuf,
    pub entries: Vec<DirEntryView>,
    pub selected: usize,
    pub show_hidden: bool,
}

/// A single displayed entry in the explorer list
pub struct DirEntryView {
    pub name: String,
    pub is_dir: bool,
}

/// Editor modes (simple Vim-like)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Command,
}

/// Text editor state backed by ropey for efficient edits
pub struct EditorState {
    pub path: Option<PathBuf>,
    pub buffer: Rope,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub mode: EditorMode,
    pub cmdline: String,
    pub dirty: bool,
    /// Last search query entered (for Ctrl+F prefill)
    pub last_search: Option<String>,
    pub search_positions: Vec<(usize, usize)>, // (row, col in chars)
    pub search_index: Option<usize>,
    /// Undo/redo stacks (bounded)
    pub undo_stack: Vec<EditorSnapshot>,
    pub redo_stack: Vec<EditorSnapshot>,
}

impl EditorState {
    /// Create an empty editor buffer with default positions and mode
    pub fn new_empty() -> Self {
        Self {
            path: None,
            buffer: Rope::from_str(""),
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            mode: EditorMode::Normal,
            cmdline: String::new(),
            dirty: false,
            last_search: None,
            search_positions: Vec::new(),
            search_index: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

/// Global TUI state including the active screen and child states
pub struct TuiState {
    pub running: bool,
    pub screen: Screen,
    pub focus: Focus,
    pub show_logs: bool,
    pub overlay: Overlay,
    // Input overlay is handled via this optional state when overlay == Input
    pub overlay_input: Option<InputOverlay>,
    pub explorer: FileExplorerState,
    pub editor: Option<EditorState>,
    /// Multiple editor tabs; current determines which one is shown.
    pub tabs: EditorTabs,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            running: true,
            screen: Screen::Home,   
            focus: Focus::Editor,
            show_logs: false,
            overlay: Overlay::None,
            overlay_input: None,
            explorer: FileExplorerState::default(),
            editor: None,
            tabs: EditorTabs::default(),
        }
    }
}

impl TuiState {
    /// Convenience constructor equal to Default
    pub fn new() -> Self { Self::default() }
}

pub struct EditorTab {
    pub state: EditorState,
}

pub struct EditorTabs {
    pub tabs: Vec<EditorTab>,
    pub current: usize,
}

/// Snapshot for undo/redo
pub struct EditorSnapshot {
    pub buffer: Rope,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub dirty: bool,
}

impl EditorState {
    /// Push current state to undo stack, clear redo; keep at most 50 entries
    pub fn push_undo(&mut self) {
        let snap = EditorSnapshot {
            buffer: self.buffer.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
            scroll_row: self.scroll_row,
            dirty: self.dirty,
        };
        self.undo_stack.push(snap);
        if self.undo_stack.len() > 50 { let overflow = self.undo_stack.len() - 50; self.undo_stack.drain(0..overflow); }
        self.redo_stack.clear();
    }
}

/// Kind of input requested by an input overlay
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    NewEntry,       // create file or folder (folder if name ends with '/')
    RenameEntry,    // rename selected entry
    DeleteConfirm,  // confirm deletion of selected entry (type 'y' to confirm)
    SearchText,     // search text within current editor buffer
    GotoLine,       // go to a specific line number
}

/// State for a minimal input overlay (prompt at bottom or centered popup)
pub struct InputOverlay {
    pub kind: InputKind,
    pub buffer: String,
}

impl Default for EditorTabs {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            current: 0,
        }
    }
}

impl EditorTabs {
    /// Return true if no tabs are open.
    pub fn is_empty(&self) -> bool { self.tabs.is_empty() }

    /// Current editor state (immutable), if any
    pub fn current(&self) -> Option<&EditorState> { self.tabs.get(self.current).map(|t| &t.state) }

    /// Current editor state (mutable), if any
    pub fn current_mut(&mut self) -> Option<&mut EditorState> { self.tabs.get_mut(self.current).map(|t| &mut t.state) }

    /// Focus the next tab (wrap-around)
    pub fn next(&mut self) {
        if !self.tabs.is_empty() { self.current = (self.current + 1) % self.tabs.len(); }
    }

    /// Focus the previous tab (wrap-around)
    pub fn prev(&mut self) {
        if !self.tabs.is_empty() { self.current = (self.current + self.tabs.len() - 1) % self.tabs.len(); }
    }

    /// Close the current tab and adjust the index. Does nothing if no tabs.
    pub fn close_current(&mut self) {
        if self.tabs.is_empty() { return; }
        self.tabs.remove(self.current);
        if self.current >= self.tabs.len() { self.current = self.tabs.len().saturating_sub(1); }
    }

    /// Focus the tab at a given index if it exists.
    pub fn focus(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.current = idx;
        }
    }

    /// For now: keep a single tab. Replace existing buffer with new state.
    pub fn open_or_focus(&mut self, ed: EditorState) {
        self.tabs.clear();
        self.tabs.push(EditorTab { state: ed });
        self.current = 0;
    }
}