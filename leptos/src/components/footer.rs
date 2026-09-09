use leptos::prelude::*;

/// Story footer component rendering credits and engine attribution.
#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="passage-footer">
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
