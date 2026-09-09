//! Helper functions for rendering styles, modifiers, and header metadata.

use ifengine::view::{Modifier, Span, SpanVariant};

/// Extracts header statistics (Day, Miles travelled, Rations) from story state if available.
#[cfg(all(feature = "saltwrack", not(feature = "test")))]
pub fn extract_header(game: &story::Game) -> Vec<String> {
    if game.context.miles != 0 {
        vec![
            format!("Day: {}", game.context.days),
            format!("Miles travelled: {}", game.context.miles),
            format!("Rations: {}", game.context.rations),
        ]
    } else {
        vec![]
    }
}

/// Extracts header statistics when using `test_story` (empty status header).
#[cfg(any(feature = "test", not(feature = "saltwrack")))]
pub fn extract_header(_game: &story::Game) -> Vec<String> {
    vec![]
}

/// Converts [`Modifier`] bitflags and custom span inline styles to a CSS style string.
pub fn span_to_css_style(span: &Span) -> String {
    let mut styles = Vec::new();
    let m = span.modifiers;

    if m.contains(Modifier::BOLD) {
        styles.push("font-weight: bold;".to_string());
    }
    if m.contains(Modifier::ITALIC) {
        styles.push("font-style: italic;".to_string());
    }
    if m.contains(Modifier::DIM) || matches!(span.variant, SpanVariant::Muted) {
        styles.push("opacity: 0.65;".to_string());
    }
    if m.contains(Modifier::UNDERLINE) || matches!(span.variant, SpanVariant::Link) {
        styles.push("text-decoration: underline; text-underline-offset: 3px;".to_string());
    }
    if m.contains(Modifier::STRIKETHROUGH) {
        styles.push("text-decoration: line-through;".to_string());
    }
    if m.contains(Modifier::HIDDEN) {
        styles.push("display: none;".to_string());
    }
    if m.contains(Modifier::SUPER_SCRIPT) {
        styles.push("vertical-align: super; font-size: 0.8em;".to_string());
    }
    if m.contains(Modifier::SUBSCRIPT) {
        styles.push("vertical-align: sub; font-size: 0.8em;".to_string());
    }

    if matches!(span.variant, SpanVariant::Secondary) {
        styles.push("color: var(--color-secondary, #adb5bd);".to_string());
    }

    // Custom inline style map
    for (k, v) in &span.style {
        match k.as_str() {
            "color" => styles.push(format!("color: {v};")),
            "background" => styles.push(format!("background-color: {v};")),
            "font-size" => styles.push(format!("font-size: {v};")),
            _ => styles.push(format!("{k}: {v};")),
        }
    }

    styles.join(" ")
}

/// Generates the CSS class string for a span.
pub fn span_classes(span: &Span, extra_classes: &[&str]) -> String {
    let mut classes = span.classes.clone();

    match span.variant {
        SpanVariant::Link => classes.push("variant-link".into()),
        SpanVariant::Muted => classes.push("variant-muted".into()),
        SpanVariant::Secondary => classes.push("variant-secondary".into()),
        SpanVariant::None => {}
    }

    if span.action.is_some() {
        classes.push("interactive-action".into());
    }

    for &extra in extra_classes {
        classes.push(extra.into());
    }

    classes.join(" ")
}
