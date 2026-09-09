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
mod transition;

pub use app::App;
