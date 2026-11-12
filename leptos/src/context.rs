use ifengine::{Actions, PageKey};
use leptos::prelude::*;

use crate::transition::TransitionHandle;

/// Unified reactive context provided at the application root.
///
/// Eliminates prop drilling by delivering action/choice dispatchers
/// and the transition state directly to leaf components (`SpanView`, `ChoiceView`).
#[derive(Clone, Copy)]
pub struct StoryContext {
    /// Dispatches state mutations and passage transitions.
    pub dispatch_action: Callback<Actions>,
    /// Dispatches bitmask choice selections.
    pub dispatch_choice: Callback<(PageKey, u8)>,
    /// Dispatches a choice selection and action atomically.
    pub dispatch_choice_action: Callback<((PageKey, u8), Actions)>,
    /// Reactive transition runtime: change tracking, current plan and backend.
    pub transitions: TransitionHandle,
    /// Opens the Save Game modal.
    pub open_save_modal: Callback<()>,
    /// Opens the Load Game modal.
    pub open_load_modal: Callback<()>,
    /// Opens the options modal.
    pub open_options_modal: Callback<()>,
}
