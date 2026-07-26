//! Theme and color palette manager for Nexus TUI and CLI.

/// Color Palette definitions for terminal rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    pub primary: &'static str,   // ANSI color code
    pub secondary: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub reset: &'static str,
}

impl Theme {
    /// Dark theme (default).
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            primary: "\x1b[1;34m",   // Bold Blue
            secondary: "\x1b[1;36m", // Bold Cyan
            success: "\x1b[1;32m",   // Bold Green
            warning: "\x1b[1;33m",   // Bold Yellow
            error: "\x1b[1;31m",     // Bold Red
            reset: "\x1b[0m",
        }
    }

    /// Cyberpunk theme.
    pub fn cyberpunk() -> Self {
        Self {
            name: "cyberpunk".to_string(),
            primary: "\x1b[1;35m",   // Bold Magenta
            secondary: "\x1b[1;33m", // Bold Yellow
            success: "\x1b[1;36m",   // Bold Cyan
            warning: "\x1b[1;32m",   // Bold Green
            error: "\x1b[1;31m",     // Bold Red
            reset: "\x1b[0m",
        }
    }

    pub fn apply_primary(&self, text: &str) -> String {
        format!("{}{}{}", self.primary, text, self.reset)
    }

    pub fn apply_success(&self, text: &str) -> String {
        format!("{}{}{}", self.success, text, self.reset)
    }
}

/// Theme manager registry.
pub struct ThemeManager;

impl ThemeManager {
    pub fn get_theme(name: &str) -> Theme {
        match name.to_lowercase().as_str() {
            "cyberpunk" => Theme::cyberpunk(),
            _ => Theme::dark(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_rendering() {
        let theme = ThemeManager::get_theme("cyberpunk");
        assert_eq!(theme.name, "cyberpunk");
        let styled = theme.apply_primary("Nexus");
        assert!(styled.contains("Nexus"));
        assert!(styled.contains("\x1b[1;35m"));
    }
}
