use std::sync::LazyLock;

use egui::Visuals;
use phf::phf_map;

pub struct Theme {
    pub colors: phf::Map<&'static str, phf::Map<&'static str, egui::Color32>>,
    current: &'static str,
}

pub mod colors {
    include!(concat!(env!("OUT_DIR"), "/generated_colors.rs"));
}

use colors::*;

pub static THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    colors: phf_map! {
        "dark" => {
            use colors::nord::*;
            phf_map! {
                "red" => RED,
                "yellow" => YELLOW,
                "green" => GREEN,
                "blue" => BLUE,
                "purple" => PURPLE,
                "black" => BLACK,
                "white" => WHITE,
                "gray" => GRAY,
                "dark_gray" => DARK_GRAY,
                "light_gray" => LIGHT_GRAY,
                "primary" => PRIMARY,
                "secondary" => SECONDARY,
                "bg" => BG,
                "bg_secondary" => BG_SECONDARY,
                "selection_fg" => SELECTION_FG,
                "selection_bg" => SELECTION_BG,
                "muted2" => MUTED2,
                "muted" => MUTED,
                "strong" => STRONG
            }
        },
        "light" => {
            use colors::monochrome_light::*;
            phf_map! {
                "red" => RED,
                "yellow" => YELLOW,
                "green" => GREEN,
                "blue" => BLUE,
                "purple" => PURPLE,
                "black" => BLACK,
                "white" => WHITE,
                "gray" => GRAY,
                "dark_gray" => DARK_GRAY,
                "light_gray" => LIGHT_GRAY,
                "primary" => PRIMARY,
                "secondary" => SECONDARY,
                "bg" => BG,
                "bg_secondary" => BG_SECONDARY,
                "selection_fg" => SELECTION_FG,
                "selection_bg" => SELECTION_BG,
                "muted2" => MUTED2,
                "muted" => MUTED,
                "strong" => STRONG
            }
        },
    },
    current: "light",
});

impl Theme {
    /// Change the current theme
    pub fn switch(&mut self, variant: &'static str) -> Option<Visuals> {
        let Some(colors) = self.colors.get(variant) else {
            return None;
        };

        self.current = variant;
        Some(self.visuals())
    }

    /// Convert a into an `egui::Visuals` instance
    pub fn visuals(&self) -> Visuals {
        let colors = self.colors.get(self.current()).unwrap();

        let mut visuals = if self.current() == "dark" {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        // Map your theme colors to egui Visuals
        if let Some(bg) = colors.get("bg") {
            visuals.window_fill = *bg;
        }

        if let Some(bg_secondary) = colors.get("bg_secondary") {
            visuals.panel_fill = *bg_secondary;
        }

        if let Some(primary) = colors.get("primary") {
            visuals.override_text_color = Some(*primary);
        }

        if let Some(selection_bg) = colors.get("selection_bg") {
            visuals.selection.bg_fill = *selection_bg;
        }

        if let Some(selection_fg) = colors.get("selection_fg") {
            visuals.selection.stroke.color = *selection_fg;
        }

        if let Some(muted) = colors.get("muted") {
            visuals.weak_text_color = Some(*muted)
        }

        // todo: include this once we override rich text without style[hover] = true to not apply hover effect
        // if let Some(on_hover) = colors.get("muted2") {
        //     visuals.widgets.hovered.fg_stroke.color = *on_hover;
        // }

        visuals
    }

    pub fn current(&self) -> &str {
        self.current
    }

    pub fn get_color(&self, name_or_hex: &str) -> Option<egui::Color32> {
        self.get_color_from_theme(self.current(), name_or_hex)
    }

    pub fn get_color_from_theme(&self, theme: &str, name_or_hex: &str) -> Option<egui::Color32> {
        if let Some(theme_map) = self.colors.get(theme) {
            if let Some(&color) = theme_map.get(name_or_hex) {
                return Some(color);
            }
        }

        // Fallback to parsing as hex
        egui::Color32::from_hex(name_or_hex).ok()
    }
}
