use egui::{Response, RichText, TextStyle, Ui};
use egui_alignments::center_horizontal;

pub fn draw_heading(ui: &mut Ui, mut text: RichText, level: u8) -> Response {
    let (size, margin) = match level {
        1 => (80.0, 16.0),
        2 => (48.0, 12.0),
        3 => (32.0, 8.0),
        4 => (24.0, 6.0),
        _ => (18.0, 4.0),
    };

    text = text.size(size).strong();

    ui.add_space(margin);
    let resp = ui.label(text);
    ui.add_space(margin);

    resp
}

pub fn draw_empty(n: u8, ui: &mut Ui) {
    let style = ui.ctx().style();
    let body = style.text_styles.get(&TextStyle::Body);
    let row_height = body.map_or(18.0, |f| f.size);
    ui.add_space(row_height * n as f32);
}

pub fn horizontal_centered_labels<I>(ui: &mut Ui, labels: Vec<I>)
where
    I: Into<egui::WidgetText>,
{
    let n = labels.len();
    if n == 0 {
        return;
    }

    // ui.spacing_mut doesn't seem to store the frame inner margin, so we assume same and get from min
    let margin = ui.min_rect().min.x;

    let available_width = ui.available_width() - 2.0 * margin; // includes margin so set to 0
    let column_width = available_width / n as f32;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        for l in labels {
            ui.allocate_ui([column_width, 0.0].into(), |sub_ui| {
                sub_ui.set_width(column_width);
                center_horizontal(sub_ui, |ui| {
                    ui.label(l);
                });
            });
        }
    });
}

//
// fn theme(ctx: &egui::Context) -> &'static str {
//     if ctx.style().visuals.dark_mode {
//         "dark"
//     } else {
//         "light"
//     }
// }
