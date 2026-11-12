//! Clone-snapshot CSS strategy.
//!
//! Used by Gecko-based engines, where inline elements broken across line boxes are
//! mishandled by the View Transitions capture (bug 2080048). `css.js` clones the old
//! subtree into a fixed overlay, swaps immediately, fades old/new at true geometry,
//! and transform-FLIPs moved block-level images.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/transition/css.js")]
extern "C" {
    fn run_css_transition(
        plan_json: &str,
        is_secondary: bool,
        update: &js_sys::Function,
        on_finished: &js_sys::Function,
    );
    fn reset_css_transition();
}

/// Runs `update` immediately, with clone-snapshot fades around it.
pub(crate) fn run(
    plan_json: &str,
    is_secondary: bool,
    update: &js_sys::Function,
    on_finished: &js_sys::Function,
) {
    run_css_transition(plan_json, is_secondary, update, on_finished);
}

/// Removes any clone overlay and effect classes from an in-flight transition.
pub(crate) fn reset() {
    reset_css_transition();
}
