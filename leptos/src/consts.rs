//! Configuration constants for the Leptos frontend.

/// Whether to automatically generate IDs for spans without explicit IDs,
/// enabling view transitions on all newly revealed inline spans.
pub const AUTO_SPAN_TRANSITIONS: bool = true;

/// Pixel tolerance for treating an element's size or position as unchanged
/// when deciding between `fade` and `reflow` during view transitions.
pub const REFLOW_TOLERANCE_PX: f64 = 2.5;

/// Whether a secondary batch may stretch past `--secondary-reflow-duration` to
/// cover a longer authored fade. Off: the slide and the cross-fades both run at
/// the reflow duration, so a batch never outlives the length it declares.
pub const STRETCH_SECONDARY_TO_AUTHORED_FADE: bool = false;

/// Default transition-in delay in milliseconds for elements in primary transitions.
pub const PRIMARY_IN_DELAY_MS: u64 = 400;

/// Default transition-in delay in milliseconds for whole blocks that transition
/// in during primary transitions on same-page interactions.
pub const PRIMARY_BLOCK_IN_DELAY_MS: u64 = 400;

/// Whether to only add delayed elements to seen once their transitions have begun to run.
pub const ADD_DELAYED_TO_SEEN_ON_TRANSITION_START: bool = true;

