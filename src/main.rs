//! PascheK Shell - A modern, customizable command-line interface
//! 
//! This is the main entry point for the PascheK Shell application. The shell provides
//! a feature-rich REPL environment with customizable themes, built-in commands, and
//! system command execution capabilities.

mod shell;

/// Program entry point that initializes and starts the PascheK Shell REPL.
/// 
/// The REPL (Read-Eval-Print Loop) is responsible for:
/// - Displaying a customizable prompt
/// - Reading user input
/// - Executing built-in or system commands
/// - Displaying command output
/// - Maintaining the shell state
fn main() {
    shell::repl::start_repl();
}