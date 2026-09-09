use leptos::prelude::*;

/// Story header component rendering game status metadata (e.g. Day, Miles, Rations).
#[component]
pub fn Header(items: ReadSignal<Vec<String>>) -> impl IntoView {
    view! {
        {move || {
            let list = items.get();
            if list.is_empty() {
                None
            } else {
                let left = list.first().cloned().unwrap_or_default();
                let center = list.get(1).cloned().unwrap_or_default();
                let right = list.get(2).cloned().unwrap_or_default();

                Some(view! {
                    <header class="passage-header header-has-content">
                        <div class="header-section header-left">{left}</div>
                        <div class="header-section header-center">{center}</div>
                        <div class="header-section header-right">{right}</div>
                    </header>
                })
            }
        }}
    }
}
