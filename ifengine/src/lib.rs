//! IFEngine is a rust framework for writing interactive fiction.
//!
//! Other projects in this space include Twine and Inkle.
//!
//! The goal is to enjoy an effortless writing experience, together with all the benefits of the Rust ecosystem.

//! To use this library, you write functions which produce [`Responses`](https://docs.rs/ifengine/latest/ifengine/core/enum.Response.html), eventualling resolving to a [`View`](https://docs.rs/ifengine/latest/ifengine/view/struct.View.html). The view corresponding to the current game state is retrieved by calling [`Game::view`](https://docs.rs/ifengine/latest/ifengine/core/struct.Game.html#method.view).
//!
//! A view is a sequence of [`Objects`](https://docs.rs/ifengine/latest/ifengine/view/enum.Object.html) which you can attach by calling the provided [elements and macros](https://docs.rs/ifengine/latest/ifengine/elements/index.html) within a function decorated by [`#[ifview]`](https://docs.rs/ifengine/latest/ifengine/attr.ifview.html).

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
