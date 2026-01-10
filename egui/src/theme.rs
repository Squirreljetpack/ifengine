use std::sync::LazyLock;

use egui::Visuals;
use egui_snarl::ui::{NodeLayout, PinPlacement, SnarlStyle};
use phf::phf_map;

pub type ColorMap = phf::Map<&'static str, egui::Color32>;
pub type ColorMaps = phf::Map<&'static str, &'static ColorMap>;

pub struct Theme {
    pub colors: &'static ColorMaps,
    current: &'static str,
    pub snarl: SnarlStyle,
}

pub mod colors {
    include!(concat!(env!("OUT_DIR"), "/generated_colors.rs"));
}
// note: strong (heading) and primary are often the same - not sure if they should be
static COLORS: ColorMaps = phf_map! {
    "dark" => &colors::monokai_original::COLORS_MAP,
    "light" => {
        use colors::monochrome_light::*;
        &phf_map! {
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
            "primary" => STRONG,
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
    "gruvbox_hard" => &colors::gruvbox_hard::COLORS_MAP,
    "gruvbox_soft" => &colors::gruvbox_soft::COLORS_MAP,
    // "ayu_mirage_dark" => &colors::ayu_mirage_dark::COLORS_MAP,
    "laserwave" => &colors::laserwave::COLORS_MAP,
    "md_dark" => &colors::md_dark::COLORS_MAP,
    "nord" => &colors::nord::COLORS_MAP,
    "everforest_light" => &colors::everforest_light::COLORS_MAP,
    "everforest_dark" => &colors::everforest_dark::COLORS_MAP,
    "dark_dim" => &colors::dark_dim::COLORS_MAP,
    "dark_cool" => &colors::dark_cool::COLORS_MAP,
    "monochrome_light_strong" => &colors::monochrome_light_strong::COLORS_MAP,
    // "monocrome_dark" => {
    // todo
    // },
};

static THEME: LazyLock<std::sync::RwLock<Theme>> = LazyLock::new(|| {
    Theme {
        colors: &COLORS,
        current: "light",
        snarl: SnarlStyle {
            node_layout: Some(NodeLayout::coil()),
            pin_placement: Some(PinPlacement::Outside { margin: 0.0 }),
            pin_size: Some(0.0),
            node_frame: Some(egui::Frame {
                inner_margin: egui::Margin::same(8),
                outer_margin: egui::Margin {
                    left: 0,
                    right: 0,
                    top: 0,
                    bottom: 4,
                },
                corner_radius: egui::CornerRadius::same(8),
                stroke: egui::Stroke {
                    width: 1.0,
                    color: egui::Color32::from_gray(100),
                },
                shadow: egui::Shadow::NONE,
                ..Default::default()
            }),
            bg_frame: Some(egui::Frame {
                inner_margin: egui::Margin::ZERO,
                outer_margin: egui::Margin::ZERO,
                corner_radius: egui::CornerRadius::ZERO,
                // fill: egui::Color32::from_gray(40),
                stroke: egui::Stroke::NONE,
                shadow: egui::Shadow::NONE,
                ..Default::default()
            }),
            wire_width: Some(2.0),
            wire_smoothness: Some(1.0), // lower is smoother
            upscale_wire_frame: Some(true),
            wire_frame_size: Some(20.0),
            ..SnarlStyle::new()
        },
        ..Default::default()
    }
    .into()
});

// --------- IMPL -------------
impl Theme {
    /// Change the current theme
    /// Returns: whether theme was set
    pub fn switch(&mut self, variant: &'static str, ctx: &egui::Context) -> bool {
        if !self.colors.contains_key(variant) {
            return false;
        };

        self.current = variant;
        ctx.set_visuals(self.visuals());
        true
    }

    pub fn is_light(&self) -> bool {
        self.current.contains("light")
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

        visuals.override_text_color = colors.get("primary").cloned();

        if let Some(selection_bg) = colors.get("selection_bg") {
            visuals.selection.bg_fill = *selection_bg;
        }

        if let Some(selection_fg) = colors.get("selection_fg") {
            visuals.selection.stroke.color = *selection_fg;
        }

        visuals.weak_text_color = colors.get("muted").cloned();

        if let Some(border) = colors.get("border") {
            visuals.window_stroke.color = *border;
        }

        // hyperlinks
        if let Some(muted) = colors.get("muted") {
            visuals.hyperlink_color = *muted;
        }

        visuals.widgets.hovered.fg_stroke.width = 0.5;

        // todo: include this once we override rich text without style[hover] = true to not apply hover effect
        // if let Some(on_hover) = colors.get("muted2") {
        //     visuals.widgets.hovered.fg_stroke.color = *on_hover;
        // }

        visuals
    }

    pub fn current(&self) -> &str {
        self.current
    }

    pub fn variants(&self) -> Vec<&'static str> {
        let keys: Vec<_> = self.colors.keys().cloned().collect();

        let mut first = Vec::new();
        let mut rest = Vec::new();

        for &k in &keys {
            match k {
                "light" | "dark" => first.push(k),
                _ => rest.push(k),
            }
        }

        // Sort "light" before "dark"
        first.sort_by(|a, b| match (*a, *b) {
            ("light", "dark") => std::cmp::Ordering::Less,
            ("dark", "light") => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        });

        rest.sort(); // sort the rest alphabetically
        first.extend(rest); // combine
        first
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

pub fn global_theme() -> std::sync::RwLockReadGuard<'static, Theme> {
    THEME.read().unwrap()
}

pub fn global_theme_mut() -> std::sync::RwLockWriteGuard<'static, Theme> {
    THEME.write().unwrap()
}

// ----------

impl Default for Theme {
    fn default() -> Self {
        Theme {
            colors: &phf_map! {},
            current: "",
            snarl: SnarlStyle::default(),
        }
    }
}
