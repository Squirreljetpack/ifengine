pub mod core;
mod errors;

#[cfg(feature = "macros")]
pub use ifengine_macros::ifview;

pub mod elements;
pub mod utils;
pub mod view;
pub mod run;

// core types needed to use the library
pub use {
    // local_state
    core::Action,
    // ui
    core::Game,
    errors::*,
    view::View,
};
