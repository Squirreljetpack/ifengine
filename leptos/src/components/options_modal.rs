use leptos::prelude::*;

use crate::context::StoryContext;
use crate::transition::TransitionBackend;

/// Motion choices offered by the options modal, in display order.
const MOTION_OPTIONS: [(TransitionBackend, &str); 3] = [
    (TransitionBackend::ViewTransition, "View Transitions"),
    (TransitionBackend::Css, "CSS clone"),
    (TransitionBackend::Instant, "Instant"),
];

/// Reader options modal, currently holding the motion backend setting.
#[component]
pub fn OptionsModal(
    /// Callback invoked to close the modal.
    on_close: Callback<()>,
) -> impl IntoView {
    let ctx = use_context::<StoryContext>();

    let choose = move |backend: TransitionBackend| {
        if let Some(ctx) = ctx {
            ctx.transitions.set_backend(backend);
        }
    };

    let motion_rows = MOTION_OPTIONS
        .iter()
        .map(|&(backend, title)| {
            let selected =
                move || ctx.is_some_and(|ctx| ctx.transitions.selected_backend() == backend);
            view! {
                <label class=move || {
                    if selected() { "options-row options-row-selected" } else { "options-row" }
                }>
                    <input
                        type="radio"
                        class="options-radio"
                        name="motion"
                        prop:checked=selected
                        on:change=move |_| choose(backend)
                    />
                    <span class="options-row-title">{title}</span>
                </label>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="passage-popup-backdrop" on:click=move |_| on_close.run(())>
            <div
                class="passage-popup-dialog options-dialog"
                role="dialog"
                aria-modal="true"
                aria-label="Options"
                on:click=move |ev| ev.stop_propagation()
            >
                <div class="save-load-header">
                    <h3 class="save-load-title">"Options"</h3>
                    <button type="button" class="save-load-close-btn" on:click=move |_| on_close.run(())>
                        "✕"
                    </button>
                </div>

                <div class="options-section">
                    <h4 class="options-section-title">"Motion"</h4>
                    <p class="options-section-hint">
                        "How content updates animate within a page."
                    </p>
                    <div class="options-list">{motion_rows}</div>
                </div>
            </div>
        </div>
    }
}
