use ifengine::PageKey;
use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::line::LineView;
use crate::context::StoryContext;
use crate::transition::{ItemKey, ItemKind, choice_item_parent_id, is_line_delayed, line_in_delay};

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
    #[prop(default = false)] is_changed: bool,
) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let choice_ctx = Some(ActiveChoiceContext {
        key,
        index: choice_idx,
    });

    let item_parent_id = choice_item_parent_id(key, choice_idx);

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
                <LineView line=line choice=choice_ctx is_changed=is_changed parent_id=Some(item_parent_id) />
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
                <LineView line=line choice=choice_ctx is_changed=is_changed parent_id=Some(item_parent_id) />
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

    let attrs = ctx.transitions.item_attrs(ItemKey {
        id: Some(key),
        hash: 0,
        changed: false,
        kind: ItemKind::Block,
        classes: &[],
        inherited_classes: None,
    });
    let is_changed = attrs.changed;
    let all_delayed = !choices.is_empty() && choices.iter().all(|(_, l)| is_line_delayed(l));

    let should_delay = all_delayed && is_changed;
    let min_delay = if should_delay {
        choices
            .iter()
            .map(|(_, l)| line_in_delay(l))
            .min()
            .unwrap_or(0)
    } else {
        0
    };

    let (is_pending, set_pending) = signal(should_delay);
    if should_delay {
        let transitions = ctx.transitions;
        ctx.transitions.schedule(min_delay, move || {
            transitions.mark_seen(Some(key), 0);
            set_pending.set(false);
        });
    }

    let render_choices = move || {
        if is_pending.get() {
            return None;
        }

        Some(view! {
            <div class="passage-choices choice-container">
                {choices.clone().into_iter().map(|(idx, line)| {
                    let has_internal_actions = line.spans.iter().any(|s| s.active());
                    view! {
                        <ChoiceItemView
                            key=key
                            choice_idx=idx
                            line=line
                            has_internal_actions=has_internal_actions
                            is_changed=is_changed
                        />
                    }
                }).collect::<Vec<_>>() }
            </div>
        })
    };

    view! {
        {render_choices}
    }
}
