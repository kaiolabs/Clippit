use clippit_core::Config;

pub struct ConfigModel {
    config: Config,
}

impl ConfigModel {
    pub fn new() -> Self {
        Self {
            config: Config::load().unwrap_or_default(),
        }
    }

    pub fn load_config(&mut self) {
        self.config = Config::load().unwrap_or_default();
    }

    pub fn save_config(&self) -> bool {
        self.config.save().is_ok()
    }

    pub fn reset_to_defaults(&mut self) {
        self.config = Config::default();
    }

    pub fn get_max_history_items(&self) -> i32 {
        self.config.general.max_history_items as i32
    }

    pub fn set_max_history_items(&mut self, value: i32) {
        self.config.general.max_history_items = value as usize;
    }

    pub fn get_poll_interval_ms(&self) -> i32 {
        self.config.general.poll_interval_ms as i32
    }

    pub fn set_poll_interval_ms(&mut self, value: i32) {
        self.config.general.poll_interval_ms = value as u64;
    }

    pub fn get_hotkey_modifier(&self) -> String {
        self.config.hotkeys.show_history_modifier.clone()
    }

    pub fn set_hotkey_modifier(&mut self, value: String) {
        self.config.hotkeys.show_history_modifier = value;
    }

    pub fn get_hotkey_key(&self) -> String {
        self.config.hotkeys.show_history_key.clone()
    }

    pub fn set_hotkey_key(&mut self, value: String) {
        self.config.hotkeys.show_history_key = value;
    }

    pub fn get_theme(&self) -> String {
        self.config.ui.theme.clone()
    }

    pub fn set_theme(&mut self, value: String) {
        self.config.ui.theme = value;
    }

    pub fn get_font_family(&self) -> String {
        self.config.ui.font_family.clone()
    }

    pub fn set_font_family(&mut self, value: String) {
        self.config.ui.font_family = value;
    }

    pub fn get_font_size(&self) -> i32 {
        self.config.ui.font_size as i32
    }

    pub fn set_font_size(&mut self, value: i32) {
        self.config.ui.font_size = value as u32;
    }
}

impl Default for ConfigModel {
    fn default() -> Self {
        Self::new()
    }
}
