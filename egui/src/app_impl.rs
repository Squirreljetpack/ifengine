use egui::{FontFamily, FontId, Margin, RichText, TextStyle};
use egui_alignments::center_vertical;
use egui_snarl::ui::SnarlWidget;

use crate::{
    App,
    graph::GraphViewer,
    render::render,
    theme::{global_theme, global_theme_mut},
    utils::{UiExt, show_overlay},
};

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    #[cfg(feature = "serde")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ctx.all_styles_mut(|x| {
        //     dbg!(&x.text_styles);
        // });
        let light = global_theme().is_light();

        let mut show_graph = self.show_graph;
        if show_graph {
            show_overlay(
                ctx,
                |ui| {
                    let (snarl, viewer) = self.graph_viewer.get_or_insert_with(|| {
                        let (snarl, prefix_len) =
                        crate::graph::new_snarl(ui.available_width(), ui.available_height());
                        (
                            snarl,
                            GraphViewer {
                                prefix_len,
                                init_transform: None,
                            },
                        )
                    });
                    SnarlWidget::new()
                    .id(egui::Id::new("snarl-demo"))
                    .style(global_theme().snarl)
                    .show(snarl, viewer, ui);
                },
                || {
                    show_graph = false;
                },
            );
        }
        self.show_graph = show_graph;
        egui::TopBottomPanel::top("top_panel")
        .frame(egui::Frame {
            inner_margin: egui::Margin::same(10),
            ..Default::default()
        })
        .show_separator_line(false)
        .show(ctx, |ui| {
            let hovered = ui
            .ctx()
            .input(|i| i.pointer.interact_pos())
            .map(|pos| ui.response().rect.contains(pos))
            .unwrap_or(false);
            ui.add_space(10.0);
            ui.horizontal_centered_labels(self.header(), |ui| {
                ui.add_menu(hovered, light, |ui| {
                    ui.style_mut().spacing.item_spacing = MENU_SPACING;
                    if ui.button("Graph").clicked() {
                        self.show_graph = true;
                    }
                    ui.add_submenu("Themes", |ui| {
                        ui.style_mut().spacing.item_spacing = MENU_SPACING;

                        let variants = global_theme().variants();
                        for variant in variants {
                            if ui.button(variant).clicked() {
                                global_theme_mut().switch(variant, ui.ctx());
                            }
                        }
                    })
                });
            });
        });


        let resp = match self.game.view() {
            Ok(view) => view,
            Err(e) => {
                dbg!(e);
                todo!()
            }
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            center_vertical(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    let width = ui.available_width().min(800.0);
                    ui.set_width(width);
                    render(resp, ui, &mut self.game);
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel")
        // .frame(egui::Frame {
        //     inner_margin: egui::Margin {
        //         left: 0,
        //         right: 2,
        //         top: 0,
        //         bottom: 2,
        //     },
        //     ..Default::default()
        // })
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.add_custom_link(
                    RichText::new("Made with IfEngine").size(10.0),
                    "https://github.com/Squirreljetpack/ifengine",
                )
            });
        });
    }
}

// Include the generated fonts module
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated_fonts.rs"));
}

pub static MENU_SPACING: egui::Vec2 = egui::vec2(0.0, 12.0);
pub static TEXT_SMALL: f32 = 16.0;

pub fn new_app(cc: &eframe::CreationContext<'_>) -> App {
    // This is also where you can customize the look and feel of egui using
    // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

    let mut fonts = egui::FontDefinitions::default();
    generated::add_fonts(&mut fonts);

    cc.egui_ctx.set_fonts(fonts);

    egui_extras::install_image_loaders(&cc.egui_ctx);
    cc.egui_ctx.all_styles_mut(|style| {
        for (text_style, font_id) in style.text_styles.iter_mut() {
            *font_id = match text_style {
                TextStyle::Heading => FontId::new(48.0, FontFamily::Proportional), // default proportional
                TextStyle::Body => FontId::new(20.0, FontFamily::Proportional),
                TextStyle::Monospace => FontId::new(20.0, FontFamily::Monospace),
                TextStyle::Button => FontId::new(18.0, FontFamily::Proportional),
                TextStyle::Small => FontId::new(TEXT_SMALL, FontFamily::Name("quote".into())),
                _ => FontId::new(16.0, FontFamily::Proportional),
            };
        }
        style.text_styles.insert(
            TextStyle::Name("title".into()),
            FontId::new(80.0, FontFamily::Name("title".into())),
        );
        style.spacing.menu_margin = Margin::symmetric(10, 10)
    });

    let theme = if cc.egui_ctx.style().visuals.dark_mode {
        "dark"
    } else {
        "light"
    };
    global_theme_mut().switch(theme, &cc.egui_ctx); // todo: doesn't work

    if on_graph_route() {
        let mut ret = App::new();
        ret.show_graph = true;
        ret
    } else {
        App::new()
    }
    // Load previous app state (if any).
    // Note that you must enable the `persistence` feature for this to work.
    // if let Some(storage) = cc.storage {
    //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_else(|| story::new())
    // } else {
    //     story::new()
    // }
}


#[cfg(target_arch = "wasm32")]
fn on_graph_route() -> bool {
    use web_sys::window;

    if let Some(win) = window() {
        let loc = win.location();
        let pathname = loc.pathname().unwrap_or_default(); // e.g., "/graph"
        let hash = loc.hash().unwrap_or_default();         // e.g., "#graph"

        pathname == "/graph" || hash == "#graph"
    } else {
        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn on_graph_route() -> bool {
    false
}
