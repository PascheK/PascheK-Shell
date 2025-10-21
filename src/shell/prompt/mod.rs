pub mod builder;
pub mod theme;

use crate::shell::config::ThemeConfig;
use crate::shell::prompt::builder::build_prompt;
use crate::shell::prompt::theme::Theme;

pub struct Prompt {
    theme: Theme,
}

impl Prompt {
    pub fn new() -> Self {
        let theme = ThemeConfig::load_from_file("config/theme.toml")
            .map(|cfg| Theme::from_config(&cfg))
            .unwrap_or_else(Theme::default);
        Self { theme }
    }

    pub fn reload(&mut self) {
        if let Some(cfg) = ThemeConfig::load_from_file("config/theme.toml") {
            self.theme = Theme::from_config(&cfg);
            println!("🔄 Theme reloaded successfully!");
        } else {
            println!("⚠️ Could not reload theme (missing or invalid config).");
        }
    }

    pub fn render(&self) -> String {
        build_prompt(&self.theme)
    }
}
