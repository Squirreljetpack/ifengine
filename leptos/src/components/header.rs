use std::sync::Arc;

use leptos::prelude::*;

/// A single rendered item in a header slot: either plain text or a clickable link.
#[derive(Clone)]
pub enum HeaderItem {
    Text(String),
    Action {
        label: String,
        callback: Arc<dyn Fn() + Send + Sync>,
    },
}

/// Story header component rendering three slots (left / center / right), each accepting
/// a mix of plain text and action links.
#[component]
pub fn Header(slots: ReadSignal<[Vec<HeaderItem>; 3]>) -> impl IntoView {
    view! {
        {move || {
            let [left, center, right] = slots.get();
            let all_empty = left.is_empty() && center.is_empty() && right.is_empty();

            if all_empty {
                None
            } else {
                Some(view! {
                    <header class="passage-header header-has-content">
                        <div class="header-section header-left">{render_slot(left)}</div>
                        <div class="header-section header-center">{render_slot(center)}</div>
                        <div class="header-section header-right">{render_slot(right)}</div>
                    </header>
                })
            }
        }}
    }
}

fn render_slot(items: Vec<HeaderItem>) -> impl IntoView {
    items
        .into_iter()
        .map(|item| match item {
            HeaderItem::Text(t) => view! { <span class="header-text">{t}</span> }.into_any(),
            HeaderItem::Action { label, callback } => view! {
                <a
                    class="header-action-link"
                    href="#"
                    on:click=move |e| {
                        e.prevent_default();
                        callback();
                    }
                >
                    {label}
                </a>
            }
            .into_any(),
        })
        .collect::<Vec<_>>()
}
