//! Built-in command system for PascheK Shell
//!
//! This module provides the infrastructure for registering and executing
//! built-in shell commands. It includes:
//!
//! - A trait-based command system for consistent command interfaces
//! - A central registry for managing available commands
//! - Built-in commands for core shell functionality
//!
//! Available commands:
//! - [`cd`]: Change current directory
//! - [`clear`]: Clear terminal screen
//! - [`hello`]: Display greeting message
//! - [`help`]: Show available commands
//! - [`theme`]: Customize prompt appearance

pub mod cd;
pub mod clear;
pub mod hello;
pub mod help;
pub mod theme;

/// Trait defining the interface for all shell commands
///
/// This trait ensures all commands provide:
/// - A static name for command invocation
/// - A description for help text
/// - An execution method that handles the command's logic
///
/// # Implementation Guidelines
/// - `name`: Should be lowercase, no spaces
/// - `description`: Should be a brief, single-line explanation
/// - `execute`: Should handle all command-specific logic and output
///
/// # Examples
/// ```no_run
/// struct MyCommand;
/// impl Command for MyCommand {
///     fn name(&self) -> &'static str { "mycommand" }
///     fn description(&self) -> &'static str { "Does something useful" }
///     fn execute(&self, args: &[&str]) {
///         println!("Executed with args: {:?}", args);
///     }
/// }
/// ```
pub trait Command {
    /// Returns the command's name used for invocation
    fn name(&self) -> &'static str;
    /// Returns a brief description of the command's purpose
    fn description(&self) -> &'static str;
    /// Executes the command with the given arguments
    fn execute(&self, args: &[&str]);
}

use std::{collections::HashMap, sync::{Arc, Mutex}};
use crate::shell::prompt::Prompt;

use self::{
    cd::CdCommand, clear::ClearCommand, hello::HelloCommand, help::HelpCommand, theme::ThemeCommand,
};

/// Central registry for managing and executing built-in shell commands
///
/// The CommandRegistry maintains a collection of commands that implement the
/// [`Command`] trait, allowing for:
/// - Dynamic registration of new commands
/// - Command lookup by name
/// - Unified execution interface
///
/// Commands are stored as trait objects (`Box<dyn Command>`) to allow for
/// runtime polymorphism and extensibility.
pub struct CommandRegistry {
    /// Map of command names to their implementations
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    /// Creates a new registry initialized with basic commands
    /// 
    /// # Returns
    /// A new CommandRegistry with core commands registered:
    /// - cd (change directory)
    /// - clear (clear screen)
    /// - hello (greeting)
    /// - help (command list)
    #[allow(dead_code)]  // Used as alternative constructor
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        // Register core commands
        registry.register(CdCommand);
        registry.register(ClearCommand);
        registry.register(HelloCommand);
        registry.register(HelpCommand);
        registry
    }

    /// Registers a new command in the registry
    ///
    /// # Arguments
    /// * `cmd` - Any type that implements the Command trait
    ///
    /// # Type Parameters
    /// * `C` - The command type, must implement Command and have static lifetime
    pub fn register<C: Command + 'static>(&mut self, cmd: C) {
        self.commands.insert(cmd.name().to_string(), Box::new(cmd));
    }

    /// Attempts to execute a command by name
    ///
    /// # Arguments
    /// * `cmd` - The name of the command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// * `true` if the command was found and executed
    /// * `false` if the command wasn't found
    pub fn execute(&self, cmd: &str, args: &[&str]) -> bool {
        if let Some(command) = self.commands.get(cmd) {
            command.execute(args);
            true
        } else {
            false
        }
    }

    /// Creates a new registry with prompt access and theme support
    ///
    /// # Arguments
    /// * `prompt` - Thread-safe reference to the shell's prompt
    ///
    /// # Returns
    /// A new CommandRegistry with all commands including theme customization
    pub fn new_with_prompt(prompt: Arc<Mutex<Prompt>>) -> Self {
        let mut registry = Self {
            commands: std::collections::HashMap::new(),
        };

        registry.register(HelloCommand);
        registry.register(ClearCommand);
        registry.register(CdCommand);
        registry.register(HelpCommand);
        registry.register(ThemeCommand { prompt });

        registry
    }
}
