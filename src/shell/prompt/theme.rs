//! Theme management for PascheK Shell prompt customization
//!
//! This module handles the visual theming of the shell prompt, including:
//! - Color definitions for different prompt segments
//! - Theme loading from TOML configuration
//! - Color parsing from string names
//!
//! # Supported Colors
//! All ANSI colors are supported through the `owo-colors` crate:
//! - Standard colors: black, red, green, yellow, blue, magenta, cyan, white
//! - Bright variants: brightred, brightgreen, etc.
//!
//! # Configuration
//! Themes are configured via TOML files with sections for each prompt segment:
//! ```toml
//! [shell]
//! color = "brightgreen"
//! [path]
//! color = "brightblue"
//! [time]
//! color = "brightyellow"
//! [symbol]
//! color = "brightmagenta"
//! ```

use owo_colors::AnsiColors;
use crate::shell::config::ThemeConfig;

/// Theme configuration for the shell prompt
///
/// Defines colors for each segment of the prompt:
/// - Shell name
/// - Current path
/// - Timestamp
/// - Prompt symbol
///
/// Colors are stored as ANSI color values for efficient rendering.
/// The theme can be created from default values or loaded from a
/// configuration file.
#[derive(Clone)]
pub struct Theme {
    /// Color for the shell name segment
    pub shell_color: AnsiColors,
    /// Color for the current path segment
    pub path_color: AnsiColors,
    /// Color for the timestamp segment
    pub time_color: AnsiColors,
    /// Color for the prompt symbol
    pub symbol_color: AnsiColors,
}

impl Theme {
    /// Creates a new Theme with default color settings
    ///
    /// The default theme uses bright variants of standard colors:
    /// - Shell: Bright Green
    /// - Path: Bright Blue
    /// - Time: Bright Yellow
    /// - Symbol: Bright Magenta
    ///
    /// # Returns
    /// A new Theme instance with default colors
    pub fn default() -> Self {
        Self {
            shell_color: AnsiColors::BrightGreen,
            path_color: AnsiColors::BrightBlue,
            time_color: AnsiColors::BrightYellow,
            symbol_color: AnsiColors::BrightMagenta,
        }
    }

    /// Creates a new Theme from a TOML configuration
    ///
    /// # Arguments
    /// * `cfg` - Reference to a ThemeConfig containing color settings
    ///
    /// # Color Parsing
    /// Colors are parsed from strings in the configuration file.
    /// If a color name is invalid, it falls back to a default color.
    ///
    /// # Returns
    /// A new Theme instance with colors from the configuration
    pub fn from_config(cfg: &ThemeConfig) -> Self {
        Self {
            shell_color: Self::parse_color(&cfg.shell.color),
            path_color: Self::parse_color(&cfg.path.color),
            time_color: Self::parse_color(&cfg.time.color),
            symbol_color: Self::parse_color(&cfg.symbol.color),
        }
    }

    fn parse_color(name: &str) -> AnsiColors {
        match name.to_lowercase().as_str() {
            "black" => AnsiColors::Black,
            "red" => AnsiColors::Red,
            "green" => AnsiColors::Green,
            "yellow" => AnsiColors::Yellow,
            "blue" => AnsiColors::Blue,
            "magenta" => AnsiColors::Magenta,
            "cyan" => AnsiColors::Cyan,
            "white" => AnsiColors::White,
            "brightgreen" => AnsiColors::BrightGreen,
            "brightblue" => AnsiColors::BrightBlue,
            "brightyellow" => AnsiColors::BrightYellow,
            "brightmagenta" => AnsiColors::BrightMagenta,
            "brightcyan" => AnsiColors::BrightCyan,
            _ => AnsiColors::White,
        }
    }
}