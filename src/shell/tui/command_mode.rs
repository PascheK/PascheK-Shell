//! TUI command handler for ":"-prefixed commands in the Shell screen.
//!
//! Supported commands:
//! - :q, :quit        → exit the TUI
//! - :l, :logs        → toggle the logs side panel (sticky)
//! - :h, :help        → toggle the ephemeral help overlay
//! - :clear           → clear logs
//!
// src/shell/tui/command_mode.rs
use crate::shell::tui::state::{TuiState, Overlay};
use crate::shell::tui::components::logs::LogPanel;

/// Small helper object that mutates TuiState and LogPanel based on a parsed command.
pub struct TuiCommandHandler<'a> {
    pub state: &'a mut TuiState,
    pub logs: &'a mut LogPanel,
}

impl<'a> TuiCommandHandler<'a> {
    /// Execute a ":"-prefixed TUI command.
    pub fn execute(&mut self, input: &str) {
        let cmd = input.trim_start_matches(':').trim();
        match cmd {
            "q" | "quit" => {
                self.logs.add("👋 Quit requested.");
                self.state.running = false;
            }
            "l" | "logs" => {
                self.state.show_logs = !self.state.show_logs; // ✅ sticky toggle
                self.logs.add(if self.state.show_logs { "🪵 Logs opened." } else { "🪵 Logs closed." });
            }
            "h" | "help" => {
                // ✅ overlay éphémère : s’affiche, se fermera à la 1re touche
                self.state.overlay = match self.state.overlay {
                    Overlay::None => Overlay::Help,
                    _ => Overlay::None, // Close Help or any input overlay
                };
                self.state.overlay_input = None;
                self.logs.add("🛈 Help toggled.");
            }
            "clear" => {
                self.logs.clear();
                self.logs.add("🧹 Logs cleared.");
            }
            _ => self.logs.add(format!("❓ Unknown TUI command: :{cmd}")),
        }
    }
}