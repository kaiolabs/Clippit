use clippit_core::Config;

pub struct ThemeModel {
    current_theme: String,
    background: String,
    foreground: String,
    selection: String,
    border: String,
}

impl ThemeModel {
    pub fn new() -> Self {
        let mut model = Self {
            current_theme: String::from("Dark"),
            background: String::new(),
            foreground: String::new(),
            selection: String::new(),
            border: String::new(),
        };
        model.load_theme("Dark".to_string());
        model
    }

    pub fn load_theme(&mut self, theme_name: String) {
        let config = Config::load().unwrap_or_default();

        let colors = if theme_name == "dark" || theme_name == "Dark" {
            &config.ui.colors.dark
        } else {
            &config.ui.colors.light
        };

        self.current_theme = theme_name;
        self.background = colors.background.clone();
        self.foreground = colors.foreground.clone();
        self.selection = colors.selection.clone();
        self.border = colors.border.clone();
    }

    pub fn save_custom_theme(&self) -> bool {
        let mut config = Config::load().unwrap_or_default();

        let is_dark = self.current_theme == "dark";
        let colors = if is_dark {
            &mut config.ui.colors.dark
        } else {
            &mut config.ui.colors.light
        };

        colors.background = self.background.clone();
        colors.foreground = self.foreground.clone();
        colors.selection = self.selection.clone();
        colors.border = self.border.clone();

        config.save().is_ok()
    }

    pub fn get_available_themes(&self) -> Vec<String> {
        vec![
            "Dark".to_string(),
            "Light".to_string(),
            "Nord".to_string(),
            "Dracula".to_string(),
            "Gruvbox".to_string(),
            "Solarized".to_string(),
        ]
    }

    pub fn get_current_theme(&self) -> String {
        self.current_theme.clone()
    }

    pub fn get_background(&self) -> String {
        self.background.clone()
    }

    pub fn get_foreground(&self) -> String {
        self.foreground.clone()
    }

    pub fn get_selection(&self) -> String {
        self.selection.clone()
    }

    pub fn get_border(&self) -> String {
        self.border.clone()
    }
}

impl Default for ThemeModel {
    fn default() -> Self {
        Self::new()
    }
}
