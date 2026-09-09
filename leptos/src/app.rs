use std::sync::{Arc, RwLock};

use ifengine::core::{Action, game_state::PageKey};
use leptos::prelude::*;

use crate::components::{Footer, Header, ObjectView};
use crate::context::StoryContext;
use crate::render::extract_header;
use crate::transition::TransitionManager;

/// Current phase of a full-page transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageTransitionPhase {
    #[default]
    None,
    Leaving,
    Entering,
}

/// Type alias for a thread-safe header extractor closure.
pub type HeaderExtractor<C> =
    Arc<dyn Fn(&ifengine::Game<C>) -> Vec<String> + Send + Sync + 'static>;

/// Generic story application runner component.
///
/// Provides fine-grained reactive state handling, deterministic animation/transition tracking,
/// action/choice dispatchers via [`StoryContext`], and full DOM rendering for any `ifengine::Game<C>`.
#[component]
pub fn StoryApp<C>(
    /// Initial game instance.
    game: ifengine::Game<C>,
    /// Optional header extractor function returning header items (left, center, right).
    #[prop(optional)]
    header_extractor: Option<HeaderExtractor<C>>,
) -> impl IntoView
where
    C: ifengine::core::GameContext + Send + Sync + 'static,
{
    let game_lock = Arc::new(RwLock::new(game));

    let initial_view = game_lock
        .write()
        .unwrap()
        .view()
        .expect("failed to evaluate initial story page");

    let initial_header = header_extractor
        .as_ref()
        .map_or_else(Vec::new, |f| f(&game_lock.read().unwrap()));

    let (current_view, set_current_view) = signal(initial_view);
    let (header_items, set_header_items) = signal(initial_header);
    let (transition_phase, set_transition_phase) = signal(PageTransitionPhase::None);
    let transitions = RwSignal::new(TransitionManager::new());

    transitions
        .write_untracked()
        .on_view_change(&current_view.get_untracked().pageid, true);

    // execute the fade-out -> swap -> fade-in transition
    let trigger_page_transition = move |new_view: ifengine::View, header: Vec<String>| {
        set_transition_phase.set(PageTransitionPhase::Leaving);

        set_timeout(
            move || {
                transitions
                    .write_untracked()
                    .on_view_change(&new_view.pageid, true);

                set_header_items.set(header);
                set_current_view.set(new_view);

                if let Some(window) = web_sys::window() {
                    window.scroll_to_with_x_and_y(0.0, 0.0);
                }

                set_transition_phase.set(PageTransitionPhase::Entering);

                set_timeout(
                    move || set_transition_phase.set(PageTransitionPhase::None),
                    std::time::Duration::from_millis(crate::consts::DEFAULT_PAGE_TRANSITION_IN_MS),
                );
            },
            std::time::Duration::from_millis(crate::consts::DEFAULT_PAGE_TRANSITION_OUT_MS),
        );
    };

    let refresh_view = {
        let header_extractor = header_extractor.clone();
        move |game: &mut ifengine::Game<C>| {
            let new_view = match game.view() {
                Ok(view) => view,
                Err(err) => {
                    web_sys::console::error_1(
                        &format!("[ifengine_leptos] Error rendering page view: {err:?}").into(),
                    );
                    return;
                }
            };

            let header = header_extractor.as_ref().map_or_else(Vec::new, |f| f(game));

            if game.fresh() {
                trigger_page_transition(new_view, header);
            } else {
                transitions
                    .write_untracked()
                    .on_view_change(&new_view.pageid, false);

                run_with_view_transition(move || {
                    set_header_items.set(header);
                    set_current_view.set(new_view);
                });
            }
        }
    };

    // --- Action Dispatcher ---
    let game_for_action = Arc::clone(&game_lock);
    let on_action = refresh_view.clone();
    let dispatch_action = Callback::new(move |action: Action| {
        let mut game = game_for_action.write().unwrap();
        if let Err(err) = game.handle_action(action) {
            web_sys::console::error_1(
                &format!("[ifengine_leptos] Error executing action: {err:?}").into(),
            );
            return;
        }
        on_action(&mut game);
    });

    // --- Choice Dispatcher ---
    let game_for_choice = Arc::clone(&game_lock);
    let on_choice = refresh_view;
    let dispatch_choice = Callback::new(move |(choice_key, index): (PageKey, u8)| {
        let mut game = game_for_choice.write().unwrap();
        game.handle_choice(choice_key, index);
        on_choice(&mut game);
    });

    provide_context(StoryContext {
        dispatch_action,
        dispatch_choice,
        transitions,
    });

    let article_class = move || match transition_phase.get() {
        PageTransitionPhase::None => "passage-article",
        PageTransitionPhase::Leaving => "passage-article page-fade-out",
        PageTransitionPhase::Entering => "passage-article page-fade-in",
    };

    view! {
        <div id="backdrop">
            <main id="page" aria-live="polite">
                <Header items=header_items />

                <article class=article_class>
                    {move || {
                        let view = current_view.get();
                        view.inner.into_iter().map(|object| {
                            view! { <ObjectView object=object page_id=view.pageid.clone() /> }
                        }).collect::<Vec<_>>()
                    }}
                </article>

                <Footer />
            </main>
        </div>
    }
}

/// Main Leptos application component for `ifengine`.
///
/// Automatically mounts the active story configured by feature flags (`saltwrack` or `test`),
/// extracting header metadata via [`extract_header`].
#[component]
pub fn App() -> impl IntoView {
    let initial_game = story::new();

    view! {
        <StoryApp
            game=initial_game
            header_extractor=Arc::new(extract_header)
        />
    }
}

/// Executes a DOM state update within `document.startViewTransition` if supported by the browser.
///
/// Used exclusively for same-page interactions to smoothly transition elements sharing the same ID (e.g. `alts!`),
/// while page transitions use the immediate DOM swap model with element-level CSS animations.
pub fn run_with_view_transition<F: FnOnce() + 'static>(update: F) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let doc_val = wasm_bindgen::JsValue::from(&document);
            if let Ok(method) = js_sys::Reflect::get(
                &doc_val,
                &wasm_bindgen::JsValue::from_str("startViewTransition"),
            ) {
                if method.is_function() {
                    let function = js_sys::Function::from(method);
                    let closure = wasm_bindgen::closure::Closure::once_into_js(move || {
                        update();
                    });
                    let _ = function.call1(&doc_val, &closure);
                    return;
                }
            }
        }
    }

    // Direct update when View Transitions API is unavailable
    update();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_resolution() {
        let mut game = story::new();
        let view = game.view().expect("should render initial view");

        #[cfg(all(feature = "saltwrack", not(feature = "test")))]
        {
            assert!(
                view.pageid.0.contains("saltwrack"),
                "expected saltwrack story, got: {}",
                view.pageid.0
            );
            let header = extract_header(&game);
            assert!(header.is_empty());
        }

        #[cfg(any(feature = "test", not(feature = "saltwrack")))]
        {
            assert!(
                view.pageid.0.contains("rainy_day"),
                "expected test_story (rainy_day), got: {}",
                view.pageid.0
            );
            let header = extract_header(&game);
            assert!(header.is_empty());
        }
    }

    #[test]
    fn test_modular_custom_game() {
        let mut game: ifengine::Game<()> = ifengine::Game::new_with_page("test_page", |_game| {
            let mut view = ifengine::view::View::new(ifengine::core::PageId("test_page".into()));
            view.inner.push(ifengine::view::Object::Paragraph(
                ifengine::view::Line::from("hello"),
            ));
            ifengine::core::Response::View(view)
        });
        let view = game.view().expect("custom game should render view");
        assert_eq!(view.pageid.0.as_ref(), "test_page");
        assert_eq!(view.inner.len(), 1);
    }
}
