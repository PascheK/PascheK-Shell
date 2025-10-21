use std::{fs, path::Path};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    pub shell: ColorSection,
    pub path: ColorSection,
    pub time: ColorSection,
    pub symbol: ColorSection,
}

#[derive(Debug, Deserialize)]
pub struct ColorSection {
    pub color: String,
}

impl ThemeConfig {
    pub fn load_from_file(path: &str) -> Option<Self> {
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).ok()?;
            toml::from_str::<ThemeConfig>(&content).ok()
        } else {
            None
        }
    }
}