use egui::Ui;

use ifengine::{
    View,
    core::GameInner,
    view::{Image, ImageVariant, Object},
};

use crate::{
    utils::UiExt, view::{ElementExt, LineExt}
};

// i don't think theres a nice way to extract this to ifengine crate, so this logic ig is fine to require each project to reimplement

pub fn render(view: View, ui: &mut Ui, game: &mut GameInner) {
    let name = view.name();
    for object in view {
        match object {
            Object::Paragraph(line) => {
                ui.draw_empty(1);
                line.ui(ui, game);
                ui.draw_empty(1);
            }
            Object::Text(line, _) => {
                let _ = line.ui(ui, game).interact(egui::Sense::click());
            }
            Object::Choice(key, choices) => {
                ui.draw_empty(1);
                for (i, line) in choices.into_iter() {
                    if line.ui_clicked(ui, game) {
                        game.handle_choice((name.clone(), key.clone()), i);
                    }
                }
                ui.draw_empty(1);
            }
            Object::Image(img) => match img {
                Image { size: [w, h], variant, action, alt } => {
                    let img = match variant {
                        ImageVariant::Local(uri, bytes) => {
                            egui::Image::from_bytes(uri, bytes)
                        },
                        ImageVariant::Url(p) => egui::Image::from_uri(p),
                    };

                    let mut resp = match (w, h) {
                        (0, 0) => {
                            ui.add(img)
                        }
                        (0, h) => {
                            ui.add(img.max_height(h as f32))
                        }
                        (w, 0) => {
                            ui.add(img.max_width(w as f32))
                        }
                        (w, h) => {
                            ui.add(img.fit_to_exact_size(egui::vec2(w as f32, h as f32)))
                        }
                    };
                    resp = if !alt.is_empty() {
                        resp.on_hover_text(alt)
                    } else {
                        resp
                    };
                    if let Some(action) = action {
                        if resp.clicked() {
                            let _ = game.handle_action(action);
                        }
                    }

                }
            },
            Object::Heading(line, level) => {
                line.add_as_heading(ui, level);
            }
            Object::Break => {
                ui.add(egui::Separator::default());
            }
            Object::Empty(n) => {
                ui.draw_empty(n);
            }
            Object::Quote(_, _) => {
                todo!()
            }
            Object::Note(_, _) => {
                todo!()
            }
            Object::Custom(_) => {
                unimplemented!()
            }
        }
    }
}
