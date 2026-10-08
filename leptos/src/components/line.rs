use ifengine::PageKey;
use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::choice::ActiveChoiceContext;
use crate::components::span::SpanView;
use crate::consts::AUTO_SPAN_TRANSITIONS;
use crate::transition::{
    TransitionPhase, compute_initial_phase, generate_active_transition_style,
    parse_transition_classes, setup_transition_timers,
};

/// Renders a [`Line`] consisting of multiple spans, with line-level animation and class support.
#[component]
pub fn LineView(
    line: Line,
    #[prop(optional_no_strip)] choice: Option<ActiveChoiceContext>,
    #[prop(default = false)] is_changed: bool,
    #[prop(optional_no_strip)] parent_id: Option<PageKey>,
) -> impl IntoView {
    let config = parse_transition_classes(&line.classes);
    let should_animate = config.has_transition() && is_changed;

    let (phase, set_phase) = signal(compute_initial_phase(&config, should_animate));
    if should_animate {
        setup_transition_timers(&config, set_phase);
    }

    let line_config = config.clone();
    let line_style = move || {
        let mut styles = Vec::new();
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

    let ctx = expect_context::<crate::context::StoryContext>();
    let parent = parent_id.unwrap_or(0);
    let mut chunks: Vec<(Option<PageKey>, Vec<ifengine::view::Span>)> = Vec::new();

    for (idx, mut span) in line.spans.into_iter().enumerate() {
        if span.id.is_none() && AUTO_SPAN_TRANSITIONS && !span.content.trim().is_empty() {
            let auto_key = (parent.rotate_left(7) ^ ((idx as u64) + 1)) | (1u64 << 63);
            span.id = Some(auto_key);
        }

        let id = span.id;
        if let Some(key) = id {
            if let Some((Some(last_key), spans)) = chunks.last_mut() {
                if *last_key == key {
                    spans.push(span);
                    continue;
                }
            }
            chunks.push((Some(key), vec![span]));
        } else {
            chunks.push((None, vec![span]));
        }
    }

    let mut seen_ids: std::collections::HashMap<PageKey, usize> = std::collections::HashMap::new();

    view! {
        <span class=class_str style=line_style>
            {chunks.into_iter().map(|(maybe_id, group_spans)| {
                if let Some(id) = maybe_id {
                    let occurrence = seen_ids.entry(id).or_insert(0);
                    let effective_id = if *occurrence == 0 {
                        id
                    } else {
                        (id.rotate_left(13) ^ (*occurrence as u64)) | (1u64 << 63)
                    };
                    *occurrence += 1;

                    let mut hasher = std::hash::DefaultHasher::new();
                    for s in &group_spans {
                        use std::hash::Hash;
                        s.content.hash(&mut hasher);
                    }
                    use std::hash::Hasher;
                    let group_hash = match hasher.finish() {
                        0 => 1,
                        h => h,
                    };
                    let is_changed = ctx
                        .transitions
                        .write_untracked()
                        .is_content_changed(Some(effective_id), group_hash);
                    let vt_style = crate::transition::generate_view_transition_style(Some(effective_id), is_changed);

                    view! {
                        <span class="passage-inline-group" style=vt_style>
                            {group_spans.into_iter().map(|s| view! {
                                <SpanView span=s choice=choice in_group=true is_changed=is_changed />
                            }).collect::<Vec<_>>()}
                        </span>
                    }
                    .into_any()
                } else {
                    view! {
                        {group_spans.into_iter().map(|s| view! {
                            <SpanView span=s choice=choice in_group=false />
                        }).collect::<Vec<_>>()}
                    }
                    .into_any()
                }
            }).collect::<Vec<_>>()}
        </span>
    }
}
