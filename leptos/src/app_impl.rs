use std::sync::{Arc, RwLock};

use ifengine::{Actions, PageKey};
use leptos::prelude::*;

use crate::components::{
    Footer, Header, HeaderItem, ModalMode, ObjectView, OptionsModal, SaveLoadModal,
};
use crate::context::StoryContext;
use crate::transition::{
    PageTransitionPhase, TransitionHandle, TransitionRuntime, ensure_duration_styles,
    read_css_time_ms,
};

/// A raw header item as returned by the extractor: either plain text or an action with a game handler.
#[allow(dead_code)]
pub enum RawHeaderItem<C> {
    Text(String),
    Action {
        label: String,
        handler: fn(&mut ifengine::Game<C>),
    },
}

/// Type alias for a header extractor function returning three slots of raw header items.
pub type HeaderExtractor<C> = fn(&ifengine::Game<C>) -> [Vec<RawHeaderItem<C>>; 3];

/// Generic story application runner component.
///
/// Provides fine-grained reactive state handling, deterministic animation/transition tracking,
/// action/choice dispatchers via [`StoryContext`], and full DOM rendering for any `ifengine::Game<C>`.
#[component]
pub fn StoryApp<C>(
    /// Initial game instance.
    game: ifengine::Game<C>,
    /// Optional header extractor returning three slots (left / center / right) of items.
    #[prop(optional_no_strip)]
    header_extractor: Option<HeaderExtractor<C>>,
) -> impl IntoView
where
    C: ifengine::core::GameContext
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Send
        + Sync
        + 'static,
{
    // Timings are owned by styles.css; read them back so the DOM swap is scheduled
    // in sync with the CSS animations.
    let page_transition_out_ms = read_css_time_ms("--page-transition-out-duration", 600);
    let page_transition_in_ms = read_css_time_ms("--page-transition-in-duration", 400);

    let game_lock = Arc::new(RwLock::new(game));

    // Check if initial URL hash targets a registered page:
    if let Some(hash) = crate::dev::get_url_hash()
        && let Some(handle) = crate::dev::find_registered_page(&hash)
    {
        let mut g = game_lock.write().unwrap();
        let _ = g.handle_action(ifengine::Action::Next(handle));
    }

    let initial_view = game_lock
        .write()
        .unwrap()
        .view()
        .expect("failed to evaluate initial story page");

    let transitions = TransitionHandle::new(TransitionRuntime::detect(initial_view.pageid.clone()));
    ensure_duration_styles(&transitions.build_plan(&initial_view));

    let dispatch_cell: Arc<RwLock<Option<Arc<dyn Fn(fn(&mut ifengine::Game<C>)) + Send + Sync>>>> =
        Arc::new(RwLock::new(None));

    let dispatch_action_fn = {
        let dispatch_cell = Arc::clone(&dispatch_cell);
        Arc::new(move |handler: fn(&mut ifengine::Game<C>)| {
            if let Some(dispatch) = dispatch_cell.read().unwrap().as_ref() {
                dispatch(handler);
            }
        }) as Arc<dyn Fn(fn(&mut ifengine::Game<C>)) + Send + Sync>
    };

    let initial_slots = header_extractor.map_or_else(
        || [vec![], vec![], vec![]],
        |f| {
            resolve_slots(
                f(&game_lock.read().unwrap()),
                Arc::clone(&dispatch_action_fn),
            )
        },
    );

    let (current_view, set_current_view) = signal(initial_view);
    let (header_slots, set_header_slots) = signal(initial_slots);
    let (transition_phase, set_transition_phase) = signal(PageTransitionPhase::None);
    let (active_modal, set_active_modal) = signal(Option::<ModalMode>::None);
    let (options_modal_open, set_options_modal_open) = signal(false);
    transitions.finish_page_transition();

    // execute the fade-out -> swap -> fade-in transition
    let trigger_page_transition = move |new_view: ifengine::View, slots: [Vec<HeaderItem>; 3]| {
        transitions.reset();
        set_transition_phase.set(PageTransitionPhase::Leaving);

        set_timeout(
            move || {
                set_header_slots.set(slots);
                set_current_view.set(new_view);

                if let Some(window) = web_sys::window() {
                    window.scroll_to_with_x_and_y(0.0, 0.0);
                }

                set_transition_phase.set(PageTransitionPhase::Entering);

                set_timeout(
                    move || {
                        set_transition_phase.set(PageTransitionPhase::None);
                        transitions.finish_page_transition();
                    },
                    std::time::Duration::from_millis(page_transition_in_ms),
                );
            },
            std::time::Duration::from_millis(page_transition_out_ms),
        );
    };

    let refresh_view: Arc<dyn Fn(&mut ifengine::Game<C>) + Send + Sync> = Arc::new({
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

            let slots = header_extractor.map_or_else(
                || [vec![], vec![], vec![]],
                |f| resolve_slots(f(game), Arc::clone(&dispatch_action_fn)),
            );

            transitions.on_view_change(&new_view.pageid);
            let plan = transitions.build_plan(&new_view);
            ensure_duration_styles(&plan);

            if game.fresh() {
                trigger_page_transition(new_view, slots);
            } else {
                let cur_has_modal = has_modal(&current_view.get_untracked());
                let new_has_modal = has_modal(&new_view);

                if cur_has_modal || new_has_modal {
                    set_header_slots.set(slots);
                    set_current_view.set(new_view);
                } else {
                    transitions.commit(plan, move || {
                        set_header_slots.set(slots);
                        set_current_view.set(new_view);
                    });
                }
            }
        }
    });

    crate::dev::mount_dev_bridge(Arc::clone(&game_lock), Arc::clone(&refresh_view));

    // --- Interaction Dispatcher (Choice and/or Action) ---
    let game_for_interaction = Arc::clone(&game_lock);
    let on_interaction = refresh_view.clone();
    let dispatch_interaction = Callback::new(
        move |(choice, action): (Option<(PageKey, u8)>, Option<Actions>)| {
            let mut game = game_for_interaction.write().unwrap();
            crate::storage::save_autosave(&game);
            if let Some((choice_key, index)) = choice {
                game.handle_choice(choice_key, index);
            }
            if let Some(action) = action
                && let Err(err) = game.handle_action(action)
            {
                web_sys::console::error_1(
                    &format!("[ifengine_leptos] Error executing action: {err:?}").into(),
                );
                return;
            }
            on_interaction(&mut game);
        },
    );

    let dispatch_action = Callback::new(move |action: Actions| {
        dispatch_interaction.run((None, Some(action)));
    });

    let dispatch_choice = Callback::new(move |(choice_key, index): (PageKey, u8)| {
        dispatch_interaction.run((Some((choice_key, index)), None));
    });

    let dispatch_choice_action = Callback::new(
        move |((choice_key, index), action): ((PageKey, u8), Actions)| {
            dispatch_interaction.run((Some((choice_key, index)), Some(action)));
        },
    );

    // --- Modal Controls & Game Reload Handlers ---
    let open_save_modal = Callback::new(move |_| {
        set_active_modal.set(Some(ModalMode::Save));
    });

    let open_load_modal = Callback::new(move |_| {
        set_active_modal.set(Some(ModalMode::Load));
    });

    let close_modal = Callback::new(move |_| {
        set_active_modal.set(None);
    });

    let open_options_modal = Callback::new(move |_| {
        set_options_modal_open.set(true);
    });

    let close_options_modal = Callback::new(move |_| {
        set_options_modal_open.set(false);
    });

    let game_for_header = Arc::clone(&game_lock);
    let on_refresh_header = refresh_view.clone();
    let dispatch_header_action: Arc<dyn Fn(fn(&mut ifengine::Game<C>)) + Send + Sync> =
        Arc::new(move |handler: fn(&mut ifengine::Game<C>)| {
            let mut game = game_for_header.write().unwrap();
            // these actions don't need autosaving
            // crate::storage::save_autosave(&game);
            handler(&mut game);
            on_refresh_header(&mut game);
        });
    *dispatch_cell.write().unwrap() = Some(dispatch_header_action);

    let game_for_load = Arc::clone(&game_lock);
    let on_refresh_load = refresh_view.clone();
    let on_load_game = Callback::new(move |loaded_game: ifengine::Game<C>| {
        let mut game = game_for_load.write().unwrap();
        *game = loaded_game;
        on_refresh_load(&mut game);
    });

    let game_for_get = Arc::clone(&game_lock);
    let get_current_game = Callback::new(move |_| game_for_get.read().unwrap().clone());

    provide_context(StoryContext {
        dispatch_action,
        dispatch_choice,
        dispatch_choice_action,
        transitions,
        open_save_modal,
        open_load_modal,
        open_options_modal,
    });

    let article_class = move || match transition_phase.get() {
        PageTransitionPhase::None => "passage-article",
        PageTransitionPhase::Leaving => "passage-article page-fade-out",
        PageTransitionPhase::Entering => "passage-article page-fade-in",
    };

    view! {
        <div id="backdrop">
            <main id="page" aria-live="polite">
                <Header
                    slots=header_slots
                />

                <article class=article_class>
                    {move || {
                        // Items must re-render when the reader switches motion backend:
                        // their inline transition attributes depend on it.
                        let _ = transitions.selected_backend();
                        let view = current_view.get();
                        view.inner.into_iter().map(|stamped| {
                            view! { <ObjectView stamped=stamped /> }
                        }).collect::<Vec<_>>()
                    }}
                </article>

                <Footer />
            </main>

            {move || {
                active_modal.get().map(|mode| {
                    view! {
                        <SaveLoadModal
                            mode=mode
                            on_close=close_modal
                            on_load=on_load_game
                            get_current_game=get_current_game
                        />
                    }
                })
            }}

            {move || {
                options_modal_open.get().then(|| {
                    view! { <OptionsModal on_close=close_options_modal /> }
                })
            }}
        </div>
    }
}

/// Recursively checks whether a page view contains an active popup or modal element.
fn has_modal(view: &ifengine::View) -> bool {
    view.inner.iter().any(stamped_has_modal)
}

fn stamped_has_modal(stamped: &ifengine::view::StampedObject) -> bool {
    match &stamped.object {
        ifengine::view::Object::Paragraph(_, rd) => *rd == "popup" || *rd == "modal",
        ifengine::view::Object::Embed(embedded, rd) => {
            *rd == "popup" || *rd == "modal" || has_modal(embedded)
        }
        _ => false,
    }
}

/// Converts raw extractor output into resolved [`HeaderItem`] slots by wrapping each action
/// handler in an `Arc` closure that dispatches through the shared game action dispatcher.
fn resolve_slots<C: 'static>(
    raw: [Vec<RawHeaderItem<C>>; 3],
    dispatch: Arc<dyn Fn(fn(&mut ifengine::Game<C>)) + Send + Sync>,
) -> [Vec<HeaderItem>; 3] {
    raw.map(|slot| {
        slot.into_iter()
            .map(|item| match item {
                RawHeaderItem::Text(t) => HeaderItem::Text(t),
                RawHeaderItem::Action { label, handler } => {
                    let dispatch = Arc::clone(&dispatch);
                    HeaderItem::Action {
                        label,
                        callback: Arc::new(move || dispatch(handler)),
                    }
                }
            })
            .collect()
    })
}
