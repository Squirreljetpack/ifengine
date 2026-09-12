use ifengine::core::game_state::PageKey;
use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::line::LineView;
use crate::context::StoryContext;
use crate::transition::{generate_view_transition_style, is_line_delayed, line_in_delay};

/// Context provided to descendants within a choice item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveChoiceContext {
    pub key: PageKey,
    pub index: u8,
}

/// Renders a single choice item (button or inline container).
#[component]
fn ChoiceItemView(
    key: PageKey,
    choice_idx: u8,
    line: Line,
    has_internal_actions: bool,
) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let choice_ctx = Some(ActiveChoiceContext {
        key,
        index: choice_idx,
    });

    if has_internal_actions {
        view! {
            <div
                class="choice-item choice-item-inline"
                role="button"
                tabindex="0"
                on:click=move |e: leptos::ev::MouseEvent| {
                    e.prevent_default();
                    ctx.dispatch_choice.run((key, choice_idx));
                }
            >
                <LineView line=line choice=choice_ctx />
            </div>
        }
        .into_any()
    } else {
        view! {
            <button
                type="button"
                class="choice-item choice-button"
                on:click=move |e: leptos::ev::MouseEvent| {
                    e.prevent_default();
                    ctx.dispatch_choice.run((key, choice_idx));
                }
            >
                <LineView line=line choice=choice_ctx />
            </button>
        }
        .into_any()
    }
}

/// Renders an [`Object::Choice`](ifengine::view::Object::Choice) element containing
/// a list of selectable story paths or options.
#[component]
pub fn ChoiceView(key: PageKey, choices: Vec<(u8, Line)>) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();

    let is_changed = ctx
        .transitions
        .write_untracked()
        .is_content_changed(Some(key), 0);
    let vt_style = generate_view_transition_style(Some(key), is_changed);

    let is_fresh = ctx.transitions.read_untracked().is_fresh;
    let all_delayed = !choices.is_empty() && choices.iter().all(|(_, l)| is_line_delayed(l));
    let should_delay = all_delayed && !is_fresh && is_changed;
    let (is_pending, set_pending) = signal(should_delay);
    if should_delay {
        let min_delay = choices
            .iter()
            .map(|(_, l)| line_in_delay(l))
            .min()
            .unwrap_or(0);
        leptos::prelude::set_timeout(
            move || {
                set_pending.set(false);
            },
            std::time::Duration::from_millis(min_delay),
        );
    }

    let vt_clone = vt_style.clone();
    let choice_style = move || {
        if !is_pending.get() && !vt_clone.is_empty() {
            vt_clone.clone()
        } else {
            String::new()
        }
    };

    view! {
        <div class="passage-choices choice-container" style=choice_style>
            {choices.into_iter().map(|(idx, line)| {
                let has_internal_actions = line.spans.iter().any(|s| s.action.is_some());
                view! {
                    <ChoiceItemView
                        key=key
                        choice_idx=idx
                        line=line
                        has_internal_actions=has_internal_actions
                    />
                }
            }).collect::<Vec<_>>() }
        </div>
    }
}
