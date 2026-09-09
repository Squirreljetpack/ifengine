use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::storage::{self, SaveSlot};

/// Indicates whether the modal is opened for saving or loading.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModalMode {
    Save,
    Load,
}

/// Unified modal component for managing save slots, autosave, and restarting a new game.
#[component]
pub fn SaveLoadModal<C>(
    /// Whether this modal is configured for Save or Load operations.
    mode: ModalMode,
    /// Callback invoked to close the modal.
    on_close: Callback<()>,
    /// Callback invoked when a saved game state is selected to load.
    on_load: Callback<ifengine::Game<C>>,
    /// Accessor to retrieve a clone of the active in-memory game state.
    get_current_game: Callback<(), ifengine::Game<C>>,
) -> impl IntoView
where
    C: ifengine::core::GameContext
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Send
        + Sync
        + 'static,
{
    let is_save = mode == ModalMode::Save;

    // Local signals for loaded slots and autosave
    let saves_signal = RwSignal::new(storage::load_saves::<C>());
    let autosave_signal = RwSignal::new(storage::load_autosave::<C>());

    let handle_create_new = move |_| {
        let game = get_current_game.run(());
        let new_slot = storage::create_save(&game, None);
        saves_signal.update(|slots| slots.push(new_slot));
    };

    view! {
        <div class="passage-popup-backdrop" on:click=move |_| on_close.run(())>
            <div class="passage-popup-dialog save-load-dialog" on:click=move |ev| ev.stop_propagation()>
                <div class="save-load-header">
                    <h3 class="save-load-title">{if is_save { "Save" } else { "Load" }}</h3>
                    <button type="button" class="save-load-close-btn" on:click=move |_| on_close.run(())>
                        "✕"
                    </button>
                </div>

                <div class="save-slots-list">
                    // --- First row: Autosave ---
                    {move || {
                        let autosave = autosave_signal.get();
                        let has_auto = autosave.is_some();
                        if !has_auto && !is_save {
                            return ().into_any();
                        }

                        let date_str = match &autosave {
                            Some(slot) => slot.date.clone(),
                            None => String::new(),
                        };

                        let autosave_for_load = autosave.clone();
                        let on_load_auto = move |_| {
                            if let Some(slot) = &autosave_for_load {
                                on_load.run(slot.game.clone());
                                on_close.run(());
                            }
                        };

                        view! {
                            <div class="save-slot-row">
                                <div class="save-slot-main">
                                    {if !is_save {
                                        view! {
                                            <button
                                                type="button"
                                                class="passage-link save-slot-link"
                                                on:click=on_load_auto
                                            >
                                                "Autosave"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="save-slot-title-text">"Autosave"</span>
                                        }.into_any()
                                    }}
                                    <div class="save-slot-meta">
                                         {if !date_str.is_empty() {
                                             view! { <span class="save-slot-date">{date_str}</span> }.into_any()
                                         } else {
                                             ().into_any()
                                         }}
                                     </div>
                                </div>
                            </div>
                        }.into_any()
                    }}

                    // --- Subsequent rows: Manual Saves ---
                    {move || {
                        let saves = saves_signal.get();
                        saves.into_iter().map(|slot: SaveSlot<C>| {
                                let id_for_rename = slot.id.clone();
                                let id_for_delete = slot.id.clone();
                                let id_for_overwrite = slot.id.clone();
                                let game_for_load = slot.game.clone();

                                let on_rename = move |ev: web_sys::Event| {
                                    if let Some(target) = ev.target()
                                        && let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>()
                                    {
                                        let new_name = input.value();
                                        storage::rename_save::<C>(&id_for_rename, &new_name);
                                    }
                                };

                                let on_delete = move |_| {
                                    storage::delete_save::<C>(&id_for_delete);
                                    let id_del = id_for_delete.clone();
                                    saves_signal.update(|slots| slots.retain(|s| s.id != id_del));
                                };

                                let on_overwrite = move |_| {
                                    let cur_game = get_current_game.run(());
                                    storage::overwrite_save::<C>(&id_for_overwrite, &cur_game);
                                    saves_signal.set(storage::load_saves::<C>());
                                };

                                let on_slot_load = move |_| {
                                    on_load.run(game_for_load.clone());
                                    on_close.run(());
                                };

                                view! {
                                    <div class="save-slot-row">
                                        <div class="save-slot-main">
                                            {if is_save {
                                                view! {
                                                    <input
                                                        type="text"
                                                        class="save-slot-input"
                                                        prop:value=slot.name
                                                        on:change=on_rename
                                                    />
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="passage-link save-slot-link"
                                                        on:click=on_slot_load
                                                    >
                                                        {slot.name}
                                                    </button>
                                                }.into_any()
                                            }}
                                            <div class="save-slot-meta">
                                                <span class="save-slot-date">{slot.date}</span>
                                            </div>
                                        </div>
                                        <div class="save-slot-actions">
                                            {if is_save {
                                                view! {
                                                    <button type="button" class="save-text-btn" on:click=on_overwrite>
                                                        "Overwrite"
                                                    </button>
                                                    <button type="button" class="save-text-btn" on:click=on_delete>
                                                        "Delete"
                                                    </button>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <button type="button" class="save-text-btn" on:click=on_delete>
                                                        "Delete"
                                                    </button>
                                                }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>().into_any()
                    }}
                </div>

                {if is_save {
                    view! {
                        <button type="button" class="passage-link save-create-link" on:click=handle_create_new>
                            "+ New Save"
                        </button>
                    }.into_any()
                } else {
                    ().into_any()
                }}
            </div>
        </div>
    }
}
