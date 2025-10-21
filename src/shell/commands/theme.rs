// src/shell/commands/theme.rs
use super::Command;
use crate::shell::commands::CommandRegistry;
use crate::shell::prompt::Prompt;
use std::sync::{Arc, Mutex};

pub struct ThemeCommand {
    pub prompt: Arc<Mutex<Prompt>>,
}

impl Command for ThemeCommand {
    fn name(&self) -> &'static str {
        "theme"
    }
    fn about(&self) -> &'static str {
        "Gestion du thème (reload)."
    }
    fn usage(&self) -> &'static str {
        "theme reload"
    }

    fn execute(&self, args: &[&str], _registry: &CommandRegistry) {
        if args.first().copied() == Some("reload") {
            let mut p = self.prompt.lock().unwrap();
            p.reload();
        } else {
            println!("Usage: theme reload");
        }
    }
}
