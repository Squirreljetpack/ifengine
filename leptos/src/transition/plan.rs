//! Transition plan construction.
//!
//! A plan is built once per rendered view, walking it in render order. For every
//! element the frontend can name, the plan records identity, content hash, authored
//! timing and a geometry kind. Its `changed` flag marks first sight only: backends
//! never parse authored transition classes and never trust that flag, they compare
//! the live DOM against their own pre-update snapshot using the plan's hash and
//! pixel tolerance, so an entry that merely moved does not flash.

use std::collections::{HashMap, HashSet};

use ifengine::PageKey;
use ifengine::view::{Line, Object, Span, StampedObject, View};
use serde::Serialize;

use super::scheduler::read_css_time_ms;
use super::{
    TransitionManager, choice_item_parent_id, effective_transition_classes,
    parse_transition_classes,
};
use crate::consts::{
    AUTO_SPAN_TRANSITIONS, REFLOW_TOLERANCE_PX, STRETCH_SECONDARY_TO_AUTHORED_FADE,
};

/// Hash-based classification for an item, before any geometry upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    Fade,
    Reflow,
}

impl Effect {
    pub fn from_changed(changed: bool) -> Self {
        if changed { Self::Fade } else { Self::Reflow }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fade => "fade",
            Self::Reflow => "reflow",
        }
    }
}

/// The geometry family of a named element, used by the clone backend to decide
/// between sliding (block-level images) and fading (inline text, spacers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    /// Inline content (spans).
    Span,
    /// Block-level image wrapper: transform FLIP is safe.
    Image,
    /// Contentless spacer (`<hr>`, empty): never animated.
    Spacer,
    /// Tracked container without a `view-transition-name` (paragraphs, choices, embeds).
    Block,
}

impl ItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Span => "span",
            Self::Image => "image",
            Self::Spacer => "spacer",
            Self::Block => "block",
        }
    }
}

/// Authored timing resolved from `in-*` / `out-*` classes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timing {
    pub in_delay: u64,
    pub fade_in: Option<u64>,
    pub out_delay: Option<u64>,
    pub fade_out: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeDirection {
    In,
    Out,
}

/// One `::view-transition-{new,old}(.{token})` duration override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationRule {
    pub token: String,
    pub direction: FadeDirection,
    pub duration_ms: u64,
}

/// A single named or tracked element in a rendered view.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPlan {
    /// Identity key; only used for Rust-side lookup.
    #[serde(skip)]
    pub id: PageKey,
    /// `view-transition-name` value, absent for tracked containers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Content hash; serialized as a string so JS never loses precision.
    #[serde(serialize_with = "serialize_hash")]
    pub hash: u64,
    pub changed: bool,
    pub effect: Effect,
    pub kind: ItemKind,
    /// Authored `in-*` / `out-*` tokens (only carried by changed items in VT classes).
    pub tokens: Vec<String>,
    pub timing: Timing,
    /// Effective fade-in duration when the item carries an `in*` token.
    pub fade_in_ms: Option<u64>,
    /// Effective fade-out duration when the item carries an `out*` token.
    pub fade_out_ms: Option<u64>,
    #[serde(default)]
    pub is_new_block: bool,
}

fn serialize_hash<S: serde::Serializer>(hash: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&hash.to_string())
}

/// A complete transition plan for one rendered view.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionPlan {
    pub items: Vec<ItemPlan>,
    pub tolerance_px: f64,
    pub secondary_reflow_ms: u64,
    pub primary_in_delay: u64,
    pub primary_block_in_delay: u64,
    pub stretch_secondary: bool,
    #[serde(skip)]
    pub duration_rules: Vec<DurationRule>,
    #[serde(skip)]
    lookup: HashMap<PageKey, usize>,
}

impl Default for TransitionPlan {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            tolerance_px: REFLOW_TOLERANCE_PX,
            secondary_reflow_ms: 1500,
            primary_in_delay: crate::consts::PRIMARY_IN_DELAY_MS,
            primary_block_in_delay: crate::consts::PRIMARY_BLOCK_IN_DELAY_MS,
            stretch_secondary: STRETCH_SECONDARY_TO_AUTHORED_FADE,
            duration_rules: Vec::new(),
            lookup: HashMap::new(),
        }
    }
}

impl TransitionPlan {
    /// Returns the plan entry for `id` when its hash matches the caller's.
    pub fn entry(&self, id: PageKey, hash: u64) -> Option<&ItemPlan> {
        let &index = self.lookup.get(&id)?;
        let entry = &self.items[index];
        (entry.hash == hash).then_some(entry)
    }
}

/// CSS-owned fallback durations, read once per plan build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlanDefaults {
    pub fade_in: u64,
    pub fade_out: u64,
    pub secondary_reflow_ms: u64,
    pub primary_in_delay: u64,
    pub primary_block_in_delay: u64,
}

impl Default for PlanDefaults {
    fn default() -> Self {
        Self {
            fade_in: 1500,
            fade_out: 1500,
            secondary_reflow_ms: 1500,
            primary_in_delay: crate::consts::PRIMARY_IN_DELAY_MS,
            primary_block_in_delay: crate::consts::PRIMARY_BLOCK_IN_DELAY_MS,
        }
    }
}

impl PlanDefaults {
    pub(crate) fn from_css() -> Self {
        Self {
            fade_in: read_css_time_ms("--default-fade-in", 1500),
            fade_out: read_css_time_ms("--default-fade-out", 1500),
            secondary_reflow_ms: read_css_time_ms("--secondary-reflow-duration", 1500),
            primary_in_delay: read_css_time_ms(
                "--primary-in-delay",
                crate::consts::PRIMARY_IN_DELAY_MS,
            ),
            primary_block_in_delay: read_css_time_ms(
                "--primary-block-in-delay",
                crate::consts::PRIMARY_BLOCK_IN_DELAY_MS,
            ),
        }
    }
}

/// Builds a plan for `view`, recording content-change state in `manager`.
pub(crate) fn build(view: &View, manager: &mut TransitionManager) -> TransitionPlan {
    build_with(view, manager, PlanDefaults::from_css())
}

/// Builds a plan with caller-supplied defaults (unit-testable without a DOM).
pub(crate) fn build_with(
    view: &View,
    manager: &mut TransitionManager,
    defaults: PlanDefaults,
) -> TransitionPlan {
    let page_seen_before = manager.has_seen_page();
    let mut builder = Builder {
        manager,
        items: Vec::new(),
        lookup: HashMap::new(),
        classes: HashSet::new(),
        defaults,
        page_seen_before,
    };
    builder.view(view);
    let duration_rules = builder.duration_rules();

    TransitionPlan {
        items: builder.items,
        tolerance_px: REFLOW_TOLERANCE_PX,
        secondary_reflow_ms: defaults.secondary_reflow_ms,
        primary_in_delay: defaults.primary_in_delay,
        primary_block_in_delay: defaults.primary_block_in_delay,
        stretch_secondary: STRETCH_SECONDARY_TO_AUTHORED_FADE,
        duration_rules,
        lookup: builder.lookup,
    }
}

struct Builder<'a> {
    manager: &'a mut TransitionManager,
    items: Vec<ItemPlan>,
    lookup: HashMap<PageKey, usize>,
    classes: HashSet<String>,
    defaults: PlanDefaults,
    page_seen_before: bool,
}

impl Builder<'_> {
    fn view(&mut self, view: &View) {
        for stamped in &view.inner {
            self.object(stamped);
        }
    }

    fn object(&mut self, stamped: &StampedObject) {
        let is_block_delayed = match &stamped.object {
            Object::Paragraph(line, _) | Object::Note(line, _) => super::is_line_delayed(line),
            Object::Choice(choices) => {
                !choices.is_empty() && choices.iter().all(|(_, l)| super::is_line_delayed(l))
            }
            Object::Heading(span, _) => {
                parse_transition_classes(&span.classes).in_delay.unwrap_or(0) > 0
            }
            _ => false,
        };
        let is_new_block = self.page_seen_before && !self.manager.was_seen(stamped.id);
        // Mirrors `ObjectView`'s initial content-change query for the object itself.
        let changed = self
            .manager
            .is_content_changed_delayed(stamped.id, stamped.content_hash(), is_block_delayed);

        match &stamped.object {
            Object::Paragraph(line, _) => {
                self.track(stamped.id, stamped.content_hash(), changed);
                self.line(line, stamped.id, is_new_block, is_block_delayed);
            }
            Object::Choice(choices) => {
                let key = stamped.id.unwrap_or(0);
                self.track(stamped.id, 0, changed);
                for (index, line) in choices {
                    self.line(
                        line,
                        Some(choice_item_parent_id(key, *index)),
                        is_new_block,
                        is_block_delayed,
                    );
                }
            }
            Object::Image(_) => {
                self.named(
                    stamped.id,
                    stamped.content_hash(),
                    changed,
                    ItemKind::Image,
                    vec![],
                );
            }
            Object::Heading(span, _) => {
                self.track(stamped.id, stamped.content_hash(), changed);
                self.span(span, &[], is_new_block, is_block_delayed);
            }
            Object::Break | Object::Empty(_) => {
                self.named(
                    stamped.id,
                    stamped.content_hash(),
                    changed,
                    ItemKind::Spacer,
                    vec![],
                );
            }
            Object::Note(line, _) => {
                self.track(stamped.id, stamped.content_hash(), changed);
                self.line(line, stamped.id, is_new_block, is_block_delayed);
            }
            Object::Embed(view, _) => {
                self.track(stamped.id, stamped.content_hash(), changed);
                self.view(view);
            }
        }
    }

    fn line(
        &mut self,
        line: &Line,
        parent_id: Option<PageKey>,
        is_new_block: bool,
        is_block_delayed: bool,
    ) {
        self.collect_classes(&line.classes);
        let parent = parent_id.unwrap_or(0);
        let mut spans = line.spans.clone();
        if AUTO_SPAN_TRANSITIONS {
            crate::components::line::assign_auto_keys(&mut spans, parent);
        }
        for span in spans {
            if span.content.is_empty() && !span.active() {
                continue;
            }
            self.span(&span, &line.classes, is_new_block, is_block_delayed);
        }
    }

    fn span(
        &mut self,
        span: &Span,
        line_classes: &[String],
        is_new_block: bool,
        is_block_delayed: bool,
    ) {
        self.collect_classes(&span.classes);
        let Some(id) = span.id else { return };
        let hash = span.content_hash();
        let tokens = effective_transition_classes(&span.classes, line_classes);
        let config = parse_transition_classes(&tokens);
        let is_delayed = is_block_delayed || config.in_delay.unwrap_or(0) > 0;
        let changed = self
            .manager
            .is_content_changed_delayed(Some(id), hash, is_delayed);

        let in_delay = match config.in_delay {
            Some(d) => d,
            None if is_new_block => self.defaults.primary_block_in_delay,
            None => 0,
        };

        let timing = Timing {
            in_delay,
            fade_in: config.fade_in,
            out_delay: config.out_delay,
            fade_out: config.fade_out,
        };
        let (fade_in_ms, fade_out_ms) = resolve_fades(&config, self.defaults);
        self.push(ItemPlan {
            id,
            name: Some(format!("item-{id}")),
            hash,
            changed,
            effect: Effect::from_changed(changed),
            kind: ItemKind::Span,
            tokens,
            timing,
            fade_in_ms,
            fade_out_ms,
            is_new_block,
        });
    }

    fn named(
        &mut self,
        id: Option<PageKey>,
        hash: u64,
        changed: bool,
        kind: ItemKind,
        tokens: Vec<String>,
    ) {
        let Some(id) = id else { return };
        self.push(ItemPlan {
            id,
            name: Some(format!("item-{id}")),
            hash,
            changed,
            effect: Effect::from_changed(changed),
            kind,
            tokens,
            timing: Timing::default(),
            fade_in_ms: None,
            fade_out_ms: None,
            is_new_block: false,
        });
    }

    fn track(&mut self, id: Option<PageKey>, hash: u64, changed: bool) {
        let Some(id) = id else { return };
        self.push(ItemPlan {
            id,
            name: None,
            hash,
            changed,
            effect: Effect::from_changed(changed),
            kind: ItemKind::Block,
            tokens: vec![],
            timing: Timing::default(),
            fade_in_ms: None,
            fade_out_ms: None,
            is_new_block: false,
        });
    }

    fn push(&mut self, item: ItemPlan) {
        if let Some(&index) = self.lookup.get(&item.id) {
            self.items[index] = item;
        } else {
            self.lookup.insert(item.id, self.items.len());
            self.items.push(item);
        }
    }

    fn collect_classes(&mut self, classes: &[String]) {
        for class in classes {
            if super::is_transition_token(class) {
                self.classes.insert(class.trim().to_string());
            }
        }
    }

    fn duration_rules(&self) -> Vec<DurationRule> {
        let mut rules = Vec::new();
        for token in &self.classes {
            let config = parse_transition_classes(std::slice::from_ref(token));
            if let Some(duration_ms) = config.fade_in {
                rules.push(DurationRule {
                    token: token.clone(),
                    direction: FadeDirection::In,
                    duration_ms,
                });
            }
            if let Some(duration_ms) = config.fade_out {
                rules.push(DurationRule {
                    token: token.clone(),
                    direction: FadeDirection::Out,
                    duration_ms,
                });
            }
        }
        rules
    }
}

fn resolve_fades(
    config: &super::TransitionConfig,
    defaults: PlanDefaults,
) -> (Option<u64>, Option<u64>) {
    (
        config
            .has_in
            .then(|| config.fade_in.unwrap_or(defaults.fade_in)),
        config
            .has_out
            .then(|| config.fade_out.unwrap_or(defaults.fade_out)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ifengine::core::PageId;
    use ifengine::view::{Line, Object, Span, StampedObject, View};

    fn line_with(spans: Vec<Span>, classes: &[&str]) -> Line {
        Line {
            spans,
            classes: classes.iter().map(|c| c.to_string()).collect(),
        }
    }

    fn view_of(objects: Vec<StampedObject>) -> View {
        let mut view = View::new(PageId("page".into()));
        view.inner = objects;
        view
    }

    fn manager() -> TransitionManager {
        TransitionManager::new(PageId("page".into()))
    }

    fn plan_of(view: &View, manager: &mut TransitionManager) -> TransitionPlan {
        build_with(view, manager, PlanDefaults::default())
    }

    #[test]
    fn heading_spans_and_spacers_are_planned() {
        let view = view_of(vec![
            StampedObject::new(Object::Heading(Span::new("title").with_id(9), 1)).with_id(99),
            StampedObject::new(Object::Break).with_id(1),
            StampedObject::new(Object::Empty(2)).with_id(2),
        ]);
        let plan = plan_of(&view, &mut manager());

        assert!(plan.entry(9, Span::new("title").content_hash()).is_some());
        assert_eq!(
            plan.entry(9, 0).map(|e| e.kind),
            None,
            "hash mismatch must not resolve"
        );
        assert!(plan.entry(1, 1).is_some_and(|e| e.kind == ItemKind::Spacer));
        assert!(plan.entry(2, 4).is_some_and(|e| e.kind == ItemKind::Spacer));
        // Wrapped-line spans were never touched, so they are absent.
        assert!(plan.entry(11, 0).is_none());
        assert!(plan.entry(12, 0).is_none());
    }

    #[test]
    fn first_sight_is_fade_and_second_sight_is_reflow() {
        let make_view = || {
            view_of(vec![StampedObject::new(Object::Paragraph(
                line_with(vec![Span::new("alpha").with_id(7)], &[]),
                "",
            ))])
        };
        let mut manager = manager();

        let plan = plan_of(&make_view(), &mut manager);
        assert!(
            plan.entry(7, Span::new("alpha").content_hash())
                .unwrap()
                .changed
        );
        assert_eq!(
            plan.entry(7, Span::new("alpha").content_hash())
                .unwrap()
                .effect,
            Effect::Fade
        );

        let plan = plan_of(&make_view(), &mut manager);
        let entry = plan.entry(7, Span::new("alpha").content_hash()).unwrap();
        assert!(!entry.changed);
        assert_eq!(entry.effect, Effect::Reflow);
    }

    #[test]
    fn changed_hash_is_fade_even_with_same_geometry() {
        let mut manager = manager();
        let first = view_of(vec![StampedObject::new(Object::Paragraph(
            line_with(vec![Span::new("alpha").with_id(7)], &[]),
            "",
        ))]);
        let second = view_of(vec![StampedObject::new(Object::Paragraph(
            line_with(vec![Span::new("beta").with_id(7)], &[]),
            "",
        ))]);

        plan_of(&first, &mut manager);
        let plan = plan_of(&second, &mut manager);

        let entry = plan.entry(7, Span::new("beta").content_hash()).unwrap();
        assert!(entry.changed);
        assert_eq!(entry.effect, Effect::Fade);
        assert!(entry.fade_in_ms.is_none(), "no authored in-* token");
    }

    #[test]
    fn choice_arm_parent_ids_match_choice_view() {
        let key: PageKey = 0x0000_0000_0000_0040;
        let view = view_of(vec![
            StampedObject::new(Object::Choice(vec![(
                3,
                line_with(vec![Span::new("arm").with_id(21)], &[]),
            )]))
            .with_id(key),
        ]);
        let plan = plan_of(&view, &mut manager());

        // The container is tracked without a name; the arm span keeps its explicit id.
        let container = plan.entry(key, 0).unwrap();
        assert!(container.name.is_none());
        assert!(plan.entry(21, Span::new("arm").content_hash()).is_some());
    }

    #[test]
    fn authored_tokens_carry_timing() {
        let view = view_of(vec![StampedObject::new(Object::Paragraph(
            line_with(
                vec![Span::new("x").with_id(5).cls("in-500-300")],
                &["in-200"],
            ),
            "",
        ))]);
        let plan = plan_of(&view, &mut manager());
        let entry = plan.entry(5, Span::new("x").content_hash()).unwrap();
        assert_eq!(entry.tokens, vec!["in-500-300".to_string()]);
        assert_eq!(entry.timing.in_delay, 500);
        assert_eq!(entry.timing.fade_in, Some(300));
        assert_eq!(entry.fade_in_ms, Some(300));
    }

    #[test]
    fn line_tokens_inherit_when_span_has_none() {
        let view = view_of(vec![StampedObject::new(Object::Paragraph(
            line_with(vec![Span::new("x").with_id(5)], &["in-200-100"]),
            "",
        ))]);
        let plan = plan_of(&view, &mut manager());
        let entry = plan.entry(5, Span::new("x").content_hash()).unwrap();
        assert_eq!(entry.tokens, vec!["in-200-100".to_string()]);
        assert_eq!(entry.fade_in_ms, Some(100));
        assert!(
            plan.duration_rules
                .iter()
                .any(|r| r.token == "in-200-100" && r.duration_ms == 100)
        );
    }

    #[test]
    fn whole_blocks_transitioning_in_delayed_on_same_page_not_fresh_page() {
        let mut manager = manager();

        // 1. Fresh page navigation: whole block does NOT receive transition-in delay
        let fresh_view = view_of(vec![
            StampedObject::new(Object::Paragraph(
                line_with(vec![Span::new("initial").with_id(10)], &[]),
                "",
            ))
            .with_id(100),
        ]);
        let fresh_plan = plan_of(&fresh_view, &mut manager);
        let fresh_entry = fresh_plan.entry(10, Span::new("initial").content_hash()).unwrap();
        assert_eq!(fresh_entry.timing.in_delay, 0, "fresh page must not add in-delay");
        assert!(fresh_entry.tokens.is_empty());

        // 2. Same-page interaction (e.g. click reveals new paragraphs)
        manager.on_view_change(&PageId("page".into()));

        let same_page_view = view_of(vec![
            StampedObject::new(Object::Paragraph(
                line_with(
                    vec![
                        Span::new("initial").with_id(10),
                        Span::new("span inside existing block").with_id(15),
                    ],
                    &[],
                ),
                "",
            ))
            .with_id(100),
            // Newly introduced block on same page:
            StampedObject::new(Object::Paragraph(
                line_with(vec![Span::new("new block paragraph").with_id(20)], &[]),
                "",
            ))
            .with_id(200),
            // Newly introduced block with authored timing:
            StampedObject::new(Object::Paragraph(
                line_with(vec![Span::new("authored paragraph").with_id(30)], &["in-600"]),
                "",
            ))
            .with_id(300),
        ]);

        let same_page_plan = plan_of(&same_page_view, &mut manager);

        // Initial unchanged block keeps no in-delay:
        let initial_entry = same_page_plan.entry(10, Span::new("initial").content_hash()).unwrap();
        assert_eq!(initial_entry.timing.in_delay, 0);

        // Span inside existing block does not receive block-in delay:
        let existing_block_new_span = same_page_plan
            .entry(15, Span::new("span inside existing block").content_hash())
            .unwrap();
        assert_eq!(existing_block_new_span.timing.in_delay, 0);

        // Whole block that transitioned in stays on the primary transition without gaining in- class:
        let new_block_entry = same_page_plan
            .entry(20, Span::new("new block paragraph").content_hash())
            .unwrap();
        assert!(new_block_entry.tokens.is_empty(), "must not gain in- class token");
        assert_eq!(new_block_entry.timing.in_delay, 400);
        assert_eq!(same_page_plan.primary_in_delay, 400);
        assert_eq!(same_page_plan.primary_block_in_delay, 400);

        // Authored block timing is preserved:
        let authored_entry = same_page_plan
            .entry(30, Span::new("authored paragraph").content_hash())
            .unwrap();
        assert_eq!(authored_entry.timing.in_delay, 600);
        assert_eq!(authored_entry.tokens, vec!["in-600".to_string()]);
    }
}
