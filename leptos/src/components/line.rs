use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::span::SpanView;
use crate::context::StoryContext;
use crate::transition::{
    TransitionPhase, compute_initial_phase, generate_active_transition_style,
    generate_view_transition_style, parse_transition_classes, setup_transition_timers,
};

/// Renders a [`Line`] consisting of multiple spans, with line-level animation and class support.
#[component]
pub fn LineView(line: Line) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let config = parse_transition_classes(&line.classes);
    let is_changed = ctx
        .transitions
        .write_untracked()
        .is_content_changed(line.id, line.content_hash());
    let should_animate = config.has_transition() && is_changed;

    let (phase, set_phase) = signal(compute_initial_phase(&config, should_animate));
    if should_animate {
        setup_transition_timers(&config, set_phase);
    }

    let vt_style = generate_view_transition_style(line.id, is_changed);

    let line_config = config.clone();
    let line_style = move || {
        let mut styles = Vec::new();
        if !vt_style.is_empty() {
            styles.push(vt_style.clone());
        }
        match phase.get() {
            TransitionPhase::Pending | TransitionPhase::Removed => {}
            TransitionPhase::Active => {
                let (trans_style, _) =
                    generate_active_transition_style(&line_config, should_animate);
                if !trans_style.is_empty() {
                    styles.push(trans_style);
                }
            }
        }
        styles.join(" ")
    };

    let line_classes_base = line.classes.clone();
    let class_str = move || {
        let mut classes = line_classes_base.clone();
        classes.push("passage-line".into());
        match phase.get() {
            TransitionPhase::Pending => classes.push("delayed-hidden".into()),
            TransitionPhase::Removed => classes.push("fade-out-removed".into()),
            TransitionPhase::Active => {
                if should_animate {
                    classes.push("transition-active".into());
                } else {
                    classes.push("skip-animation".into());
                }
            }
        }
        classes.join(" ")
    };

    let spans = line.spans;

    view! {
        <span class=class_str style=line_style>
            {spans.into_iter().map(|span| {
                view! {
                    <SpanView span=span />
                }
            }).collect::<Vec<_>>()}
        </span>
    }
}
