use ifengine::core::PageId;
use ifengine::view::{Span, SpanVariant};
use leptos::prelude::*;

use crate::context::StoryContext;
use crate::render::{span_classes, span_to_css_style};
use crate::transition::{
    compute_initial_phase, generate_active_transition_style, generate_view_transition_style,
    parse_transition_classes, setup_transition_timers, TransitionPhase,
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

    let (phase, set_phase) = signal(compute_initial_phase(&config, should_animate));
    if should_animate {
        setup_transition_timers(&config, set_phase);
    }

    let vt_style = generate_view_transition_style(span.id, is_changed);
    let base_style = span_to_css_style(&span);

    let span_config = config.clone();
    let span_style = move || {
        let mut styles = Vec::new();
        if !base_style.is_empty() {
            styles.push(base_style.clone());
        }
        if !vt_style.is_empty() {
            styles.push(vt_style.clone());
        }
        match phase.get() {
            TransitionPhase::Pending | TransitionPhase::Removed => {}
            TransitionPhase::Active => {
                let (trans_style, _) = generate_active_transition_style(&span_config, should_animate);
                if !trans_style.is_empty() {
                    styles.push(trans_style);
                }
            }
        }
        styles.join(" ")
    };

    let span_for_class = span.clone();
    let class_str = move || {
        let mut extra = Vec::new();
        match phase.get() {
            TransitionPhase::Pending => extra.push("delayed-hidden"),
            TransitionPhase::Removed => extra.push("fade-out-removed"),
            TransitionPhase::Active => {
                if should_animate {
                    extra.push("transition-active");
                } else {
                    extra.push("skip-animation");
                }
            }
        }
        span_classes(&span_for_class, &extra)
    };

    if let Some(action) = span.action {
        view! {
            <a
                href="javascript:void(0)"
                class=move || format!("passage-link {}", class_str())
                style=span_style
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
            <span class=move || format!("passage-link-static {}", class_str()) style=span_style>
                {span.content}
            </span>
        }
        .into_any()
    } else {
        view! {
            <span class=class_str style=span_style>
                {span.content}
            </span>
        }
        .into_any()
    }
}
