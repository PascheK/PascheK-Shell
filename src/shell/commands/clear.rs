use super::Command;

pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &'static str { "clear" }
    fn description(&self) -> &'static str { "Clears the terminal screen." }
    fn execute(&self, _args: &[&str]) {
        print!("\x1B[2J\x1B[1;1H");
    }
}