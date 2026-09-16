//! Core types and logic.

// Page
mod page;
mod page_state;
pub use page::*;
pub use page_state::*;

// Global state
pub mod game_state;
pub use game_state::{HASH_KEY_BIT, PageKey, PageMap, USER_KEY_MASK, hash_key};

// app, state -(guide)-> run -> view -(composed)-> element
mod game;
pub use game::*;

mod action;
pub use action::*;
