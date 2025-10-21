// src/shell/commands/hello.rs
use super::Command;
use crate::shell::commands::CommandRegistry;

pub struct HelloCommand;

impl Command for HelloCommand {
    fn name(&self) -> &'static str {
        "hello"
    }
    fn about(&self) -> &'static str {
        "Affiche un message de salutation."
    }
    fn usage(&self) -> &'static str {
        "hello"
    }

    fn execute(&self, _args: &[&str], _registry: &CommandRegistry) {
        println!("Hello from PascheK Shell 🦀");
    }
}
