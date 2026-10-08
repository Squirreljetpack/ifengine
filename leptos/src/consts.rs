//! Configuration constants for the Leptos frontend.

/// Default fade-in duration in milliseconds when the `"in"` class is specified
/// without a custom duration suffix (defaults to 1000ms, or 2000ms for chant).
pub const DEFAULT_FADE_IN: u64 = if cfg!(feature = "chant") { 2000 } else { 1000 };

/// Default fade-out duration in milliseconds when the `"out"` class is specified
/// without a custom duration suffix (defaults to 1000ms, or 2000ms for chant).
pub const DEFAULT_FADE_OUT: u64 = if cfg!(feature = "chant") { 2000 } else { 1000 };

/// Default page transition fade-out duration in milliseconds (0.3s, or 0.6s for chant).
pub const PAGE_TRANSITION_OUT: u64 = if cfg!(feature = "chant") { 600 } else { 300 };

/// Default page transition fade-in duration in milliseconds (0.2s, or 0.4s for chant).
pub const PAGE_TRANSITION_IN: u64 = if cfg!(feature = "chant") { 400 } else { 200 };

/// Default element view-transition fade-out duration in milliseconds (0.3s, or 0.6s for chant).
pub const ELEMENT_TRANSITION_OUT: u64 = if cfg!(feature = "chant") { 600 } else { 300 };

/// Default element view-transition fade-in duration in milliseconds (0.2s, or 0.4s for chant).
pub const ELEMENT_TRANSITION_IN: u64 = if cfg!(feature = "chant") { 400 } else { 200 };

/// Maximum width of the story text container in rem.
pub const MAX_PAGE_WIDTH_REM: f32 = 44.0;

/// Whether to automatically generate IDs for spans without explicit IDs,
/// enabling view transitions on all newly revealed inline spans.
pub const AUTO_SPAN_TRANSITIONS: bool = true;
