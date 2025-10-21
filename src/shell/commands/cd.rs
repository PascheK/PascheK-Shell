// src/shell/commands/cd.rs
use super::Command;
use crate::shell::commands::CommandRegistry;
use std::env;

pub struct CdCommand;

impl Command for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }
    fn about(&self) -> &'static str {
        "Change le répertoire courant."
    }
    fn usage(&self) -> &'static str {
        "cd <path>"
    }

    fn execute(&self, args: &[&str], _registry: &CommandRegistry) {
        if args.is_empty() {
            eprintln!("Usage: cd <path>");
            return;
        }
        if let Err(e) = env::set_current_dir(args[0]) {
            eprintln!("❌ Impossible de se déplacer: {e}");
        }
    }
}
