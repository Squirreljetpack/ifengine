//! Core types and runtime for the ifengine interactive fiction framework.

pub mod core;
pub mod elements;
mod errors;
pub mod run;
pub mod utils;
pub mod view;

// Core types needed to use the library
pub use {core::Action, core::Game, errors::*, view::View};
