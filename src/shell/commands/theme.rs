use super::Command;
use crate::shell::prompt::Prompt;
use std::sync::{Arc, Mutex};

pub struct ThemeCommand {
    pub prompt: Arc<Mutex<Prompt>>,
}

impl Command for ThemeCommand {
    fn name(&self) -> &'static str { "theme" }
    fn description(&self) -> &'static str { "Reloads or manages theme configuration." }

    fn execute(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: theme reload");
            return;
        }

        if args[0] == "reload" {
            let mut prompt = self.prompt.lock().unwrap();
            prompt.reload();
        } else {
            println!("Unknown argument: {}", args[0]);
        }
    }
}