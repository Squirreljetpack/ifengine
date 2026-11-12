use ifengine::PageKey;
use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::choice::ActiveChoiceContext;
use crate::components::span::SpanView;
use crate::consts::AUTO_SPAN_TRANSITIONS;
use crate::context::StoryContext;
use crate::transition::{
    TransitionPhase, compute_initial_phase, parse_transition_classes, setup_transition_timers,
};

/// Renders a [`Line`] consisting of multiple spans, with line-level animation and class support.
#[component]
pub fn LineView(
    line: Line,
    #[prop(optional_no_strip)] choice: Option<ActiveChoiceContext>,
    #[prop(default = false)] is_changed: bool,
    #[prop(optional_no_strip)] parent_id: Option<PageKey>,
    #[prop(default = 0)] waited_delay: u64,
) -> impl IntoView {
    let mut config = parse_transition_classes(&line.classes);
    if waited_delay > 0
        && let Some(d) = config.in_delay
    {
        config.in_delay = Some(d.saturating_sub(waited_delay));
    }
    let should_animate = config.has_transition() && is_changed;

    let ctx = expect_context::<StoryContext>();

    let (phase, set_phase) = signal(compute_initial_phase(&config, should_animate));
    if should_animate {
        setup_transition_timers(&config, &ctx.transitions, set_phase);
    }

    let line_classes_base = line.classes.clone();
    let class_str = move || {
        let mut classes = line_classes_base.clone();
        classes.push("passage-line".into());
        match phase.get() {
            TransitionPhase::Pending => classes.push("delayed-hidden".into()),
            TransitionPhase::Removing => classes.push("fade-out-removed".into()),
            TransitionPhase::Removed => classes.push("transition-removed".into()),
            _ => {}
        }
        classes.join(" ")
    };

    let out_fade_style = config
        .fade_out
        .map(|ms| format!("--vt-fade-out: {ms}ms;"))
        .unwrap_or_default();
    let line_style = move || {
        if matches!(phase.get(), TransitionPhase::Removing) {
            out_fade_style.clone()
        } else {
            String::new()
        }
    };

    let parent = parent_id.unwrap_or(0);
    let mut spans = line.spans.clone();
    if AUTO_SPAN_TRANSITIONS {
        assign_auto_keys(&mut spans, parent);
    }

    let line_cfg = if config.has_transition() {
        Some(config.clone())
    } else {
        None
    };
    let line_classes = line.classes.clone();
    let rendered_spans = spans
        .into_iter()
        .filter(|s| !s.content.is_empty() || s.active())
        .map(|span| {
            view! {
                <SpanView
                    span=span
                    choice=choice
                    inherited_config=line_cfg.clone()
                    inherited_classes=Some(line_classes.clone())
                    waited_delay=waited_delay
                />
            }
            .into_any()
        })
        .collect::<Vec<_>>();

    view! {
        <span class=class_str style=line_style>
            {rendered_spans}
        </span>
    }
}

pub(crate) fn assign_auto_keys(spans: &mut [ifengine::view::Span], parent: PageKey) {
    let mut seen: std::collections::HashSet<PageKey> = std::collections::HashSet::new();
    let mut slot = 1u64;
    for span in spans.iter_mut() {
        if span.content.is_empty() && !span.active() {
            continue;
        }
        match span.id {
            Some(id) if seen.insert(id) => {
                // First time seeing this explicit ID; keep it.
            }
            Some(_) => {
                // Duplicate ID in the same line: give it a distinct auto key.
                loop {
                    let auto_key =
                        (parent.rotate_left(7) ^ slot) | ifengine::core::key::HASH_KEY_BIT;
                    slot = slot.saturating_add(1);
                    if seen.insert(auto_key) {
                        span.id = Some(auto_key);
                        break;
                    }
                }
                continue;
            }
            None => {
                if !span.content.trim().is_empty() {
                    loop {
                        let auto_key =
                            (parent.rotate_left(7) ^ slot) | ifengine::core::key::HASH_KEY_BIT;
                        slot = slot.saturating_add(1);
                        if seen.insert(auto_key) {
                            span.id = Some(auto_key);
                            break;
                        }
                    }
                    continue;
                }
            }
        }
        slot = slot.saturating_add(1);
    }
}
