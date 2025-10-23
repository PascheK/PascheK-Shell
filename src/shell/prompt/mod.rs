//! Module `prompt`: gestion de l'invite et des thèmes pour PascheK Shell.
//! 
//! Ce module expose deux sous-modules :
//! - `builder` : construction de la chaîne d'invite (prompt)
//! - `theme`   : définition et chargement des couleurs/thèmes
//!
//! Il réexporte également `Theme` pour un accès direct via `crate::shell::prompt::Theme`.

pub mod builder;
pub mod theme;

use crate::shell::config::ThemeConfig;
use crate::shell::prompt::builder::build_prompt;

// Réexport public pour éviter d’avoir à importer `theme::Theme` partout.
pub use self::theme::Theme;

/// Représente l'invite (prompt) courante du shell, pilotée par un `Theme`.
pub struct Prompt {
    theme: Theme,
}

impl Prompt {
    /// Crée une nouvelle instance de `Prompt`.
    ///
    /// Tente de charger la configuration depuis `config/theme.toml`; en cas d’échec,
    /// utilise `Theme::default()`.
    pub fn new() -> Self {
        let theme = ThemeConfig::load_from_file("config/theme.toml")
            .map(|cfg| Theme::from_config(&cfg))
            .unwrap_or_else(Theme::default);
        Self { theme }
    }

    /// Recharge le thème depuis `config/theme.toml`.
    pub fn reload(&mut self) {
        if let Some(cfg) = ThemeConfig::load_from_file("config/theme.toml") {
            self.theme = Theme::from_config(&cfg);
            println!("🔄 Theme reloaded successfully!");
        } else {
            println!("⚠️ Could not reload theme (missing or invalid config).");
        }
    }

    /// Construit et retourne la chaîne du prompt en fonction du thème courant.
    pub fn render(&self) -> String {
        build_prompt(&self.theme)
    }

    /// (Optionnel) Accès en lecture au thème courant.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}
