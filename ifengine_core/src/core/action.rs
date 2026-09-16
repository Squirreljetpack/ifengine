use crate::core::{PageHandle, game_state::PageKey};

/// Adds an effect to a [`Span`](crate::view::Span)
/// Spans with an action occlude their containing object (i.e. [`Choice`](crate::view::Object::Choice))
#[derive(Debug, Default, Clone)]
pub enum Action {
    #[default]
    None,
    SetBit(PageKey, u8),
    Set(PageKey, u64),
    SetInc(PageKey, u64),
    Inc(PageKey),
    Reset(PageKey),
    /// The name of the handle here is purely descriptive, and NOT guaranteed to be the actual id of the page, see [`PageState`](crate::core::PageState)
    Next(PageHandle), // Arc for easy cloning
    Back(usize),
    Tunnel(PageHandle),
    Exit,
}
