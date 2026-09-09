//! Leptos frontend for `ifengine`.
//!
//! Provides a responsive, accessible web presentation layer with Chapbook-inspired typography,
//! fine-grained reactive state handling, and a deterministic ID-based transition/animation engine.

pub mod app;
pub mod components;
pub mod consts;
pub mod context;
pub mod render;
pub mod transition;

pub use app::{App, HeaderExtractor, PageTransitionPhase, StoryApp};
pub use context::StoryContext;
