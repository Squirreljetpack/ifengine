use crate::core::{PageHandle, game_state::InternalKey};

#[derive(Debug, Default, Clone)]
pub enum Action {
    #[default]
    None,
    SetBit(InternalKey, u8),
    Set(InternalKey, u64),
    Inc(InternalKey),
    Reset(InternalKey),
    /// The name of the handle here is just for debug, and NOT guaranteed to be the actual id of the page, see [`crate::core::PageState`]
    Next(PageHandle), // Arc for easy cloning
    Back(usize),
    Tunnel(PageHandle),
    Exit,
}
