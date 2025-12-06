use egui::{FontFamily, FontId, Image, TextStyle, include_image};
use egui_alignments::center_vertical;
use egui_snarl::ui::SnarlWidget;
use ifengine::Game;

use crate::{App, graph::GraphViewer, render::render, theme::{global_theme, global_theme_mut}, utils::{UiExt, show_overlay}};

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
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
            show_overlay(ctx,
                |ui| {
                    // ui.vertical_centered(|ui| {
                    //     ui.label("Hello Overlay!");
                    //     if ui.button("Do something").clicked() {
                    //         println!("Clicked inside overlay");
                    //     }
                    // });

                    let (snarl, viewer) = self.graph_viewer.get_or_insert_with(|| {
                        let (snarl, prefix_len) = crate::graph::new_snarl(ui.available_width(), ui.available_height());
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
                }
            );
        }
        self.show_graph = show_graph;

        egui::TopBottomPanel::top("top_panel")
        .frame(egui::Frame {
            fill: ctx.style().visuals.window_fill, // normal panel background
            // stroke: Stroke::NONE,
            inner_margin: egui::Margin::same(10), // optional padding
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
            ui.horizontal_centered_labels(
                vec![
                format!("Day: {}", self.game.context.days),
                format!("Miles travelled: {}", self.game.context.miles),
                format!("Rations: {}", self.game.context.rations),
                ],
                |ui| {
                    ui.add_menu(hovered, light, |ui| {
                        if ui.button("Graph").clicked() {
                            self.show_graph = true;
                        }
                        ui.add_submenu("Themes", |ui| {
                            let variants = global_theme().variants();
                            for variant in variants {
                                if ui.button(variant).clicked() {
                                    global_theme_mut().switch(variant, ui.ctx());
                                }
                            }
                        })
                    });
                },
            );
        });

        let Ok(resp) = self.game.view() else { todo!() };

        egui::CentralPanel::default().show(ctx, |ui| {
            center_vertical(ui, |ui| {
                // ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                //     ui.add_space(40.0);
                // });
                ui.vertical_centered_justified(|ui| {
                    let width = ui.available_width().min(800.0);
                    ui.set_width(width);
                    render(resp, ui, &mut self.game);

                    // ui.set_max_width(800.0);
                });
            });
        });
    }
}

// Include the generated fonts module
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated_fonts.rs"));
}

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
                TextStyle::Heading => FontId::new(32.0, FontFamily::Proportional), // default proportional
                TextStyle::Body => FontId::new(18.0, FontFamily::Proportional),
                TextStyle::Monospace => FontId::new(16.0, FontFamily::Monospace),
                TextStyle::Button => FontId::new(10.0, FontFamily::Proportional),
                TextStyle::Small => FontId::new(18.0, FontFamily::Name("quote".into())),
                _ => FontId::new(16.0, FontFamily::Proportional),
            };
        }
        style.text_styles.insert(
            TextStyle::Name("title".into()),
            FontId::new(80.0, FontFamily::Name("title".into())),
        );

        // style.spacing.menu_margin.top += 20;
    });
    // cc.egui_ctx.set_zoom_factor(1.0);

    let visuals = global_theme().visuals(); // or detect OS theme, or user pref
    cc.egui_ctx.set_visuals(visuals);

    // Load previous app state (if any).
    // Note that you must enable the `persistence` feature for this to work.
    // if let Some(storage) = cc.storage {
    //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_else(|| story::new())
    // } else {
    //     story::new()
    // }
    App::new()
}
