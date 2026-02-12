use super::state::CrapApp;
use crate::models::ThemeMode;
use eframe::egui;

impl CrapApp {
    pub fn set_theme(&mut self, theme: ThemeMode) {
        self.theme = theme;
        self.apply_theme();

        let db = self.db.clone();
        let val = theme.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("theme", &val).await;
        });
    }

    pub fn apply_theme(&self) {
        match self.theme {
            ThemeMode::System => {
                self.ctx.set_style(egui::Style::default());
            }
            ThemeMode::Light => {
                self.ctx.set_visuals(egui::Visuals::light());
            }
            ThemeMode::Dark => {
                self.ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.ui_scale = scale;
        self.ctx.set_pixels_per_point(scale);

        let db = self.db.clone();
        let val = scale.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("ui_scale", &val).await;
        });
    }

    pub fn set_custom_background_mode(&mut self, enabled: bool) {
        self.use_custom_background = enabled;
        let db = self.db.clone();
        let val = if enabled { "true" } else { "false" };
        tokio::spawn(async move {
            let _ = db.set_setting("use_custom_background", val).await;
        });
    }

    pub fn set_watermark_visibility(&mut self, visible: bool) {
        self.show_watermark = visible;
        let db = self.db.clone();
        let val = visible.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("show_watermark", &val).await;
        });
    }

    pub fn set_background_visibility(&mut self, visible: bool) {
        self.show_background = visible;
        let db = self.db.clone();
        let val = visible.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("show_background", &val).await;
        });
    }

    pub fn set_spell_check(&mut self, enabled: bool) {
        self.enable_spell_check = enabled;
        let db = self.db.clone();
        let val = enabled.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("enable_spell_check", &val).await;
        });
    }

    pub fn set_background_scale(&mut self, scale: f32) {
        self.background_scale = scale;
        self.ctx.request_repaint();

        let db = self.db.clone();
        tokio::spawn(async move {
            let _ = db.set_setting("background_scale", &scale.to_string()).await;
        });
    }
}
