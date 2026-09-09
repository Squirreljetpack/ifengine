//! Configuration constants for the Leptos frontend.

/// Default fade-in duration in milliseconds when the `"in"` class is specified
/// without a custom duration suffix (defaults to 1000ms).
pub const DEFAULT_FADE_IN: u64 = 1000;

/// Default fade-out duration in milliseconds when the `"out"` class is specified
/// without a custom duration suffix (defaults to 1000ms).
pub const DEFAULT_FADE_OUT: u64 = 1000;

/// Default page transition fade-out duration in milliseconds (0.3s).
pub const PAGE_TRANSITION_OUT: u64 = 300;

/// Default page transition fade-in duration in milliseconds (0.2s).
pub const PAGE_TRANSITION_IN: u64 = 200;

/// Default element view-transition fade-out duration in milliseconds (0.3s).
pub const ELEMENT_TRANSITION_OUT: u64 = 300;

/// Default element view-transition fade-in duration in milliseconds (0.2s).
pub const ELEMENT_TRANSITION_IN: u64 = 200;

/// Maximum width of the story text container in rem.
pub const MAX_PAGE_WIDTH_REM: f32 = 44.0;
