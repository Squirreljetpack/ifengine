use egui::Ui;

use ifengine::{
    Game, View,
    core::{GameContext, GameInner},
    view::{Image, ImageVariant, Object},
};

use crate::{
    utils::draw_empty,
    view::{ElementExt, LineExt},
};

// i don't think theres a nice way to extract this to ifengine crate, so this logic ig is fine to require each project to reimplement

pub fn render(view: View, ui: &mut Ui, game: &mut GameInner) {
    let name = view.name();
    for object in view {
        match object {
            Object::Paragraph(line) => {
                draw_empty(1, ui);
                line.ui(ui, game);
                draw_empty(1, ui);
            }
            Object::Text(line, _) => {
                let resp = line.ui(ui, game).interact(egui::Sense::click());
            }
            Object::Choice(key, choices) => {
                draw_empty(1, ui);
                for (i, line) in choices.into_iter() {
                    if line.ui_clicked(ui, game) {
                        game.handle_choice((name.clone(), key.clone()), i);
                    }
                }
                draw_empty(1, ui);
            }
            Object::Image(img) => match img {
                Image { size: [w, h], variant, action, alt } => {
                    let img = match variant {
                        ImageVariant::Local(p) => { todo!() }, // this isn't really possible...
                        ImageVariant::Url(p) => egui::Image::from_uri(p),
                    };

                    let resp = match (w, h) {
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
                    if !alt.is_empty() {
                        resp.on_hover_text(alt);
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
                draw_empty(n, ui);
            }
            Object::Quote(_, _) => {
                todo!()
            }
            Object::Note(_, _) => {
                todo!()
            }
        }
    }
}
