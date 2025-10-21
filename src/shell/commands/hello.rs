use super::Command;

pub struct HelloCommand;

impl Command for HelloCommand {
    fn name(&self) -> &'static str {
        "hello"
    }

    fn description(&self) -> &'static str {
        "Displays a greeting message."
    }

    fn execute(&self, _args: &[&str]) {
        println!("Hello from PascheK Shell 🦀");
    }
}