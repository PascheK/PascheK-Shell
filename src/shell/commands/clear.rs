// src/shell/commands/clear.rs
use super::Command;
use crate::shell::commands::CommandRegistry;

pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &'static str {
        "clear"
    }
    fn about(&self) -> &'static str {
        "Efface l’écran du terminal."
    }
    fn usage(&self) -> &'static str {
        "clear"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["cls"]
    }

    fn execute(&self, _args: &[&str], _registry: &CommandRegistry) {
        print!("\x1B[2J\x1B[1;1H");
    }
}
