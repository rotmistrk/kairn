//! Theme loading and storage.
//!
//! Themes are rusticle scripts that set variables in the `theme` context.
//! Embedded themes are compiled into the binary. User themes are loaded
//! from `~/.config/kairn/themes/`.

use std::collections::HashMap;
use std::path::PathBuf;

/// Resolved theme colors and properties.
pub struct ThemeValues {
    values: HashMap<String, String>,
    embedded: HashMap<&'static str, &'static str>,
    user_dir: Option<PathBuf>,
}

impl ThemeValues {
    /// Create with embedded themes and optional user directory.
    pub fn new() -> Self {
        let user_dir = std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config/kairn/themes"));
        Self {
            values: HashMap::new(),
            embedded: embedded_themes(),
            user_dir,
        }
    }

    /// Get a theme property value.
    pub fn get(&self, property: &str) -> Option<&str> {
        self.values.get(property).map(|s| s.as_str())
    }

    /// Set a theme property.
    pub fn set(&mut self, property: &str, value: &str) {
        self.values.insert(property.to_string(), value.to_string());
    }

    /// Find a theme script by name. Checks user dir first, then embedded.
    pub fn find_theme_script(&self, name: &str) -> Option<String> {
        // Check user themes directory
        if let Some(ref dir) = self.user_dir {
            let path = dir.join(format!("{name}.tcl"));
            if let Ok(content) = std::fs::read_to_string(&path) {
                return Some(content);
            }
        }
        // Check embedded themes
        self.embedded.get(name).map(|s| (*s).to_string())
    }

    /// List all available theme names.
    pub fn available_themes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.embedded.keys().map(|k| (*k).to_string()).collect();
        // Add user themes
        if let Some(ref dir) = self.user_dir {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "tcl") {
                        if let Some(stem) = path.file_stem() {
                            let name = stem.to_string_lossy().to_string();
                            if !names.contains(&name) {
                                names.push(name);
                            }
                        }
                    }
                }
            }
        }
        names.sort();
        names
    }

    /// Apply theme values from context variables after eval.
    pub fn apply_from_context(&mut self, interp: &rusticle::interpreter::Interpreter) {
        for prop in THEME_PROPERTIES {
            let var = format!("theme::{prop}");
            if let Some(val) = interp.get_var(&var) {
                self.values
                    .insert(prop.to_string(), val.as_str().to_string());
            }
        }
    }
}

impl Default for ThemeValues {
    fn default() -> Self {
        Self::new()
    }
}

/// All theme property names.
const THEME_PROPERTIES: &[&str] = &[
    "bg",
    "fg",
    "cursor",
    "selection",
    "comment",
    "keyword",
    "string",
    "number",
    "type",
    "function",
    "error",
    "warning",
    "gutter-bg",
    "gutter-fg",
    "status-bg",
    "status-fg",
    "tree-bg",
    "tree-fg",
];

fn embedded_themes() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("gruvbox-dark", GRUVBOX_DARK);
    m.insert("gruvbox-light", GRUVBOX_LIGHT);
    m.insert("catppuccin", CATPPUCCIN);
    m.insert("solarized-dark", SOLARIZED_DARK);
    m.insert("solarized-light", SOLARIZED_LIGHT);
    m.insert("one-dark", ONE_DARK);
    m
}

const GRUVBOX_DARK: &str = r##"context theme {
    set bg         "#282828"
    set fg         "#ebdbb2"
    set cursor     "#fabd2f"
    set selection  "#504945"
    set comment    "#928374"
    set keyword    "#fb4934"
    set string     "#b8bb26"
    set number     "#d3869b"
    set type       "#83a598"
    set function   "#fabd2f"
    set error      "#fb4934"
    set warning    "#fe8019"
    set gutter-bg  "#282828"
    set gutter-fg  "#665c54"
    set status-bg  "#3c3836"
    set status-fg  "#a89984"
    set tree-bg    "#282828"
    set tree-fg    "#ebdbb2"
}"##;

const GRUVBOX_LIGHT: &str = r##"context theme {
    set bg         "#fbf1c7"
    set fg         "#3c3836"
    set cursor     "#d79921"
    set selection  "#d5c4a1"
    set comment    "#928374"
    set keyword    "#9d0006"
    set string     "#79740e"
    set number     "#8f3f71"
    set type       "#076678"
    set function   "#b57614"
    set error      "#9d0006"
    set warning    "#af3a03"
    set gutter-bg  "#fbf1c7"
    set gutter-fg  "#a89984"
    set status-bg  "#ebdbb2"
    set status-fg  "#504945"
    set tree-bg    "#fbf1c7"
    set tree-fg    "#3c3836"
}"##;

const CATPPUCCIN: &str = r##"context theme {
    set bg         "#1e1e2e"
    set fg         "#cdd6f4"
    set cursor     "#f5e0dc"
    set selection  "#45475a"
    set comment    "#6c7086"
    set keyword    "#cba6f7"
    set string     "#a6e3a1"
    set number     "#fab387"
    set type       "#89b4fa"
    set function   "#f9e2af"
    set error      "#f38ba8"
    set warning    "#fab387"
    set gutter-bg  "#1e1e2e"
    set gutter-fg  "#585b70"
    set status-bg  "#313244"
    set status-fg  "#a6adc8"
    set tree-bg    "#1e1e2e"
    set tree-fg    "#cdd6f4"
}"##;

const SOLARIZED_DARK: &str = r##"context theme {
    set bg         "#002b36"
    set fg         "#839496"
    set cursor     "#b58900"
    set selection  "#073642"
    set comment    "#586e75"
    set keyword    "#859900"
    set string     "#2aa198"
    set number     "#d33682"
    set type       "#268bd2"
    set function   "#b58900"
    set error      "#dc322f"
    set warning    "#cb4b16"
    set gutter-bg  "#002b36"
    set gutter-fg  "#586e75"
    set status-bg  "#073642"
    set status-fg  "#93a1a1"
    set tree-bg    "#002b36"
    set tree-fg    "#839496"
}"##;

const SOLARIZED_LIGHT: &str = r##"context theme {
    set bg         "#fdf6e3"
    set fg         "#657b83"
    set cursor     "#b58900"
    set selection  "#eee8d5"
    set comment    "#93a1a1"
    set keyword    "#859900"
    set string     "#2aa198"
    set number     "#d33682"
    set type       "#268bd2"
    set function   "#b58900"
    set error      "#dc322f"
    set warning    "#cb4b16"
    set gutter-bg  "#fdf6e3"
    set gutter-fg  "#93a1a1"
    set status-bg  "#eee8d5"
    set status-fg  "#586e75"
    set tree-bg    "#fdf6e3"
    set tree-fg    "#657b83"
}"##;

const ONE_DARK: &str = r##"context theme {
    set bg         "#282c34"
    set fg         "#abb2bf"
    set cursor     "#528bff"
    set selection  "#3e4451"
    set comment    "#5c6370"
    set keyword    "#c678dd"
    set string     "#98c379"
    set number     "#d19a66"
    set type       "#e5c07b"
    set function   "#61afef"
    set error      "#e06c75"
    set warning    "#d19a66"
    set gutter-bg  "#282c34"
    set gutter-fg  "#4b5263"
    set status-bg  "#21252b"
    set status-fg  "#9da5b4"
    set tree-bg    "#282c34"
    set tree-fg    "#abb2bf"
}"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_themes_available() {
        let tv = ThemeValues::new();
        let names = tv.available_themes();
        assert!(names.contains(&"gruvbox-dark".to_string()));
        assert!(names.contains(&"catppuccin".to_string()));
        assert!(names.contains(&"one-dark".to_string()));
    }

    #[test]
    fn find_embedded_theme() {
        let tv = ThemeValues::new();
        let script = tv.find_theme_script("gruvbox-dark");
        assert!(script.is_some());
        assert!(script.unwrap().contains("#282828"));
    }

    #[test]
    fn unknown_theme_returns_none() {
        let tv = ThemeValues::new();
        assert!(tv.find_theme_script("nonexistent").is_none());
    }

    #[test]
    fn get_set_property() {
        let mut tv = ThemeValues::new();
        tv.set("bg", "#000000");
        assert_eq!(tv.get("bg"), Some("#000000"));
    }

    #[test]
    fn missing_property_returns_none() {
        let tv = ThemeValues::new();
        assert!(tv.get("nonexistent").is_none());
    }
}
