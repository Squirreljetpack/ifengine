use easy_ext::ext;
use egui::{
    Area, Button, Color32, Context, Image, IntoAtoms, Label, Response, RichText, TextStyle, Ui,
    containers::menu, include_image,
};
use egui_alignments::center_horizontal;

use crate::app_impl::TEXT_SMALL;
use crate::theme::global_theme;

#[ext(UiExt)]
impl Ui {
    pub fn draw_heading(&mut self, mut text: RichText, level: u8) -> Response {
        let (size, margin) = match level {
            0 => (144.0, 20.0),
            1 => (96.0, 16.0),
            2 => (64.0, 12.0),
            3 => (48.0, 8.0),
            4 => (32.0, 6.0),
            5 => (26.0, 5.0),
            _ => (22.0, 4.0),
        };

        text = text
            .size(size)
            .strong()
            .color(global_theme().get_color("strong").unwrap_or(Color32::WHITE));

        self.add_space(margin);
        let resp = self.label(text);
        self.add_space(margin);

        resp
    }

    pub fn draw_empty(&mut self, n: u8) {
        let style = self.ctx().style();
        let body = style.text_styles.get(&TextStyle::Body);
        let row_height = body.map_or(18.0, |f| f.size);
        self.add_space(row_height * n as f32);
    }

    pub fn horizontal_centered_labels<I, R>(
        &mut self,
        labels: Vec<I>,
        add_left: impl FnOnce(&mut Ui) -> R,
    ) -> ()
    where
        I: Into<egui::RichText>,
    {
        // let margin = self.min_rect().min.x;
        // let available_width = self.available_width() - 2.0 * margin;
        // let column_width = available_width / n as f32;

        self.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            ui.add_space(10.0); // push right
            let _ = add_left(ui);

            let n = labels.len();
            if n == 0 {
                return;
            }
            let available_width = ui.available_width();
            let column_width = available_width / n as f32;

            for l in labels {
                ui.allocate_ui([column_width, 0.0].into(), |sub_ui| {
                    sub_ui.set_width(column_width);
                    center_horizontal(sub_ui, |ui| {
                        ui.label(l.into().size(TEXT_SMALL));
                    });
                });
            }
        });
    }

    pub fn add_menu<R>(
        &mut self,
        show: bool,
        light_theme: bool,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        let img = Image::new(include_image!("../../assets/imgs/menu.png"))
            .fit_to_exact_size(egui::vec2(17.0, 17.0));

        let img = if show {
            if light_theme {
                img.tint(Color32::from_black_alpha(70))
            } else {
                // dark theme: use override_text_color or default text color, but alpha = 90
                let base = self
                    .visuals()
                    .override_text_color
                    .unwrap_or(self.visuals().text_color());
                let alpha = 90u8;
                img.tint(Color32::from_rgba_premultiplied(
                    base.r(),
                    base.g(),
                    base.b(),
                    alpha,
                ))
            }
        } else {
            img.tint(Color32::from_white_alpha(0)) // invisible
        };

        let btn = Button::image(img).frame(false);

        let (response, inner) = menu::MenuButton::from_button(btn).ui(self, |ui| add_contents(ui));

        egui::InnerResponse::new(inner.map(|i| i.inner), response)
    }

    pub fn add_submenu<'a, R>(
        &mut self,
        label: impl IntoAtoms<'a>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        let (response, inner) = menu::SubMenuButton::from_button(
            Button::new(label).right_text("⟩"), // haven't figured out how to render more glyphs, downloaded fonts mostly don't work
        )
        .ui(self, add_contents);

        egui::InnerResponse::new(inner.map(|i| i.inner), response)
    }

    // open in bg
    pub fn add_custom_link(&mut self, label: impl Into<RichText>, url: &str) -> Response {
        let label = label.into();
        let response = self.add(egui::Label::new(label));
        if response.clicked() {
            self.ctx().open_url(egui::OpenUrl {
                url: url.to_owned(),
                new_tab: true,
            });
        }

        if response.hovered() {
            let rect = response.rect;
            let painter = self.painter();
            let underline_y = rect.bottom() - 1.0; // 1 px above bottom
            painter.line_segment(
                [
                    Pos2::new(rect.left(), underline_y),
                    Pos2::new(rect.right(), underline_y),
                ],
                egui::Stroke::new(1.0, self.visuals().hyperlink_color),
            );
            self.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        response

        // ui.add(
        //     Hyperlink::from_label_and_url(
        //         RichText::new("Made with IfEngine").size(.0),
        //         "https://github.com/Squirreljetpack/ifengine"
        //     )
        // );
    }
}

//
// fn theme(ctx: &egui::Context) -> &'static str {
//     if ctx.style().visuals.dark_mode {
//         "dark"
//     } else {
//         "light"
//     }
// }

use egui::{Id, LayerId, Order, Pos2, Rect, Sense, Vec2};

pub fn show_overlay<Draw, Close>(ctx: &Context, mut draw: Draw, mut on_close: Close)
where
    Draw: FnMut(&mut Ui),
    Close: FnMut(),
{
    let screen_rect = ctx.content_rect();
    let screen_size = screen_rect.size();

    // compute dialog size: 80% of screen, min 800x550
    let mut size = screen_size * 0.8;
    size.x = size.x.max(800.0);
    size.y = size.y.max(550.0);

    let dialog_rect = Rect::from_center_size(screen_rect.center(), size);

    // full-screen semi-transparent background
    let painter = ctx.layer_painter(LayerId::new(Order::Background, Id::new("overlay_bg")));
    painter.rect_filled(
        screen_rect,
        0.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 160),
    );

    // detect clicks outside dialog, put this before drawing the activation listener
    if ctx.input(|i| i.pointer.any_click()) {
        if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
            if !dialog_rect.contains(pos) {
                on_close();
                return;
            }
        }
    }

    // draw overlay UI
    Area::new(Id::new("overlay_area"))
        .order(Order::Foreground)
        .fixed_pos(dialog_rect.min)
        .show(ctx, |ui| {
            // constrain size
            ui.set_max_size(size);

            // frame with background color and rounding
            egui::Frame {
                fill: global_theme().get_color("bg_secondary").unwrap_or_default(),
                inner_margin: egui::Margin::symmetric(10, 5),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.visuals_mut().override_text_color = global_theme().get_color("primary"); // need to override specifically for some reason

                // compute button size and position
                let close_size = Vec2::new(10.0, 10.0);
                let close_pos = Pos2::new(size.x - close_size.x, 2.0); // this controls the size of the ui apparently, use margin to space
                let close_rect = Rect::from_min_size(
                    dialog_rect.min + Vec2::new(close_pos.x, close_pos.y),
                    close_size,
                );

                // draw button without default frame
                let response = ui.put(
                    close_rect,
                    Label::new("×").sense(Sense::click() | Sense::hover()),
                );

                // handle click
                if response.clicked() {
                    on_close();
                }
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                // user content
                draw(ui);
            });
        });
}
