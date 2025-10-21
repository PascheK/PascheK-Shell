//! Prompt builder for PascheK Shell
//!
//! This module is responsible for constructing the shell's prompt string,
//! which includes:
//! - Shell name with customizable color
//! - Current directory name
//! - Current time
//! - Decorative symbol
//!
//! The prompt is built using the following segments:
//! ```text
//! [Shell Name]> • [Current Dir] [Time]
//! ```
//!
//! Each segment's color is controlled by the active theme.

use chrono::Local;
use std::env;
use crate::shell::prompt::theme::Theme;
use owo_colors::OwoColorize;

/// Builds a formatted prompt string for display in the terminal
///
/// # Arguments
/// * `theme` - Reference to the current Theme for color information
///
/// # Format
/// The prompt is constructed with the following segments:
/// 1. Shell name ("PascheK>") in shell_color
/// 2. Bullet point ("•") in symbol_color
/// 3. Current directory name in path_color
/// 4. Current time (HH:MM:SS) in time_color
///
/// # Example Output
/// ```text
/// PascheK> • src 22:45:13
/// ```
///
/// # Returns
/// A String containing the fully formatted prompt with ANSI color codes
pub fn build_prompt(theme: &Theme) -> String {
    // Get the current working directory name
    // Falls back to "~" if the directory name can't be determined
    let cwd = env::current_dir()
        .ok()  // Handle potential errors from current_dir()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "~".into());

    // Format current local time as HH:MM:SS
    let time = Local::now().format("%H:%M:%S").to_string();

    // Build the prompt with themed color segments:
    // 1. Shell name with theme's shell color
    // 2. Bullet separator with theme's symbol color
    // 3. Directory name with theme's path color
    // 4. Time with theme's time color
    // Note: Extra space at the end ensures proper cursor positioning
    format!(
        "{} {} {} {} ",
        theme.apply_shell("PascheK>"),
        theme.apply_symbol("•"),
        theme.apply_path(&cwd),
        theme.apply_time(&time),
    )
}


impl Theme {
    pub fn apply_shell(&self, text: &str) -> String {
        text.color(self.shell_color).to_string()
    }

    pub fn apply_symbol(&self, text: &str) -> String {
        text.color(self.symbol_color).to_string()
    }

    pub fn apply_path(&self, text: &str) -> String {
        text.color(self.path_color).to_string()
    }

    pub fn apply_time(&self, text: &str) -> String {
        text.color(self.time_color).to_string()
    }
}