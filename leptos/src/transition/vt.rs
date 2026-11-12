//! View Transitions strategy.
//!
//! Bridges the runtime to `document.startViewTransition`. The shim (`vt.js`) reads
//! the resolved plan, snapshots the pre-update DOM, patches leaving/changed elements,
//! runs the update, then classifies each named element as `fade` or `reflow`.

use wasm_bindgen::prelude::*;

use super::plan::{FadeDirection, TransitionPlan};

#[wasm_bindgen(module = "/src/transition/vt.js")]
extern "C" {
    fn run_view_transition(
        plan_json: &str,
        is_secondary: bool,
        update: &js_sys::Function,
        on_finished: &js_sys::Function,
    );
}

/// Whether the browser exposes same-document view transitions.
pub(crate) fn available() -> bool {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return false;
    };
    let doc_val = JsValue::from(&document);
    matches!(
        js_sys::Reflect::get(&doc_val, &JsValue::from_str("startViewTransition")),
        Ok(method) if method.is_function()
    )
}

/// Starts a view transition around `update`; calls `on_finished` when it settles.
pub(crate) fn start(
    plan_json: &str,
    is_secondary: bool,
    update: &js_sys::Function,
    on_finished: &js_sys::Function,
) {
    run_view_transition(plan_json, is_secondary, update, on_finished);
}

/// Writes `::view-transition-{new,old}(.{token})` duration overrides for explicit
/// authored durations into `<style id="vt-story-durations">`.
pub fn ensure_duration_styles(plan: &TransitionPlan) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(head) = document.head() else {
        return;
    };

    let style_id = "vt-story-durations";
    let style_el = match document.get_element_by_id(style_id) {
        Some(el) => el,
        None => {
            let Ok(el) = document.create_element("style") else {
                return;
            };
            el.set_id(style_id);
            let _ = head.append_child(&el);
            el
        }
    };

    let existing = style_el.text_content().unwrap_or_default();
    let mut to_append = String::new();
    for rule in &plan.duration_rules {
        let pseudo = match rule.direction {
            FadeDirection::In => "new",
            FadeDirection::Out => "old",
        };
        if existing.contains(&format!(".{})", rule.token)) {
            continue;
        }
        let css = format!(
            "::view-transition-{pseudo}(.{token}) {{ animation-duration: {ms}ms !important; }}",
            token = rule.token,
            ms = rule.duration_ms
        );
        // The secondary rules match with higher specificity than a bare pseudo
        // rule, so repeat the duration there to keep authored values in force.
        to_append.push_str(&css);
        to_append.push_str(&format!(
            " html.secondary-transition::view-transition-{pseudo}(.{}) {{ animation-duration: {}ms !important; }} ",
            rule.token, rule.duration_ms
        ));
    }
    if !to_append.is_empty() {
        style_el.set_text_content(Some(&format!("{existing}{to_append}")));
    }
}
