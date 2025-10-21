//! Read-Eval-Print Loop (REPL) implementation for PascheK Shell
//!
//! This module implements the main interaction loop of the shell, which:
//! 1. Renders the customizable prompt
//! 2. Reads user input
//! 3. Processes the input (trims whitespace, handles empty lines)
//! 4. Executes commands via the command executor
//! 5. Handles special cases (exit command, errors)
//!
//! The REPL uses thread-safe constructs (`Arc<Mutex<>>`) to manage shared state
//! between the prompt and command registry, allowing commands to modify the prompt
//! appearance while maintaining thread safety.

use std::{io::{self, Write}, sync::{Arc, Mutex}};
use crate::shell::{
    commands::CommandRegistry,
    executor::execute_command,
    prompt::Prompt,
};

/// Starts the main REPL loop of the PascheK Shell
///
/// This function:
/// 1. Initializes the prompt with thread-safe sharing (`Arc<Mutex<>>`)
/// 2. Creates the command registry with a reference to the prompt
/// 3. Displays the welcome message
/// 4. Enters the main loop which:
///    - Renders and displays the prompt
///    - Reads user input
///    - Handles special commands (exit)
///    - Delegates command execution
///
/// # Error Handling
/// - Input reading errors are caught and reported
/// - Empty lines are skipped
/// - The loop continues until an explicit 'exit' command
///
/// # Thread Safety
/// Uses `Arc<Mutex<Prompt>>` to safely share the prompt between
/// the REPL and commands that may need to modify it.
pub fn start_repl() {
    // Create thread-safe prompt instance that can be modified by commands
    let prompt = Arc::new(Mutex::new(Prompt::new()));
    let registry = CommandRegistry::new_with_prompt(prompt.clone());

    // Display welcome message and initial instructions
    println!("🦀 Welcome to PascheK Shell");
    println!("Type 'help' for a list of commands.\n");

    loop {
        // Acquire lock and render prompt - lock is automatically released at end of line
        let rendered = prompt.lock().unwrap().render();
        print!("{}", rendered);
        // Ensure prompt is displayed immediately without buffering
        io::stdout().flush().unwrap();

        // Read user input into a fresh String
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("❌ Error reading input");
            continue;  // Skip this iteration on read error
        }

        // Remove leading/trailing whitespace
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;  // Skip empty lines
        }

        // Handle special 'exit' command before regular command processing
        if trimmed == "exit" {
            println!("👋 Goodbye!");
            break;
        }

        // Delegate all other commands to the executor
        execute_command(trimmed, &registry);
    }
}