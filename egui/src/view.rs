use crate::{theme::global_theme, utils::UiExt};

use easy_ext::ext;
use egui::{Color32, Response, RichText, Ui};
use ifengine::{
    core::GameInner,
    view::{Line, Modifier, Span, SpanVariant},
};

#[ext(ElementExt)]
impl Span {
    pub fn as_rich_text(&self) -> RichText {
        let mut txt = RichText::new(&self.content);
        let mut m = self.modifiers;

        match self.variant {
            SpanVariant::Link => m |= Modifier::UNDERLINE,
            _ => {}
        }

        // ---- core style fields ----
        if m.contains(Modifier::BOLD) {
            txt = txt.strong();
        }
        if m.contains(Modifier::DIM) {
            txt = txt.weak();
        }
        if m.contains(Modifier::ITALIC) {
            txt = txt.italics();
        }
        // if m.contains(Modifier::UNDERLINE) {
        //     txt = txt.underline();
        // }
        if m.contains(Modifier::STRIKETHROUGH) {
            txt = txt.strikethrough();
        }
        if m.contains(Modifier::HIDDEN) {
            txt = txt.color(Color32::TRANSPARENT);
        }
        if m.contains(Modifier::SUPER_SCRIPT) {
            txt = txt.small_raised();
        }
        if m.contains(Modifier::SUBSCRIPT) {
            txt = txt.small();
        }
        if m.contains(Modifier::REVERSED) {
            unimplemented!()
        }

        // ---- style map ------
        if let Some(fg) = self.style.get("color") {
            if let Some(col) = global_theme().get_color(fg) {
                txt = txt.color(col);
            }
        }

        if let Some(bg) = self.style.get("background") {
            if let Some(col) = global_theme().get_color(bg) {
                txt = txt.background_color(col);
            }
        }
        // Font size
        if let Some(size) = self.style.get("font-size") {
            if let Ok(px) = size.parse::<f32>() {
                txt = txt.size(px);
            }
        }

        if let Some(hover) = self.style.get("hover")
            && hover == "true"
        {
            //
        } else {
            // todo
        }

        // ---- font styles? ------

        txt
    }

    pub fn add_as_heading(&self, ui: &mut Ui, level: u8) -> Response {
        ui.draw_heading(self.as_rich_text(), level)
    }

    // Supported style keys:
    // hover: bool
    // color: ThemeColor
    // handle underline manually due to egui exaggerating line height offset
    // Link Variant gets a cursor change
    pub fn add(self, ui: &mut Ui, sense: bool) -> Response {
        let rich = self.as_rich_text().with_line_height(ui, 1.6);

        let needs_underline = self.modifiers.contains(Modifier::UNDERLINE)
            || matches!(self.variant, SpanVariant::Link);

        let mut lbl = egui::Label::new(rich);
        if sense {
            lbl = lbl.sense(egui::Sense::click());
        }

        if needs_underline {
            let (pos, galley, response) = lbl.layout_in_ui(ui);
            ui.painter()
                .galley(pos, galley.clone(), ui.visuals().text_color());

            let line_rects = galley.rows.iter().map(|r| {
                let x = r.rect_without_leading_space().min.x;
                [
                    x + pos.x + 1.0,
                    pos.y + r.pos.y + r.size.y - 2.5,
                    r.size.x - x - 1.0,
                    r.size.y,
                ]
            });

            let color = ui.visuals().text_color();
            let stroke = egui::Stroke::new(1.0, color);

            for [x, y, w, h] in line_rects {
                ui.painter()
                    .line_segment([egui::pos2(x, y), egui::pos2(x + w, y)], stroke);

                if matches!(self.variant, SpanVariant::Link) {
                    let text_rect =
                        egui::Rect::from_min_max(egui::pos2(x, y - h), egui::pos2(x + w, y));
                    let hover = ui.allocate_rect(text_rect, egui::Sense::hover());
                    if hover.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            }

            response
        } else {
            ui.add(lbl)
        }
    }
}

#[ext(RichTextExt)]
impl RichText {
    fn with_line_height(mut self, ui: &Ui, factor: f32) -> RichText {
        self = self.line_height(Some(
            ui.style().text_styles[&egui::TextStyle::Body].size * factor,
        ));
        self
    }
}
// todo: configurable effect on sensed hover
#[ext(LineExt)]
impl Line {
    pub fn ui(self, ui: &mut Ui, game: &mut GameInner) -> Response {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);

            ui.horizontal_wrapped(|ui| {
                for mut element in self.spans {
                    if let Some(action) = element.action.take() {
                        if element.add(ui, true).clicked() {
                            let _ = game.handle_action(action);
                        };
                    } else {
                        element.add(ui, false);
                    }
                }
            })
        })
        .response
    }

    pub fn ui_clicked(self, ui: &mut Ui, game: &mut GameInner) -> bool {
        let mut clicked = false;
        let ui_resp = ui
            .scope(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);

                ui.horizontal_wrapped(|ui| {
                    for mut element in self.spans {
                        if let Some(action) = element.action.take() {
                            if element.add(ui, true).clicked() {
                                let _ = game.handle_action(action);
                            };
                        } else {
                            if element.add(ui, false).clicked() {
                                clicked = true
                            }
                        }
                    }
                })
            })
            .response
            .interact(egui::Sense::click());

        clicked || ui_resp.clicked()
    }
}
