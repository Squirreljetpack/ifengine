use leptos::prelude::*;

use crate::context::StoryContext;

/// Story footer component rendering Save/Load buttons (left-aligned) and engine attribution.
#[component]
pub fn Footer() -> impl IntoView {
    let ctx = use_context::<StoryContext>();

    let on_save = move |_| {
        if let Some(ctx) = ctx {
            ctx.open_save_modal.run(());
        }
    };

    let on_load = move |_| {
        if let Some(ctx) = ctx {
            ctx.open_load_modal.run(());
        }
    };

    view! {
        <footer class="passage-footer">
            <div class="footer-left">
                <button type="button" class="footer-btn" on:click=on_save>
                    "Save"
                </button>
                <button type="button" class="footer-btn" on:click=on_load>
                    "Load"
                </button>
            </div>
            <a
                href="https://github.com/Squirreljetpack/ifengine"
                target="_blank"
                rel="noopener noreferrer"
                class="footer-attribution-link"
            >
                "Made with IfEngine"
            </a>
        </footer>
    }
}
