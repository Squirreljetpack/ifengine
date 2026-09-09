use ifengine::core::{PageId, game_state::PageKey};
use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::line::LineView;
use crate::context::StoryContext;
use crate::transition::{
    compute_initial_phase, extract_line_transition, generate_active_transition_style,
    generate_view_transition_style, setup_transition_timers, strip_transitions_from_line,
    TransitionPhase,
};

/// Renders a single choice item (button or inline container) with transition lifecycle handling.
#[component]
fn ChoiceItemView(
    key: PageKey,
    choice_idx: u8,
    line: Line,
    page_id: PageId,
    has_internal_actions: bool,
) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let config = extract_line_transition(&line);
    let is_changed = ctx
        .transitions
        .write_untracked()
        .is_content_changed(&page_id, line.id.or(Some(key)), line.content_hash());
    let should_animate = config.has_transition() && is_changed;

    let (phase, set_phase) = signal(compute_initial_phase(&config, should_animate));

    if should_animate {
        setup_transition_timers(&config, set_phase);
    }

    let item_config = config.clone();
    let item_style = move || match phase.get() {
        TransitionPhase::Pending | TransitionPhase::Removed => {
            "display: none !important; margin: 0 !important; padding: 0 !important;".to_string()
        }
        TransitionPhase::Active => {
            let (style, _) = generate_active_transition_style(&item_config, should_animate);
            style
        }
    };

    let item_class = move || match phase.get() {
        TransitionPhase::Pending => "delayed-hidden",
        TransitionPhase::Removed => "fade-out-removed",
        TransitionPhase::Active => {
            if should_animate {
                "transition-active"
            } else {
                "skip-animation"
            }
        }
    };

    let mut inner_line = line;
    if config.has_transition() {
        inner_line = strip_transitions_from_line(inner_line);
    }

    let page_id_clone = page_id.clone();

    if has_internal_actions {
        view! {
            <div
                class=move || format!("choice-item choice-item-inline {}", item_class())
                style=item_style
            >
                <LineView
                    line=inner_line
                    page_id=page_id_clone
                />
            </div>
        }
        .into_any()
    } else {
        view! {
            <button
                type="button"
                class=move || format!("choice-item choice-button {}", item_class())
                style=item_style
                on:click=move |e: leptos::ev::MouseEvent| {
                    e.prevent_default();
                    ctx.dispatch_choice.run((key, choice_idx));
                }
            >
                <LineView
                    line=inner_line
                    page_id=page_id_clone
                />
            </button>
        }
        .into_any()
    }
}

/// Renders an [`Object::Choice`](ifengine::view::Object::Choice) element containing
/// a list of selectable story paths or options.
#[component]
pub fn ChoiceView(key: PageKey, choices: Vec<(u8, Line)>, page_id: PageId) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();

    let is_changed = ctx
        .transitions
        .write_untracked()
        .is_content_changed(&page_id, Some(key), 0);
    let vt_style = generate_view_transition_style(Some(key), is_changed);

    // If all choices have entrance delays (e.g. in-500), hide the choice container during the minimum delay
    let has_immediate = choices.is_empty()
        || choices.iter().any(|(_, line)| {
            let config = extract_line_transition(line);
            config.in_delay.unwrap_or(0) == 0
        });

    let min_delay = if has_immediate {
        0
    } else {
        choices
            .iter()
            .map(|(_, line)| extract_line_transition(line).in_delay.unwrap_or(0))
            .min()
            .unwrap_or(0)
    };

    let (container_active, set_container_active) = signal(min_delay == 0);
    if min_delay > 0 {
        set_timeout(
            move || set_container_active.set(true),
            std::time::Duration::from_millis(min_delay),
        );
    }

    let container_style = move || {
        let mut parts = Vec::new();
        if !vt_style.is_empty() {
            parts.push(vt_style.clone());
        }
        if !container_active.get() {
            parts.push("display: none !important; margin: 0 !important;".to_string());
        }
        parts.join(" ")
    };

    view! {
        <div class="passage-choices choice-container" style=container_style>
            {choices.into_iter().map(|(idx, line)| {
                let has_internal_actions = line.spans.iter().any(|s| s.action.is_some());
                view! {
                    <ChoiceItemView
                        key=key
                        choice_idx=idx
                        line=line
                        page_id=page_id.clone()
                        has_internal_actions=has_internal_actions
                    />
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
