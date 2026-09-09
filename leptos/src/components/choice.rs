use ifengine::core::{PageId, game_state::PageKey};
use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::line::LineView;
use crate::context::StoryContext;
use crate::transition::generate_view_transition_style;

/// Renders a single choice item (button or inline container).
#[component]
fn ChoiceItemView(
    key: PageKey,
    choice_idx: u8,
    line: Line,
    page_id: PageId,
    has_internal_actions: bool,
) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let page_id_clone = page_id.clone();

    if has_internal_actions {
        view! {
            <div class="choice-item choice-item-inline">
                <LineView
                    line=line
                    page_id=page_id_clone
                />
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
                <LineView
                    line=line
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

    view! {
        <div class="passage-choices choice-container" style=vt_style>
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
