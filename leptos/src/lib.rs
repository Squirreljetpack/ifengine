//! Leptos frontend for `ifengine`.
//!
//! Provides a responsive, accessible web presentation layer with Chapbook-inspired typography,
//! fine-grained reactive state handling, and a deterministic ID-based transition/animation engine.

mod app;
mod app_impl;

mod components;
pub mod consts;
mod context;
mod render;
pub mod storage;
mod transition;

pub use app::App;

cfg_if::cfg_if! {
    if #[cfg(feature = "test")] {
        pub use test_story as story;
    } else if #[cfg(feature = "forest")] {
        pub use forest as story;
    } else if #[cfg(feature = "saltwrack")] {
        pub use saltwrack as story;
    } else {
        compile_error!("At least one story feature must be enabled: `saltwrack`, `forest`, or `test`");
    }
}
