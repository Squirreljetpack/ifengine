//! Animation and transition management for `ifengine_leptos`.
//!
//! Parses authored `in`/`out` timing classes, tracks content-change state across
//! iterations, and exposes the plan/runtime/backend pipeline that effects same-page
//! transitions. The View Transitions pseudo-element vocabulary lives only in the
//! `vt` backend; components emit backend-agnostic `data-vt-*` attributes.

mod backend;
mod css;
mod plan;
mod runtime;
mod scheduler;
mod vt;

pub use backend::TransitionBackend;
pub use plan::ItemKind;
pub use runtime::{ItemKey, TransitionHandle, TransitionRuntime};
pub(crate) use scheduler::read_css_time_ms;
pub use vt::ensure_duration_styles;

use std::collections::HashMap;

use ifengine::{PageKey, core::PageId};
use leptos::prelude::*;

/// Parsed animation transition configuration for an individual element (Span or Line).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransitionConfig {
    /// Explicit fade-in duration in milliseconds (`in-{d}-{t}`); `None` uses the CSS default.
    pub fade_in: Option<u64>,
    /// Explicit fade-out duration in milliseconds (`out-{d}-{t}`); `None` uses the CSS default.
    pub fade_out: Option<u64>,
    /// Whether an entrance class (`in`/`in-*`) is present.
    pub has_in: bool,
    /// Whether an exit class (`out`/`out-*`) is present.
    pub has_out: bool,
    /// Delay in milliseconds before fade-in commences.
    pub in_delay: Option<u64>,
    /// Delay in milliseconds before fading back out (after fully fading in).
    pub out_delay: Option<u64>,
}

impl TransitionConfig {
    /// Returns true if any animation transition is configured.
    pub fn has_transition(&self) -> bool {
        self.has_in || self.has_out
    }
}

/// Parses the class list of a `Span` or `Line` to extract transition timing parameters.
///
/// Supported patterns:
/// - `"in"` -> fade in with the CSS default duration (0 delay)
/// - `"in-{d}"` -> fade in after `{d}` ms with the CSS default duration
/// - `"in-{d}-{t}"` -> fade in after `{d}` ms with an explicit `{t}` ms duration
/// - `"out"` / `"out-{d}"` / `"out-{d}-{t}"` -> parsed for compatibility
pub fn parse_transition_classes(classes: &[String]) -> TransitionConfig {
    let mut config = TransitionConfig::default();

    for class in classes {
        let class = class.trim();

        // 1. "in" or "in-{d}" or "in-{d}-{t}"
        if class == "in" {
            config.has_in = true;
            config.in_delay = Some(0);
        } else if let Some(suffix) = class.strip_prefix("in-") {
            let parts: Vec<&str> = suffix.split('-').collect();
            match parts.len() {
                1 => {
                    config.has_in = true;
                    config.in_delay = Some(parts[0].parse::<u64>().unwrap_or(0));
                }
                2 => {
                    config.has_in = true;
                    config.in_delay = Some(parts[0].parse::<u64>().unwrap_or(0));
                    config.fade_in = parts[1].parse::<u64>().ok();
                }
                _ => {}
            }
        }

        // 2. "out" or "out-{d}" or "out-{d}-{t}"
        if class == "out" {
            config.has_out = true;
            config.out_delay = Some(0);
        } else if let Some(suffix) = class.strip_prefix("out-") {
            let parts: Vec<&str> = suffix.split('-').collect();
            match parts.len() {
                1 => {
                    config.has_out = true;
                    config.out_delay = Some(parts[0].parse::<u64>().unwrap_or(0));
                }
                2 => {
                    config.has_out = true;
                    config.out_delay = Some(parts[0].parse::<u64>().unwrap_or(0));
                    config.fade_out = parts[1].parse::<u64>().ok();
                }
                _ => {}
            }
        }
    }

    config
}

/// Returns true if the line itself or all its non-whitespace spans have an entrance delay (`in_delay > 0`).
pub fn is_line_delayed(line: &ifengine::view::Line) -> bool {
    let config = parse_transition_classes(&line.classes);
    if config.in_delay.unwrap_or(0) > 0 {
        return true;
    }
    let non_empty_spans: Vec<&ifengine::view::Span> = line
        .spans
        .iter()
        .filter(|s| !s.content.trim().is_empty())
        .collect();
    if !non_empty_spans.is_empty()
        && non_empty_spans.iter().all(|s| {
            let cfg = parse_transition_classes(&s.classes);
            cfg.in_delay.unwrap_or(0) > 0
        })
    {
        return true;
    }
    false
}

/// Computes the effective entrance delay in milliseconds for a line.
pub fn line_in_delay(line: &ifengine::view::Line) -> u64 {
    let config = parse_transition_classes(&line.classes);
    if let Some(d) = config.in_delay
        && d > 0
    {
        return d;
    }
    line.spans
        .iter()
        .filter_map(|s| {
            let cfg = parse_transition_classes(&s.classes);
            cfg.in_delay
        })
        .max()
        .unwrap_or(0)
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
}

impl TransitionManager {
    /// Creates a new `TransitionManager` initialized with the initial story page ID.
    pub fn new(initial_page_id: PageId) -> Self {
        Self {
            seen_page_items: HashMap::new(),
            current_page: initial_page_id,
        }
    }

    /// Called whenever the active view changes.
    ///
    /// Seen items are indexed by `(PageId, PageKey)`, ensuring entrance animations and delays
    /// do not re-trigger when revisiting previously seen passages unless content changes.
    pub fn on_view_change(&mut self, new_page_id: &PageId) {
        self.current_page = new_page_id.clone();
    }

    /// Returns true if any items have been recorded for the current page.
    pub fn has_seen_page(&self) -> bool {
        self.seen_page_items.keys().any(|(page, _)| page == &self.current_page)
    }

    /// Returns true if `id` was already recorded on the current page.
    pub fn was_seen(&self, id: Option<PageKey>) -> bool {
        let Some(key) = id else {
            return false;
        };
        self.seen_page_items.contains_key(&(self.current_page.clone(), key))
    }

    /// Checks whether an item's content changed or is newly seen on the current page based on its hash.
    ///
    /// On fresh page load (`is_fresh == true`), returns `false` so entrance animations are suppressed.
    /// On same-page interactions (`is_fresh == false`), newly seen or content-changed items return `true`.
    /// If previously seen with the exact same hash: returns `false`.
    #[allow(dead_code)]
    pub fn is_content_changed(&mut self, id: Option<PageKey>, hash: u64) -> bool {
        self.is_content_changed_delayed(id, hash, false)
    }

    /// Checks whether an item's content changed, deferring marking it seen if `is_delayed` is true
    /// and `ADD_DELAYED_TO_SEEN_ON_TRANSITION_START` is enabled.
    pub fn is_content_changed_delayed(
        &mut self,
        id: Option<PageKey>,
        hash: u64,
        is_delayed: bool,
    ) -> bool {
        let Some(key) = id else {
            return false;
        };

        let item_key = (self.current_page.clone(), key);
        let should_defer_seen =
            crate::consts::ADD_DELAYED_TO_SEEN_ON_TRANSITION_START && is_delayed;

        if let Some(&prev_hash) = self.seen_page_items.get(&item_key) {
            if prev_hash == hash {
                false
            } else {
                if !should_defer_seen {
                    self.seen_page_items.insert(item_key, hash);
                }
                true
            }
        } else {
            if !should_defer_seen {
                self.seen_page_items.insert(item_key, hash);
            }
            hash != 0
        }
    }

    /// Explicitly records an item as seen on the current page once its transition begins to run.
    pub fn mark_seen(&mut self, id: Option<PageKey>, hash: u64) {
        let Some(key) = id else {
            return;
        };
        self.seen_page_items
            .insert((self.current_page.clone(), key), hash);
    }
}

/// Lifecycle phase for elements with delayed entrance or exit transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    /// Before entrance delay expires: element must not take up any space (display: none).
    Pending,
    /// Actively animating or visible.
    Active,
    /// After exit delay expires: fading out in place.
    Removing,
    /// Fade finished: element is hidden and takes up no layout.
    Removed,
}

/// Determines the initial transition phase for an element.
pub fn compute_initial_phase(config: &TransitionConfig, should_animate: bool) -> TransitionPhase {
    if !config.has_transition() || !should_animate {
        TransitionPhase::Active
    } else {
        let in_delay = config.in_delay.unwrap_or(0);
        if in_delay > 0 {
            TransitionPhase::Pending
        } else {
            TransitionPhase::Active
        }
    }
}

/// Schedules timeouts to transition an element:
/// - from `Pending` -> `Active` (after in_delay).
/// - from `Active` -> `Removed` (after out_delay, measured from start/completion).
pub fn setup_transition_timers(
    config: &TransitionConfig,
    transitions: &TransitionHandle,
    set_phase: leptos::prelude::WriteSignal<TransitionPhase>,
) {
    setup_transition_timers_keyed(config, transitions, None, set_phase);
}

/// Schedules timeouts to transition an element with optional key registration on entrance start:
/// - from `Pending` -> `Active` (after in_delay, marking the item seen).
/// - from `Active` -> `Removed` (after out_delay, measured from start/completion).
pub fn setup_transition_timers_keyed(
    config: &TransitionConfig,
    transitions: &TransitionHandle,
    item: Option<(PageKey, u64)>,
    set_phase: leptos::prelude::WriteSignal<TransitionPhase>,
) {
    let in_delay = config.in_delay.unwrap_or(0);
    if in_delay > 0 {
        let transitions = *transitions;
        transitions.schedule(in_delay, move || {
            if let Some((id, hash)) = item {
                transitions.mark_seen(Some(id), hash);
            }
            set_phase.set(TransitionPhase::Active);
        });
    }

    if let Some(out_d) = config.out_delay {
        let fade_out_ms = config
            .fade_out
            .unwrap_or_else(|| read_css_time_ms("--default-fade-out", 1500));
        transitions.schedule(out_d, move || {
            set_phase.set(TransitionPhase::Removing);
            leptos::prelude::set_timeout(
                move || set_phase.set(TransitionPhase::Removed),
                std::time::Duration::from_millis(fade_out_ms),
            );
        });
    }
}

/// Generates active transition styles when an element is in the `Active` phase.
pub fn generate_active_transition_style(config: &TransitionConfig, should_animate: bool) -> String {
    if !config.has_transition() {
        return String::new();
    }

    if !should_animate {
        String::new()
    } else {
        "opacity: 1;".to_string()
    }
}

/// Whether a class token declares an entrance or exit animation.
pub fn is_transition_token(class: &str) -> bool {
    let class = class.trim();
    class == "in" || class.starts_with("in-") || class == "out" || class.starts_with("out-")
}

/// Authored `in-*`/`out-*` tokens for an element: its own, or the inherited line's when it has none.
pub fn effective_transition_classes(classes: &[String], inherited: &[String]) -> Vec<String> {
    let mut tokens: Vec<String> = classes
        .iter()
        .filter(|class| is_transition_token(class))
        .map(|class| class.trim().to_string())
        .collect();
    if tokens.is_empty() {
        tokens.extend(
            inherited
                .iter()
                .filter(|class| is_transition_token(class))
                .map(|class| class.trim().to_string()),
        );
    }
    tokens
}

/// Parent key for auto-generated span ids inside a choice arm.
pub fn choice_item_parent_id(key: PageKey, index: u8) -> PageKey {
    (key.rotate_left(16) ^ ((index as u64) + 1)) | ifengine::core::key::HASH_KEY_BIT
}

/// Current phase of a full-page transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageTransitionPhase {
    #[default]
    None,
    Leaving,
    Entering,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_defaults() {
        let c = parse_transition_classes(&["in".to_string()]);
        assert_eq!(c.in_delay, Some(0));
        assert_eq!(c.fade_in, None);
        assert!(c.has_in);

        let c = parse_transition_classes(&["out".to_string()]);
        assert_eq!(c.out_delay, Some(0));
        assert_eq!(c.fade_out, None);
        assert!(c.has_out);
    }

    #[test]
    fn test_parse_in_and_out_timing() {
        // "in-200": delay 200, CSS-default duration
        let c = parse_transition_classes(&["in-200".to_string()]);
        assert_eq!(c.in_delay, Some(200));
        assert_eq!(c.fade_in, None);

        // "in-200-500": delay 200, duration 500
        let c = parse_transition_classes(&["in-200-500".to_string()]);
        assert_eq!(c.in_delay, Some(200));
        assert_eq!(c.fade_in, Some(500));

        // "in-0-500": delay 0, duration 500
        let c = parse_transition_classes(&["in-0-500".to_string()]);
        assert_eq!(c.in_delay, Some(0));
        assert_eq!(c.fade_in, Some(500));

        // "out-2000": delay 2000, CSS-default duration
        let c = parse_transition_classes(&["out-2000".to_string()]);
        assert_eq!(c.out_delay, Some(2000));
        assert_eq!(c.fade_out, None);

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
    fn test_effective_transition_classes() {
        let inherited = vec!["in-500".to_string(), "plain".to_string()];
        assert_eq!(
            effective_transition_classes(
                &["in-500".to_string(), "out-300".to_string()],
                &inherited
            ),
            vec!["in-500".to_string(), "out-300".to_string()]
        );
        assert_eq!(
            effective_transition_classes(&["plain".to_string()], &inherited),
            vec!["in-500".to_string()]
        );
        assert_eq!(
            effective_transition_classes(&["in".to_string()], &["in-500".to_string()]),
            vec!["in".to_string()]
        );
        assert!(is_transition_token("out-1000-500"));
        assert!(!is_transition_token("inside"));
    }

    #[test]
    fn test_anti_retrigger_across_iterations() {
        let page = PageId("page_1".into());
        let mut tm = TransitionManager::new(page.clone());
        let item_id = 12345u64;

        // Iteration 1: On initial load, newly seen item with non-zero hash animates!
        assert!(tm.is_content_changed(Some(item_id), 100));

        // Iteration 2 (same page, user clicks another choice, content hash unchanged)
        tm.on_view_change(&page);

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
        tm.on_view_change(&page_2);
        // On new page load, item with id 12345 is new to page_2, so it animates
        assert!(tm.is_content_changed(Some(item_id), 100));

        // Same-page interaction on page_2: newly revealed item DOES animate
        tm.on_view_change(&page_2);
        let new_item_id = 99999u64;
        assert!(tm.is_content_changed(Some(new_item_id), 500));

        // User navigates back to page_1
        tm.on_view_change(&page);
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

        // generate_active_transition_style
        assert_eq!(
            generate_active_transition_style(&config_delayed, true),
            "opacity: 1;"
        );
        assert!(generate_active_transition_style(&config_immediate, false).is_empty());
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

        // Choice arms without line id are NOT registered (pass None)
        let arm_changed = tm.is_content_changed(None, 12345);
        assert!(!arm_changed);

        // Iteration 2: Same-page interaction (e.g. alt-1/23 clicked)
        tm.on_view_change(&page);

        // Choice re-renders with hash 0 -> MUST STILL BE REFLOW (not fade!)
        let is_changed_after_alt = tm.is_content_changed(Some(choice_key), 0);
        assert!(
            !is_changed_after_alt,
            "choice should remain reflow across alt interactions"
        );

        // Iteration 3: User selects choice! It morphs into a paragraph sharing choice_key,
        // so the paragraph registers a non-zero hash against the recorded 0.
        tm.on_view_change(&page);
        let is_changed_on_selection = tm.is_content_changed(Some(choice_key), 1);
        assert!(
            is_changed_on_selection,
            "morphing from choice (hash 0) to paragraph should be marked changed (fade)"
        );

        // Iteration 4: Another same-page interaction after paragraph is already chosen
        tm.on_view_change(&page);
        let is_changed_later = tm.is_content_changed(Some(choice_key), 1);
        assert!(
            !is_changed_later,
            "paragraph with same content should now reflow"
        );

        // Iteration 5: Inline click expands the paragraph
        tm.on_view_change(&page);
        let is_changed_inline = tm.is_content_changed(Some(choice_key), 1);
        assert!(
            !is_changed_inline,
            "already-visible paragraph with changed content must remain reflow, not fade"
        );
    }

    #[test]
    fn test_delayed_elements_added_to_seen_only_when_running() {
        let page = PageId("test_page".into());
        let mut tm = TransitionManager::new(page.clone());
        let delayed_item_id = 999u64;

        // 1. Initial plan build for an item with delay (is_delayed = true)
        let is_changed = tm.is_content_changed_delayed(Some(delayed_item_id), 1234, true);
        assert!(is_changed, "delayed item should be marked changed/entering");

        // 2. Delayed item must NOT be in seen yet before transition begins running!
        assert!(
            !tm.was_seen(Some(delayed_item_id)),
            "delayed item must not be added to seen before transition runs"
        );

        // 3. Subsequent query before transition runs still considers it not seen
        let still_changed = tm.is_content_changed_delayed(Some(delayed_item_id), 1234, true);
        assert!(still_changed);
        assert!(!tm.was_seen(Some(delayed_item_id)));

        // 4. Transition begins to run: item is explicitly marked seen!
        tm.mark_seen(Some(delayed_item_id), 1234);
        assert!(
            tm.was_seen(Some(delayed_item_id)),
            "item must be marked seen once transition begins"
        );

        // 5. Subsequent queries now recognize it as seen and unchanged:
        assert!(!tm.is_content_changed_delayed(Some(delayed_item_id), 1234, false));
    }
}
