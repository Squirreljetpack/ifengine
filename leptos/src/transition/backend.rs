//! Backend selection and dispatch.
//!
//! Components never call a browser API directly; they build a plan and hand it to
//! [`run`], which routes to the selected [`TransitionBackend`]. Selection is a pure
//! function over an injected [`Engine`] and [`Capabilities`] so it is testable without
//! a DOM; an explicit choice — the stored preference or a `?vt=vt|css|instant` query
//! parameter — overrides the result.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::plan::TransitionPlan;
use super::{css, vt};

/// Which strategy effects transitions.
///
/// The variant names double as the stored preference values and the `?vt=` override values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionBackend {
    /// `document.startViewTransition` snapshot-based transitions.
    #[serde(rename = "vt")]
    ViewTransition,
    /// Clone-snapshot overlay fades and image FLIPs; used on Gecko.
    #[serde(rename = "css")]
    Css,
    /// Apply updates immediately; no same-page animation.
    #[serde(rename = "instant")]
    Instant,
}

impl TransitionBackend {
    /// Resolves the backend from the stored preference, the `?vt=` override, the engine
    /// and the browser capabilities, then announces it.
    pub(crate) fn detect() -> Self {
        let forced = forced_backend().or(crate::storage::load_preferences().motion);
        let backend = select_backend(Engine::detect(), forced, &Capabilities::detect());
        announce(backend);
        backend
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ViewTransition => "vt",
            Self::Css => "css",
            Self::Instant => "instant",
        }
    }
}

/// Records `backend` on `<html data-vt-backend>` and logs it once.
pub(crate) fn announce(backend: TransitionBackend) {
    if let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    {
        let _ = root.set_attribute("data-vt-backend", backend.as_str());
    }
    web_sys::console::log_1(
        &format!("[ifengine_leptos] transition backend: {}", backend.as_str()).into(),
    );
}

/// The browser engine family, used for the default policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    /// Firefox and Gecko-based forks (Zen, ...).
    Gecko,
    /// Chrome, Chromium, Edge.
    Blink,
    /// Everything else, including WebKit.
    Other,
}

impl Engine {
    fn detect() -> Self {
        let user_agent = web_sys::window()
            .and_then(|window| {
                let navigator = window.navigator();
                navigator.user_agent().ok()
            })
            .unwrap_or_default();
        if user_agent.contains("Firefox") || user_agent.contains("FxiOS") {
            Self::Gecko
        } else if user_agent.contains("Chrome")
            || user_agent.contains("Chromium")
            || user_agent.contains("Edg")
        {
            Self::Blink
        } else {
            Self::Other
        }
    }
}

/// Browser facts the policy needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    pub start_view_transition: bool,
    pub view_transition_class: bool,
    pub view_transition_name: bool,
}

impl Capabilities {
    fn detect() -> Self {
        Self {
            start_view_transition: vt::available(),
            view_transition_class: web_sys::css::supports_with_value(
                "view-transition-class",
                "none",
            )
            .unwrap_or(false),
            view_transition_name: web_sys::css::supports_with_value("view-transition-name", "none")
                .unwrap_or(false),
        }
    }
}

/// Default backend policy: an explicit choice wins, otherwise the engine decides.
pub fn select_backend(
    engine: Engine,
    forced: Option<TransitionBackend>,
    capabilities: &Capabilities,
) -> TransitionBackend {
    if let Some(forced) = forced {
        return forced;
    }
    match engine {
        Engine::Blink
            if capabilities.start_view_transition
                && capabilities.view_transition_class
                && capabilities.view_transition_name =>
        {
            TransitionBackend::ViewTransition
        }
        Engine::Blink => TransitionBackend::Instant,
        // Gecko mishandles inline elements broken across line boxes during VT
        // capture (bug 2080048), so it uses the clone backend unconditionally.
        Engine::Gecko => TransitionBackend::Css,
        Engine::Other => TransitionBackend::Instant,
    }
}

fn forced_backend() -> Option<TransitionBackend> {
    let search = web_sys::window()?.location().search().ok()?;
    for pair in search.trim_start_matches('?').split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key != "vt" {
            continue;
        }
        return match value {
            "vt" | "view-transition" => Some(TransitionBackend::ViewTransition),
            "css" => Some(TransitionBackend::Css),
            "instant" => Some(TransitionBackend::Instant),
            _ => None,
        };
    }
    None
}

/// Runs `callbacks` under `backend`, then invokes `on_finished`.
pub(crate) fn run(
    backend: TransitionBackend,
    plan: Arc<TransitionPlan>,
    is_secondary: bool,
    callbacks: Vec<Box<dyn FnOnce()>>,
    on_finished: impl FnOnce() + 'static,
) {
    match backend {
        TransitionBackend::ViewTransition => {
            let plan_json = serde_json::to_string(&plan).unwrap_or_else(|_| "{}".to_string());
            let update_closure = Closure::once_into_js(move || {
                for callback in callbacks {
                    callback();
                }
            });
            let update: js_sys::Function = update_closure.unchecked_into();

            if vt::available() {
                let finished_closure = Closure::once_into_js(move |_: JsValue| on_finished());
                let finished: js_sys::Function = finished_closure.unchecked_into();
                vt::start(&plan_json, is_secondary, &update, &finished);
            } else {
                let _ = update.call0(&JsValue::UNDEFINED);
                on_finished();
            }
        }
        TransitionBackend::Css => {
            let plan_json = serde_json::to_string(&plan).unwrap_or_else(|_| "{}".to_string());
            let update_closure = Closure::once_into_js(move || {
                for callback in callbacks {
                    callback();
                }
            });
            let update: js_sys::Function = update_closure.unchecked_into();
            let finished_closure = Closure::once_into_js(move |_: JsValue| on_finished());
            let finished: js_sys::Function = finished_closure.unchecked_into();
            css::run(&plan_json, is_secondary, &update, &finished);
        }
        TransitionBackend::Instant => {
            for callback in callbacks {
                callback();
            }
            on_finished();
        }
    }
}

/// Cancels backend effects; called on fresh page navigation.
pub(crate) fn reset(backend: TransitionBackend) {
    if backend == TransitionBackend::Css {
        css::reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps(
        start_view_transition: bool,
        view_transition_class: bool,
        view_transition_name: bool,
    ) -> Capabilities {
        Capabilities {
            start_view_transition,
            view_transition_class,
            view_transition_name,
        }
    }

    #[test]
    fn selection_table() {
        let full = caps(true, true, true);
        let partial = caps(true, false, true);
        let none = caps(false, false, false);

        assert_eq!(
            select_backend(Engine::Blink, None, &full),
            TransitionBackend::ViewTransition
        );
        assert_eq!(
            select_backend(Engine::Blink, None, &partial),
            TransitionBackend::Instant
        );
        assert_eq!(
            select_backend(Engine::Blink, None, &none),
            TransitionBackend::Instant
        );
        assert_eq!(
            select_backend(Engine::Gecko, None, &full),
            TransitionBackend::Css
        );
        assert_eq!(
            select_backend(Engine::Other, None, &full),
            TransitionBackend::Instant
        );
        // An explicit choice beats every other input.
        assert_eq!(
            select_backend(
                Engine::Other,
                Some(TransitionBackend::ViewTransition),
                &none
            ),
            TransitionBackend::ViewTransition
        );
        assert_eq!(
            select_backend(Engine::Gecko, Some(TransitionBackend::Instant), &full),
            TransitionBackend::Instant
        );
    }

    #[test]
    fn backend_names_match_stored_values() {
        for backend in [
            TransitionBackend::ViewTransition,
            TransitionBackend::Css,
            TransitionBackend::Instant,
        ] {
            let stored = serde_json::to_string(&backend).unwrap();
            assert_eq!(stored, format!("\"{}\"", backend.as_str()));
            assert_eq!(
                serde_json::from_str::<TransitionBackend>(&stored).unwrap(),
                backend
            );
        }
    }
}
