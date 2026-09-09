use ifengine::core::PageId;
use ifengine::view::{Span, SpanVariant};
use leptos::prelude::*;

use crate::context::StoryContext;
use crate::render::{span_classes, span_to_css_style};
use crate::transition::{
    generate_transition_style, generate_view_transition_style, parse_transition_classes,
};

/// Renders a single [`Span`] element with styling, modifiers, transition animation,
/// and interactive action handling.
#[component]
pub fn SpanView(span: Span, page_id: PageId) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let config = parse_transition_classes(&span.classes);
    let is_changed = ctx
        .transitions
        .write_untracked()
        .is_content_changed(&page_id, span.id, span.content_hash());
    let should_animate = config.has_transition() && is_changed;

    let (transition_style, extra_classes) = generate_transition_style(&config, should_animate);
    let vt_style = generate_view_transition_style(span.id, is_changed);

    let base_style = span_to_css_style(&span);
    let mut styles = Vec::new();
    if !base_style.is_empty() {
        styles.push(base_style);
    }
    if !transition_style.is_empty() {
        styles.push(transition_style);
    }
    if !vt_style.is_empty() {
        styles.push(vt_style);
    }
    let combined_style = styles.join(" ");

    let class_str = span_classes(&span, &extra_classes);

    if let Some(action) = span.action {
        view! {
            <a
                href="javascript:void(0)"
                class=format!("passage-link {class_str}")
                style=combined_style
                on:click=move |e: leptos::ev::MouseEvent| {
                    e.prevent_default();
                    ctx.dispatch_action.run(action.clone());
                }
            >
                {span.content}
            </a>
        }
        .into_any()
    } else if matches!(span.variant, SpanVariant::Link) {
        view! {
            <span class=format!("passage-link-static {class_str}") style=combined_style>
                {span.content}
            </span>
        }
        .into_any()
    } else {
        view! {
            <span class=class_str style=combined_style>
                {span.content}
            </span>
        }
        .into_any()
    }
}
