//! Change Directory (cd) command implementation
//!
//! Provides the ability to change the current working directory of the shell.
//! Supports both absolute and relative paths, and displays appropriate error
//! messages if the directory change fails.

use super::Command;
use std::env;

/// Command implementation for changing the current working directory
///
/// # Usage
/// ```shell
/// cd <path>
/// ```
///
/// # Arguments
/// * `<path>` - The target directory path (absolute or relative)
///
/// # Examples
/// ```shell
/// cd /home/user          # Absolute path
/// cd ..                  # Parent directory
/// cd ./projects/rust     # Relative path
/// ```
///
/// # Error Handling
/// - Displays usage message if no path is provided
/// - Shows error message with specific failure reason if directory change fails
pub struct CdCommand;

impl Command for CdCommand {
    /// Returns "cd" as the command name
    fn name(&self) -> &'static str {
        "cd"
    }

    /// Provides a brief description for the help command
    fn description(&self) -> &'static str {
        "Changes the current working directory."
    }

    /// Executes the cd command with the provided arguments
    ///
    /// # Arguments
    /// * `args` - Array of command arguments, expects exactly one argument
    ///   for the target directory path
    fn execute(&self, args: &[&str]) {
        // Check for required path argument
        if args.is_empty() {
            eprintln!("Usage: cd <path>");
            return;
        }

        // Extract the target path (first argument)
        let new_path = args[0];
        // Attempt to change directory using standard env function
        if let Err(e) = env::set_current_dir(new_path) {
            // Provide detailed error message on failure
            eprintln!("❌ Failed to change directory: {}", e);
        }
    }
}