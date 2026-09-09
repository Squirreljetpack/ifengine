use ifengine::core::{Action, PageId, game_state::PageKey};
use leptos::prelude::*;

use crate::transition::TransitionManager;

/// Unified reactive context provided at the application root.
///
/// Eliminates prop drilling by delivering action/choice dispatchers
/// and the transition state directly to leaf components (`SpanView`, `ChoiceView`).
#[derive(Clone, Copy)]
pub struct StoryContext {
    /// Dispatches state mutations and passage transitions.
    pub dispatch_action: Callback<Action>,
    /// Dispatches bitmask choice selections.
    pub dispatch_choice: Callback<((PageId, PageKey), u8)>,
    /// Reactive transition manager tracking animated items across iterations.
    pub transitions: RwSignal<TransitionManager>,
}
