//! Core shell functionality for the PascheK Shell
//!
//! This module serves as the primary namespace for all shell-related functionality:
//! 
//! - [`repl`]: The Read-Eval-Print Loop that drives the shell's interaction cycle
//! - [`executor`]: Command execution engine for both built-in and system commands
//! - [`commands`]: Registry and implementations of built-in shell commands
//! - [`prompt`]: Customizable prompt rendering and theming system
//! - [`config`]: Shell configuration management and persistence
//!
//! The architecture follows a clear separation of concerns:
//! 1. The REPL orchestrates the interaction loop
//! 2. Commands are dispatched through the executor
//! 3. Built-in commands are registered in the command registry
//! 4. The prompt system handles visual presentation
//! 5. Configuration manages persistent settings

pub mod repl;
pub mod executor;
pub mod commands;
pub mod prompt;
pub mod config;
pub mod tui;