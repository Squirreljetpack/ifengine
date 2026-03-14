use crate::{theme::global_theme, utils::UiExt};

use easy_ext::ext;
use egui::{Color32, FontFamily, Response, RichText, Ui};
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
            SpanVariant::Link => m |= Modifier::ITALIC,
            _ => {}
        }
        
        // ---- font variants (True Italic/Bold) ----
        let has_bold = m.contains(Modifier::BOLD);
        let has_italic = m.contains(Modifier::ITALIC);
        let mut variant_applied = false;
        
        if has_bold || has_italic {
            // Determine base family - default to proportional
            let base_family = if m.contains(Modifier::SUBSCRIPT) {
                "quote"
            } else {
                "proportional"
            };
            
            let variant = match (has_bold, has_italic) {
                (true, true) => format!("{}-bold-italic", base_family),
                (true, false) => format!("{}-bold", base_family),
                (false, true) => format!("{}-italic", base_family),
                _ => unreachable!(),
            };
            
            // Note: We check if the font exists by name? 
            // Actually egui will just fall back to default if Name doesn't exist.
            // For now we assume they exist because our build.rs generates them.
            txt = txt.family(FontFamily::Name(variant.into()));
            variant_applied = true;
        }
        
        // ---- core style fields ----
        // Only apply faux styles if we didn't use a dedicated font file
        if has_bold && !variant_applied {
            txt = txt.strong();
        }
        if m.contains(Modifier::DIM) {
            txt = txt.weak();
        }
        if has_italic && !variant_applied {
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
    
    pub const UNDERLINE_TEXT_Y_OFFSET: f32 = -0.0;
    pub const UNDERLINE_Y_OFFSET: f32 = -6.0;
    
    // Supported style keys:
    // hover: bool
    // color: ThemeColor
    // handle underline manually due to egui exaggerating line height offset
    // Link Variant gets a cursor change
    pub fn add(&self, ui: &mut Ui, sense: bool) -> Response {
        let rich = self.as_rich_text().with_line_height(ui, 1.6);
        
        let needs_underline = self.modifiers.contains(Modifier::UNDERLINE);
        
        let mut lbl = egui::Label::new(rich);
        if sense {
            lbl = lbl.sense(egui::Sense::click());
        }
        
        if needs_underline {
            let (pos, galley, response) = lbl.layout_in_ui(ui);
            
            // Shift the actual text draw position
            let text_pos = egui::pos2(pos.x, pos.y + Self::UNDERLINE_TEXT_Y_OFFSET);
            
            ui.painter()
            .galley(text_pos, galley.clone(), ui.visuals().text_color());
            
            let color = ui.visuals().text_color();
            let stroke = egui::Stroke::new(1.0, color);
            
            for r in galley.rows.iter() {
                let x_start = r.rect_without_leading_space().min.x + pos.x + 1.0;
                let w = r.rect_without_leading_space().width() - 2.0;
                
                // Anchor row positions to text_pos.y, (instead of pos.y)
                // This ensures moving the text moves the underline and hover zone with it.
                let row_top = text_pos.y + r.pos.y;
                let row_bottom = row_top + r.size.y;
                
                // Apply underline offset relative to the drawn text
                let underline_y = row_bottom + Self::UNDERLINE_Y_OFFSET;
                
                ui.painter().line_segment(
                    [
                    egui::pos2(x_start, underline_y),
                    egui::pos2(x_start + w, underline_y),
                    ],
                    stroke,
                );
                
                // add interaction
                if matches!(self.variant, SpanVariant::Link) {
                    let text_rect = egui::Rect::from_min_max(
                        egui::pos2(x_start, row_top),
                        egui::pos2(x_start + w, row_bottom),
                    );
                    
                    let interact_id = ui.id().with(r.pos.x.to_bits()).with(r.pos.y.to_bits());
                    let hover = ui.interact(text_rect, interact_id, egui::Sense::hover());
                    
                    if hover.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            }
            
            response
        } else {
            let response = ui.add(lbl);
            
            if matches!(self.variant, SpanVariant::Link) && response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            
            response
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
    pub fn ui(self, ui: &mut Ui, mut game: Option<&mut GameInner>) -> Response {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);
            
            ui.horizontal_wrapped(|ui| {
                for mut element in self.spans {
                    if let Some(action) = element.action.take()
                    && let Some(game) = game.as_mut()
                    {
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
    
    pub fn ui_clicked(self, ui: &mut Ui, mut game: Option<&mut GameInner>) -> bool {
        let mut clicked = false;
        let ui_resp = ui.horizontal_wrapped(|ui| {
            for mut element in self.spans {
                if let Some(action) = element.action.take()
                && let Some(game) = game.as_mut()
                {
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
        .response
        .interact(egui::Sense::click());
        
        clicked || ui_resp.clicked()
    }
}
