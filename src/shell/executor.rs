//! Command execution engine for PascheK Shell
//!
//! This module handles the execution of both built-in and system commands.
//! The execution strategy follows this order:
//! 1. Try to execute as a built-in command via the CommandRegistry
//! 2. If not found, attempt to execute as a system command via PATH
//!
//! Output handling:
//! - stdout is printed to the terminal normally
//! - stderr is printed to the terminal in red
//! - Command not found errors are displayed with a ❌ prefix

use std::process::Command;
use crate::shell::commands::CommandRegistry;

/// Executes a shell command with the given input string and command registry
///
/// # Arguments
/// * `input` - The raw command string from user input
/// * `registry` - The registry containing all built-in commands
///
/// # Execution Process
/// 1. Splits input into command and arguments
/// 2. Attempts to execute as built-in command first
/// 3. Falls back to system command execution if not found
///
/// # Output Handling
/// - Built-in commands handle their own output
/// - System commands:
///   - stdout is printed directly
///   - stderr is printed to stderr
///   - Command not found errors are displayed with ❌
///
/// # Examples
/// ```no_run
/// let registry = CommandRegistry::new();
/// execute_command("ls -l", &registry);  // System command
/// execute_command("help", &registry);   // Built-in command
/// ```
pub fn execute_command(input: &str, registry: &CommandRegistry) {
    // Split input into command and arguments
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    // Extract command name and arguments
    let cmd = parts[0];
    let args = &parts[1..];

    // Try to execute as a built-in command first
    // If successful, return early as the command handles its own output
    if registry.execute(cmd, args) {
        return;
    }

    // Fall back to system command execution
    match Command::new(cmd).args(args).output() {
        Ok(output) => {
            // Print stdout if present (using lossy UTF-8 conversion for safety)
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            // Print stderr if present
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => {
            // Command not found in PATH or execution error
            eprintln!("❌ Command not found: {}", cmd);
        }
    }
}