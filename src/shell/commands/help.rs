use super::{Command, CommandRegistry};

pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &'static str { "help" }
    fn description(&self) -> &'static str { "Displays this help message." }
    fn execute(&self, _args: &[&str]) {
        println!("Type a command or 'exit' to quit.\n");
        println!("For now, use 'hello', 'clear', or any system command.");
    }
}