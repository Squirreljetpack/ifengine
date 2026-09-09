use ifengine::core::PageId;
use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::span::SpanView;
use crate::context::StoryContext;
use crate::transition::{
    generate_transition_style, generate_view_transition_style, parse_transition_classes,
};

/// Renders a [`Line`] consisting of multiple spans, with line-level animation and class support.
#[component]
pub fn LineView(line: Line, page_id: PageId) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let config = parse_transition_classes(&line.classes);
    let is_changed = ctx
        .transitions
        .write_untracked()
        .is_content_changed(&page_id, line.id, line.content_hash());
    let should_animate = config.has_transition() && is_changed;

    let (transition_style, extra_classes) = generate_transition_style(&config, should_animate);
    let vt_style = generate_view_transition_style(line.id, is_changed);

    let mut styles = Vec::new();
    if !transition_style.is_empty() {
        styles.push(transition_style);
    }
    if !vt_style.is_empty() {
        styles.push(vt_style);
    }
    let combined_style = styles.join(" ");

    let mut line_classes = line.classes.clone();
    line_classes.push("passage-line".into());
    for extra in extra_classes {
        line_classes.push(extra.into());
    }
    let class_str = line_classes.join(" ");

    let spans = line.spans;

    view! {
        <span class=class_str style=combined_style>
            {spans.into_iter().map(|span| {
                let page_id_clone = page_id.clone();
                view! {
                    <SpanView
                        span=span
                        page_id=page_id_clone
                    />
                }
            }).collect::<Vec<_>>()}
        </span>
    }
}
