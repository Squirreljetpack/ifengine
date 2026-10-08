use ifengine::View;
use ifengine::view::{Object, SpanVariant};
use ifengine_core::run::Interactable;
use serde_json::json;

pub fn render_view_to_text(story: &str, status: &[String], view: &View) -> String {
    let mut out = String::new();

    out.push_str("================================================================================\n");
    out.push_str(&format!("STORY: {story} | PAGE: {}\n", view.pageid.0));
    if !status.is_empty() {
        out.push_str(&status.join(" | "));
        out.push('\n');
    }
    out.push_str("================================================================================\n\n");

    render_objects_to_text(&view.inner, &mut out, 0);

    let interactables = view.interactables_sim();
    out.push_str("\n--------------------------------------------------------------------------------\n");
    if interactables.is_empty() {
        out.push_str("NO AVAILABLE INTERACTION (Game terminal state or page transition)\n");
    } else {
        out.push_str("AVAILABLE CHOICES & ACTIONS:\n");
        for (i, item) in interactables.iter().enumerate() {
            let kind = match item {
                Interactable::Choice(_, _, _) => "Choice",
                Interactable::Span(_, span) => {
                    if matches!(span.variant, SpanVariant::Link) {
                        "Link"
                    } else {
                        "Action"
                    }
                }
            };
            out.push_str(&format!("  [{i}] ({kind}) {}\n", item.content().trim()));
        }
    }
    out.push_str("================================================================================\n");

    out
}

fn render_objects_to_text(objects: &[ifengine::view::StampedObject], out: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);
    for stamped in objects {
        match &stamped.object {
            Object::Heading(span, level) => {
                let hashes = "#".repeat((*level).clamp(1, 6) as usize);
                out.push_str(&format!("\n{pad}{hashes} {}\n\n", span.content()));
            }
            Object::Paragraph(line, render_data) => {
                let content = line.content();
                if content.trim().is_empty() {
                    continue;
                }
                if let Some(speaker) = render_data.strip_prefix(':') {
                    out.push_str(&format!("{pad}[{speaker}] {content}\n\n"));
                } else if *render_data == "modal" || *render_data == "popup" {
                    out.push_str(&format!("{pad}[MODAL DIALOG]\n{pad}{content}\n\n"));
                } else {
                    out.push_str(&format!("{pad}{content}\n\n"));
                }
            }
            Object::Choice(choices) => {
                for (idx, line) in choices {
                    out.push_str(&format!("{pad}• ({idx}) {}\n", line.content()));
                }
                out.push('\n');
            }
            Object::Break => {
                out.push_str(&format!("{pad}---\n\n"));
            }
            Object::Empty(n) => {
                for _ in 0..*n {
                    out.push('\n');
                }
            }
            Object::Note(line, _) => {
                out.push_str(&format!("{pad}[Note: {}]\n\n", line.content()));
            }
            Object::Embed(sub_view, render_data) => {
                out.push_str(&format!("{pad}[SUB-PAGE: {} (data: '{render_data}')]\n", sub_view.pageid.0));
                render_objects_to_text(&sub_view.inner, out, indent + 1);
            }
            Object::Image(img) => {
                out.push_str(&format!("{pad}[Image: {}]\n\n", img.alt));
            }
        }
    }
}

pub fn render_view_to_json(story: &str, status: &[String], view: &View) -> serde_json::Value {
    let mut prose_lines = Vec::new();
    extract_prose(&view.inner, &mut prose_lines);

    let interactables = view.interactables_sim();
    let items: Vec<serde_json::Value> = interactables
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let kind = match item {
                Interactable::Choice(_, _, _) => "choice",
                Interactable::Span(_, span) => {
                    if matches!(span.variant, SpanVariant::Link) {
                        "link"
                    } else {
                        "action"
                    }
                }
            };
            json!({
                "id": i,
                "kind": kind,
                "content": item.content().to_string(),
            })
        })
        .collect();

    json!({
        "story": story,
        "page_id": view.pageid.0.as_ref(),
        "status": status,
        "prose": prose_lines,
        "interactables": items,
    })
}

fn extract_prose(objects: &[ifengine::view::StampedObject], out: &mut Vec<String>) {
    for stamped in objects {
        match &stamped.object {
            Object::Heading(span, _) => {
                out.push(span.content().to_string());
            }
            Object::Paragraph(line, render_data) => {
                let content = line.content();
                if let Some(speaker) = render_data.strip_prefix(':') {
                    out.push(format!("[{speaker}] {content}"));
                } else {
                    out.push(content);
                }
            }
            Object::Embed(sub_view, _) => {
                extract_prose(&sub_view.inner, out);
            }
            _ => {}
        }
    }
}
