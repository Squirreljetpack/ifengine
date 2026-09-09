//! Animation and transition management for `ifengine_leptos`.
//!
//! Handles `"in"`, `"in-{d}"`, `"in-{d}-{t}"`, `"out"`, `"out-{d}"`, and `"out-{d}-{t}"` classes,
//! and assigns `view-transition-name` to elements with an ID.
//! Enforces the rule that entrance animations on the same item do not re-trigger across iterations.

use std::collections::HashMap;

use ifengine::core::{PageId, game_state::PageKey};

use crate::consts::{DEFAULT_FADE_IN_MS, DEFAULT_FADE_OUT_MS};

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
/// - `"in"` -> fade in with [`DEFAULT_FADE_IN_MS`] duration (0 delay)
/// - `"in-{d}"` -> fade in with `{d}` ms delay and [`DEFAULT_FADE_IN_MS`] duration
/// - `"in-{d}-{t}"` -> fade in with `{d}` ms delay and `{t}` ms duration
/// - `"out"` -> fade out with [`DEFAULT_FADE_OUT_MS`] duration (0 delay)
/// - `"out-{d}"` -> fade out after `{d}` ms delay with [`DEFAULT_FADE_OUT_MS`] duration
/// - `"out-{d}-{t}"` -> fade out after `{d}` ms delay with `{t}` ms duration
pub fn parse_transition_classes(classes: &[String]) -> TransitionConfig {
    let mut config = TransitionConfig::default();

    for class in classes {
        let class = class.trim();

        // 1. "in" or "in-{d}" or "in-{d}-{t}"
        if class == "in" {
            config.in_delay = Some(0);
            config.fade_in = Some(DEFAULT_FADE_IN_MS);
        } else if let Some(suffix) = class.strip_prefix("in-") {
            let parts: Vec<&str> = suffix.split('-').collect();
            match parts.len() {
                1 => {
                    if let Ok(d) = parts[0].parse::<u64>() {
                        config.in_delay = Some(d);
                    } else {
                        config.in_delay = Some(0);
                    }
                    config.fade_in = Some(DEFAULT_FADE_IN_MS);
                }
                2 => {
                    let d = parts[0].parse::<u64>().unwrap_or(0);
                    let t = parts[1].parse::<u64>().unwrap_or(DEFAULT_FADE_IN_MS);
                    config.in_delay = Some(d);
                    config.fade_in = Some(t);
                }
                _ => {}
            }
        }

        // 2. "out" or "out-{d}" or "out-{d}-{t}"
        if class == "out" {
            config.out_delay = Some(0);
            config.fade_out = Some(DEFAULT_FADE_OUT_MS);
        } else if let Some(suffix) = class.strip_prefix("out-") {
            let parts: Vec<&str> = suffix.split('-').collect();
            match parts.len() {
                1 => {
                    if let Ok(d) = parts[0].parse::<u64>() {
                        config.out_delay = Some(d);
                    } else {
                        config.out_delay = Some(0);
                    }
                    config.fade_out = Some(DEFAULT_FADE_OUT_MS);
                }
                2 => {
                    let d = parts[0].parse::<u64>().unwrap_or(0);
                    let t = parts[1].parse::<u64>().unwrap_or(DEFAULT_FADE_OUT_MS);
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
#[derive(Debug, Clone, Default)]
pub struct TransitionManager {
    /// Tracks `(PageId, PageKey)` and the last rendered content hash to detect content changes.
    seen_page_items: HashMap<(PageId, PageKey), u64>,
    /// Currently active page ID.
    current_page: Option<PageId>,
}

impl TransitionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called whenever the active view changes.
    ///
    /// When entering a fresh page (`is_fresh == true`), seen items for that page
    /// are cleared so entering the page plays the initial entrance animations.
    /// During subsequent iterations on the same page (`is_fresh == false`), seen items
    /// are preserved so re-renders do not re-trigger animations.
    pub fn on_view_change(&mut self, new_page_id: &PageId, is_fresh: bool) {
        if is_fresh || self.current_page.as_ref() != Some(new_page_id) {
            // Remove any cached entries for this page so fresh visits animate correctly
            self.seen_page_items.retain(|(p, _), _| p != new_page_id);
            self.current_page = Some(new_page_id.clone());
        }
    }

    /// Evaluates whether an element should animate based on its transition config and content hash.
    ///
    /// If the item was already seen with the exact same hash, returns `false`.
    /// If newly seen OR its hash changed (e.g. `alts!` clicked to the next variant), returns `true`.
    pub fn should_animate_item(
        &mut self,
        page_id: &PageId,
        id: Option<PageKey>,
        hash: u64,
        config: &TransitionConfig,
    ) -> bool {
        if !config.has_transition() {
            return false;
        }

        self.is_content_changed(page_id, id, hash)
    }

    /// Checks whether an item's content changed or is newly seen on this page based on its hash.
    ///
    /// A hash of `0` is used for choices (always reflows initially).
    /// Lines and spans hash to non-zero values.
    /// If newly seen or content hash changed: returns `true` (or `false` if initial hash is 0).
    /// If previously seen with the exact same hash: returns `false`.
    pub fn is_content_changed(
        &mut self,
        page_id: &PageId,
        id: Option<PageKey>,
        hash: u64,
    ) -> bool {
        let Some(key) = id else {
            return false;
        };

        let item_key = (page_id.clone(), key);
        if let Some(&prev_hash) = self.seen_page_items.get(&item_key) {
            if prev_hash == hash {
                false
            } else {
                self.seen_page_items.insert(item_key, hash);
                true
            }
        } else {
            self.seen_page_items.insert(item_key, hash);
            hash != 0
        }
    }
}

/// Generates the inline CSS styles and class list for an element based on its transition config.
///
/// If `should_animate` is `false`, the element was already seen in a prior iteration,
/// so we suppress animation (`skip-animation` / `animation: none`) to prevent visual glitches.
pub fn generate_transition_style(
    config: &TransitionConfig,
    should_animate: bool,
) -> (String, Vec<&'static str>) {
    if !config.has_transition() {
        return (String::new(), vec![]);
    }

    if !should_animate {
        // Element was already seen in a prior iteration: keep it visible or hidden and suppress re-trigger
        if config.fade_out.is_some() {
            (
                "opacity: 0; animation: none !important;".to_string(),
                vec!["skip-animation"],
            )
        } else {
            (
                "opacity: 1; animation: none !important;".to_string(),
                vec!["skip-animation"],
            )
        }
    } else {
        if config.fade_in.is_some() && config.fade_out.is_some() {
            let in_delay = config.in_delay.unwrap_or(0);
            let in_duration = config.fade_in.unwrap_or(DEFAULT_FADE_IN_MS);
            let out_delay = config.out_delay.unwrap_or(0);
            let out_duration = config.fade_out.unwrap_or(DEFAULT_FADE_OUT_MS);
            let start_out = in_delay + in_duration + out_delay;

            let style = format!(
                "animation: fade-in {in_duration}ms ease-out {in_delay}ms both, fade-out {out_duration}ms ease-out {start_out}ms forwards;"
            );
            (style, vec!["transition-active"])
        } else if let Some(fade_out_ms) = config.fade_out {
            let out_delay = config.out_delay.unwrap_or(0);
            let style = format!(
                "animation-name: fade-out; animation-duration: {fade_out_ms}ms; animation-delay: {out_delay}ms; animation-fill-mode: both; animation-timing-function: ease-out;"
            );
            (style, vec!["transition-active"])
        } else {
            let in_duration = config.fade_in.unwrap_or(DEFAULT_FADE_IN_MS);
            let in_delay = config.in_delay.unwrap_or(0);
            let style = format!(
                "animation-name: fade-in; animation-duration: {in_duration}ms; animation-delay: {in_delay}ms; animation-fill-mode: both; animation-timing-function: ease-out;"
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
        assert_eq!(c.fade_in, Some(DEFAULT_FADE_IN_MS));

        let c = parse_transition_classes(&["out".to_string()]);
        assert_eq!(c.out_delay, Some(0));
        assert_eq!(c.fade_out, Some(DEFAULT_FADE_OUT_MS));
    }

    #[test]
    fn test_parse_in_and_out_timing() {
        // "in-200": delay 200, duration default
        let c = parse_transition_classes(&["in-200".to_string()]);
        assert_eq!(c.in_delay, Some(200));
        assert_eq!(c.fade_in, Some(DEFAULT_FADE_IN_MS));

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
        assert_eq!(c.fade_out, Some(DEFAULT_FADE_OUT_MS));

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
        let mut tm = TransitionManager::new();
        let page = PageId("page_1".into());
        let item_id = 12345u64;
        let config = parse_transition_classes(&["in-200".to_string()]);

        // First view change to page_1 (fresh)
        tm.on_view_change(&page, true);

        // Iteration 1: should animate with hash 100
        assert!(tm.should_animate_item(&page, Some(item_id), 100, &config));

        // Iteration 2 (same page, user clicks another choice, content hash unchanged)
        tm.on_view_change(&page, false);

        // Content unchanged: MUST NOT ANIMATE!
        assert!(!tm.should_animate_item(&page, Some(item_id), 100, &config));

        // Iteration 2b: user clicks this alt! Content hash changes to 200: MUST ANIMATE!
        assert!(tm.should_animate_item(&page, Some(item_id), 200, &config));

        // Subsequent render with content hash 200: MUST NOT ANIMATE!
        assert!(!tm.should_animate_item(&page, Some(item_id), 200, &config));

        // Items without ID do not animate
        assert!(!tm.should_animate_item(&page, None, 300, &config));

        // Now user navigates to fresh page_2
        let page_2 = PageId("page_2".into());
        tm.on_view_change(&page_2, true);
        assert!(tm.should_animate_item(&page_2, Some(item_id), 100, &config));

        // User navigates back to page_1 (fresh = true)
        tm.on_view_change(&page, true);
        // On a fresh visit to page_1, entrance animations trigger again
        assert!(tm.should_animate_item(&page, Some(item_id), 100, &config));
    }
}
