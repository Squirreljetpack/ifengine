//! IFEngine is a rust framework for writing interactive fiction.
//!
//! Other projects in this space include Twine and Inkle.
//!
//! The goal is to enjoy an effortless writing experience, together with all the benefits of the Rust ecosystem.
//!
//! To use this library, you write functions which produce [`Responses`](https://docs.rs/ifengine/latest/ifengine/core/enum.Response.html), eventualling resolving to a [`View`](https://docs.rs/ifengine/latest/ifengine/view/struct.View.html).
//! Such functions are called [`Pages`](https://docs.rs/ifengine/latest/ifengine/core/type.Page.html), and are decorated by the [`#[ifview]`](https://docs.rs/ifengine/latest/ifengine/attr.ifview.html) macro.
//! A starting page is used to initialize the [`Game`](https://docs.rs/ifengine/latest/ifengine/core/struct.Game.html), which can then be called upon to yield its current view through [`Game::view`](https://docs.rs/ifengine/latest/ifengine/core/struct.Game.html#method.view), and updated by interacting with the view.
//! A view consists of a sequence of [`Objects`](https://docs.rs/ifengine/latest/ifengine/view/enum.Object.html) which you can attach by calling the provided [elements and macros](https://docs.rs/ifengine/latest/ifengine/elements/index.html) within the page.

pub mod core;
mod errors;

#[cfg(feature = "macros")]
pub use ifengine_macros::ifview;

pub mod elements;
pub mod run;
pub mod utils;
pub mod view;

// core types needed to use the library
pub use {
    // local_state
    core::Action,
    // ui
    core::Game,
    errors::*,
    view::View,
};
