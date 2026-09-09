//! Animation and transition management for `ifengine_leptos`.
//!
//! Handles `"in"`, `"in-{d}"`, `"in-{d}-{t}"`, `"out"`, `"out-{d}"`, and `"out-{d}-{t}"` classes,
//! and assigns `view-transition-name` to elements with an ID.
//! Enforces the rule that entrance animations on the same item do not re-trigger across iterations.

use std::collections::HashMap;

use ifengine::core::{PageId, game_state::PageKey};
use leptos::prelude::*;

use crate::consts::{DEFAULT_FADE_IN, DEFAULT_FADE_OUT};

/// Parsed animation transition configuration for an individual element (Span or Line).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransitionConfig {
    /// Duration in milliseconds for fade-in animation.
    pub fade_in: Option<u64>,
    /// Duration in milliseconds for fade-out animation.
    pub fade_out: Option<u64>,
    /// Delay in milliseconds before fade-in commences.
    pub in_delay: Option<u64>,
    /// Delay in milliseconds before fading back out (after fully fading in).
    pub out_delay: Option<u64>,
}

impl TransitionConfig {
    /// Returns true if any animation transition is configured.
    pub fn has_transition(&self) -> bool {
        self.fade_in.is_some() || self.fade_out.is_some()
    }
}

/// Parses the class list of a `Span` or `Line` to extract transition timing parameters.
///
/// Supported patterns:
/// - `"in"` -> fade in with [`DEFAULT_FADE_IN_DURATION_MS`] duration (0 delay)
/// - `"in-{d}"` -> fade in with `{d}` ms delay and [`DEFAULT_FADE_IN_DURATION_MS`] duration
/// - `"in-{d}-{t}"` -> fade in with `{d}` ms delay and `{t}` ms duration
/// - `"out"` -> fade out with [`DEFAULT_FADE_OUT_DURATION_MS`] duration (0 delay)
/// - `"out-{d}"` -> fade out after `{d}` ms delay with [`DEFAULT_FADE_OUT_DURATION_MS`] duration
/// - `"out-{d}-{t}"` -> fade out after `{d}` ms delay with `{t}` ms duration
pub fn parse_transition_classes(classes: &[String]) -> TransitionConfig {
    let mut config = TransitionConfig::default();

    for class in classes {
        let class = class.trim();

        // 1. "in" or "in-{d}" or "in-{d}-{t}"
        if class == "in" {
            config.in_delay = Some(0);
            config.fade_in = Some(DEFAULT_FADE_IN);
        } else if let Some(suffix) = class.strip_prefix("in-") {
            let parts: Vec<&str> = suffix.split('-').collect();
            match parts.len() {
                1 => {
                    if let Ok(d) = parts[0].parse::<u64>() {
                        config.in_delay = Some(d);
                    } else {
                        config.in_delay = Some(0);
                    }
                    config.fade_in = Some(DEFAULT_FADE_IN);
                }
                2 => {
                    let d = parts[0].parse::<u64>().unwrap_or(0);
                    let t = parts[1].parse::<u64>().unwrap_or(DEFAULT_FADE_IN);
                    config.in_delay = Some(d);
                    config.fade_in = Some(t);
                }
                _ => {}
            }
        }

        // 2. "out" or "out-{d}" or "out-{d}-{t}"
        if class == "out" {
            config.out_delay = Some(0);
            config.fade_out = Some(DEFAULT_FADE_OUT);
        } else if let Some(suffix) = class.strip_prefix("out-") {
            let parts: Vec<&str> = suffix.split('-').collect();
            match parts.len() {
                1 => {
                    if let Ok(d) = parts[0].parse::<u64>() {
                        config.out_delay = Some(d);
                    } else {
                        config.out_delay = Some(0);
                    }
                    config.fade_out = Some(DEFAULT_FADE_OUT);
                }
                2 => {
                    let d = parts[0].parse::<u64>().unwrap_or(0);
                    let t = parts[1].parse::<u64>().unwrap_or(DEFAULT_FADE_OUT);
                    config.out_delay = Some(d);
                    config.fade_out = Some(t);
                }
                _ => {}
            }
        }
    }

    config
}

/// Tracks previously rendered elements across page iterations to ensure
/// that entrance animations never re-trigger on items that were already displayed,
/// while re-triggering when content changes under the same ID (e.g. `alts!` clicked).
#[derive(Debug, Clone)]
pub struct TransitionManager {
    /// Tracks `(PageId, PageKey)` and the last rendered content hash to detect content changes.
    seen_page_items: HashMap<(PageId, PageKey), u64>,
    /// Currently active page ID.
    current_page: PageId,
    /// Whether the current view is a fresh page load (on which entrance animations are suppressed).
    pub is_fresh: bool,
}

impl TransitionManager {
    /// Creates a new `TransitionManager` initialized with the initial story page ID.
    pub fn new(initial_page_id: PageId) -> Self {
        Self {
            seen_page_items: HashMap::new(),
            current_page: initial_page_id,
            is_fresh: true,
        }
    }

    /// Called whenever the active view changes.
    ///
    /// Seen items are indexed by `(PageId, PageKey)`, ensuring entrance animations and delays
    /// do not re-trigger when revisiting previously seen passages unless content changes.
    pub fn on_view_change(&mut self, new_page_id: &PageId, is_fresh: bool) {
        self.current_page = new_page_id.clone();
        self.is_fresh = is_fresh;
    }

    /// Checks whether an item's content changed or is newly seen on the current page based on its hash.
    ///
    /// On fresh page load (`is_fresh == true`), returns `false` so entrance animations are suppressed.
    /// On same-page interactions (`is_fresh == false`), newly seen or content-changed items return `true`.
    /// If previously seen with the exact same hash: returns `false`.
    pub fn is_content_changed(&mut self, id: Option<PageKey>, hash: u64) -> bool {
        let Some(key) = id else {
            return false;
        };

        let item_key = (self.current_page.clone(), key);
        if let Some(&prev_hash) = self.seen_page_items.get(&item_key) {
            if prev_hash == hash {
                false
            } else {
                self.seen_page_items.insert(item_key, hash);
                true
            }
        } else {
            self.seen_page_items.insert(item_key, hash);
            if self.is_fresh { false } else { hash != 0 }
        }
    }
}

/// Lifecycle phase for elements with delayed entrance or exit transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    /// Before entrance delay expires: element must not take up any space (display: none).
    Pending,
    /// Actively animating or visible.
    Active,
    /// Finished fading out or already faded out in a prior iteration: element removed (display: none).
    Removed,
}

/// Determines the initial transition phase for an element.
pub fn compute_initial_phase(config: &TransitionConfig, should_animate: bool) -> TransitionPhase {
    if !config.has_transition() {
        return TransitionPhase::Active;
    }

    if !should_animate {
        if config.fade_out.is_some() {
            TransitionPhase::Removed
        } else {
            TransitionPhase::Active
        }
    } else {
        let in_delay = config.in_delay.unwrap_or(0);
        if in_delay > 0 {
            TransitionPhase::Pending
        } else {
            TransitionPhase::Active
        }
    }
}

/// Schedules timeouts to transition an element from `Pending` -> `Active` (after in_delay),
/// and from `Active` -> `Removed` (after fading out completes).
pub fn setup_transition_timers(
    config: &TransitionConfig,
    set_phase: leptos::prelude::WriteSignal<TransitionPhase>,
) {
    let in_delay = config.in_delay.unwrap_or(0);
    let in_duration = config.fade_in.unwrap_or(DEFAULT_FADE_IN);
    let out_delay = config.out_delay.unwrap_or(0);
    let out_duration = config.fade_out.unwrap_or(DEFAULT_FADE_OUT);

    if in_delay > 0 {
        leptos::prelude::set_timeout(
            move || {
                set_phase.set(TransitionPhase::Active);
            },
            std::time::Duration::from_millis(in_delay),
        );
    }

    if config.fade_out.is_some() {
        let total_ms = if config.fade_in.is_some() || in_delay > 0 {
            in_delay + in_duration + out_delay + out_duration
        } else {
            out_delay + out_duration
        };

        leptos::prelude::set_timeout(
            move || {
                set_phase.set(TransitionPhase::Removed);
            },
            std::time::Duration::from_millis(total_ms),
        );
    }
}

/// Generates active transition styles when an element is in the `Active` phase.
///
/// Because `in_delay` was already waited during `Pending`, the entrance fade-in starts immediately.
pub fn generate_active_transition_style(
    config: &TransitionConfig,
    should_animate: bool,
) -> (String, Vec<&'static str>) {
    if !config.has_transition() {
        return (String::new(), vec![]);
    }

    if !should_animate {
        if config.fade_out.is_some() {
            (
                "display: none !important; animation: none !important;".to_string(),
                vec!["skip-animation", "fade-out-removed"],
            )
        } else {
            (
                "opacity: 1; animation: none !important;".to_string(),
                vec!["skip-animation"],
            )
        }
    } else {
        if config.fade_in.is_some() && config.fade_out.is_some() {
            let in_duration = config.fade_in.unwrap_or(DEFAULT_FADE_IN);
            let out_delay = config.out_delay.unwrap_or(0);
            let out_duration = config.fade_out.unwrap_or(DEFAULT_FADE_OUT);
            let start_out = in_duration + out_delay;

            let style = format!(
                "animation: fade-in {in_duration}ms ease-out both, fade-out {out_duration}ms ease-out {start_out}ms forwards;"
            );
            (style, vec!["transition-active"])
        } else if let Some(fade_out_ms) = config.fade_out {
            let out_delay = config.out_delay.unwrap_or(0);
            let style = format!(
                "animation-name: fade-out; animation-duration: {fade_out_ms}ms; animation-delay: {out_delay}ms; animation-fill-mode: both; animation-timing-function: ease-out;"
            );
            (style, vec!["transition-active"])
        } else {
            let in_duration = config.fade_in.unwrap_or(DEFAULT_FADE_IN);
            let style = format!(
                "animation-name: fade-in; animation-duration: {in_duration}ms; animation-delay: 0ms; animation-fill-mode: both; animation-timing-function: ease-out;"
            );
            (style, vec!["transition-active"])
        }
    }
}

/// Generates the `view-transition-name` and `view-transition-class` inline CSS style properties for elements with an ID.
///
/// Assigns `view-transition-class: fade` if `is_changed` is true, or `view-transition-class: reflow` if false.
pub fn generate_view_transition_style(id: Option<PageKey>, is_changed: bool) -> String {
    if let Some(key) = id {
        let vt_class = if is_changed { "fade" } else { "reflow" };
        format!("view-transition-name: item-{key}; view-transition-class: {vt_class};")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_defaults() {
        let c = parse_transition_classes(&["in".to_string()]);
        assert_eq!(c.in_delay, Some(0));
        assert_eq!(c.fade_in, Some(DEFAULT_FADE_IN));

        let c = parse_transition_classes(&["out".to_string()]);
        assert_eq!(c.out_delay, Some(0));
        assert_eq!(c.fade_out, Some(DEFAULT_FADE_OUT));
    }

    #[test]
    fn test_parse_in_and_out_timing() {
        // "in-200": delay 200, duration default
        let c = parse_transition_classes(&["in-200".to_string()]);
        assert_eq!(c.in_delay, Some(200));
        assert_eq!(c.fade_in, Some(DEFAULT_FADE_IN));

        // "in-200-500": delay 200, duration 500
        let c = parse_transition_classes(&["in-200-500".to_string()]);
        assert_eq!(c.in_delay, Some(200));
        assert_eq!(c.fade_in, Some(500));

        // "in-0-500": delay 0, duration 500
        let c = parse_transition_classes(&["in-0-500".to_string()]);
        assert_eq!(c.in_delay, Some(0));
        assert_eq!(c.fade_in, Some(500));

        // "out-2000": delay 2000, duration default
        let c = parse_transition_classes(&["out-2000".to_string()]);
        assert_eq!(c.out_delay, Some(2000));
        assert_eq!(c.fade_out, Some(DEFAULT_FADE_OUT));

        // "out-2000-500": delay 2000, duration 500
        let c = parse_transition_classes(&["out-2000-500".to_string()]);
        assert_eq!(c.out_delay, Some(2000));
        assert_eq!(c.fade_out, Some(500));

        // Combined in and out
        let c = parse_transition_classes(&["in-100-400".to_string(), "out-2000-600".to_string()]);
        assert_eq!(c.in_delay, Some(100));
        assert_eq!(c.fade_in, Some(400));
        assert_eq!(c.out_delay, Some(2000));
        assert_eq!(c.fade_out, Some(600));
    }

    #[test]
    fn test_generate_view_transition_style() {
        assert_eq!(
            generate_view_transition_style(Some(12345), true),
            "view-transition-name: item-12345; view-transition-class: fade;"
        );
        assert_eq!(
            generate_view_transition_style(Some(12345), false),
            "view-transition-name: item-12345; view-transition-class: reflow;"
        );
        assert_eq!(generate_view_transition_style(None, true), "");
        assert_eq!(generate_view_transition_style(None, false), "");
    }

    #[test]
    fn test_anti_retrigger_across_iterations() {
        let page = PageId("page_1".into());
        let mut tm = TransitionManager::new(page.clone());
        let item_id = 12345u64;

        // Iteration 1: On fresh page load, NO entrance animations trigger (content not marked changed)!
        assert!(!tm.is_content_changed(Some(item_id), 100));

        // Iteration 2 (same page, user clicks another choice, content hash unchanged)
        tm.on_view_change(&page, false);

        // Content unchanged: MUST NOT ANIMATE!
        assert!(!tm.is_content_changed(Some(item_id), 100));

        // Iteration 2b: user clicks this alt! Content hash changes to 200: MUST ANIMATE!
        assert!(tm.is_content_changed(Some(item_id), 200));

        // Subsequent render with content hash 200: MUST NOT ANIMATE!
        assert!(!tm.is_content_changed(Some(item_id), 200));

        // Items without ID do not animate
        assert!(!tm.is_content_changed(None, 300));

        // Now user navigates to fresh page_2
        let page_2 = PageId("page_2".into());
        tm.on_view_change(&page_2, true);
        // On fresh page load, NO entrance animations trigger!
        assert!(!tm.is_content_changed(Some(item_id), 100));

        // Same-page interaction on page_2: newly revealed item DOES animate
        tm.on_view_change(&page_2, false);
        let new_item_id = 99999u64;
        assert!(tm.is_content_changed(Some(new_item_id), 500));

        // User navigates back to page_1 (fresh = true)
        tm.on_view_change(&page, true);
        // Previously seen items on page_1 remain seen across page visits unless content changed
        assert!(!tm.is_content_changed(Some(item_id), 200));
        // If content on page_1 changed to 300, it animates again
        assert!(tm.is_content_changed(Some(item_id), 300));
    }

    #[test]
    fn test_transition_phase_and_helpers() {
        // compute_initial_phase
        let config_delayed = parse_transition_classes(&["in-500".to_string()]);
        assert_eq!(
            compute_initial_phase(&config_delayed, true),
            TransitionPhase::Pending
        );
        assert_eq!(
            compute_initial_phase(&config_delayed, false),
            TransitionPhase::Active
        );

        let config_immediate = parse_transition_classes(&["in".to_string()]);
        assert_eq!(
            compute_initial_phase(&config_immediate, true),
            TransitionPhase::Active
        );

        let config_out = parse_transition_classes(&["out-500".to_string()]);
        assert_eq!(
            compute_initial_phase(&config_out, true),
            TransitionPhase::Active
        );
        assert_eq!(
            compute_initial_phase(&config_out, false),
            TransitionPhase::Removed
        );

        // generate_active_transition_style
        let (style, classes) = generate_active_transition_style(&config_delayed, true);
        assert!(style.contains("animation-delay: 0ms"));
        assert_eq!(classes, vec!["transition-active"]);

        let (style_skip, classes_skip) = generate_active_transition_style(&config_out, false);
        assert!(style_skip.contains("display: none !important"));
        assert!(classes_skip.contains(&"fade-out-removed"));
    }

    #[test]
    fn test_choice_transition_lifecycle() {
        let page = PageId("test_page".into());
        let mut tm = TransitionManager::new(page.clone());
        let choice_key = 64u64;

        // Iteration 1: Fresh page with a choice (registers with hash 0 -> is_changed must be false / reflow)
        let is_changed = tm.is_content_changed(Some(choice_key), 0);
        assert!(
            !is_changed,
            "choice should not be marked changed on initial view"
        );
        assert_eq!(
            generate_view_transition_style(Some(choice_key), is_changed),
            "view-transition-name: item-64; view-transition-class: reflow;"
        );

        // Choice arms without line id are NOT registered (pass None)
        let arm_changed = tm.is_content_changed(None, 12345);
        assert!(!arm_changed);

        // Iteration 2: Same-page interaction (e.g. alt-1/23 clicked)
        tm.on_view_change(&page, false);

        // Choice re-renders with hash 0 -> MUST STILL BE REFLOW (not fade!)
        let is_changed_after_alt = tm.is_content_changed(Some(choice_key), 0);
        assert!(
            !is_changed_after_alt,
            "choice should remain reflow across alt interactions"
        );
        assert_eq!(
            generate_view_transition_style(Some(choice_key), is_changed_after_alt),
            "view-transition-name: item-64; view-transition-class: reflow;"
        );

        // Iteration 3: User selects choice! It morphs into a paragraph sharing choice_key with content hash
        tm.on_view_change(&page, false);
        let paragraph_hash = 99999u64;
        let is_changed_on_selection = tm.is_content_changed(Some(choice_key), paragraph_hash);
        assert!(
            is_changed_on_selection,
            "morphing from choice (hash 0) to paragraph (hash != 0) should be marked changed (fade)"
        );
        assert_eq!(
            generate_view_transition_style(Some(choice_key), is_changed_on_selection),
            "view-transition-name: item-64; view-transition-class: fade;"
        );

        // Iteration 4: Another same-page interaction after paragraph is already chosen
        tm.on_view_change(&page, false);
        let is_changed_later = tm.is_content_changed(Some(choice_key), paragraph_hash);
        assert!(
            !is_changed_later,
            "paragraph with same content should now reflow"
        );
        assert_eq!(
            generate_view_transition_style(Some(choice_key), is_changed_later),
            "view-transition-name: item-64; view-transition-class: reflow;"
        );
    }
}
