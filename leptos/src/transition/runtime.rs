//! Runtime state: change tracking, the current plan, and the transition scheduler entry points.
//!
//! [`TransitionRuntime`] lives behind a [`leptos::prelude::RwSignal`] and is reached
//! through the `Copy` [`TransitionHandle`] stored in [`StoryContext`](crate::context::StoryContext).
//! The scheduler itself is page-global state in [`super::scheduler`]; callbacks handed to
//! JS re-enter through the handle. Guards are never held while running an update or
//! dispatching a backend.

use std::sync::Arc;

use ifengine::PageKey;
use ifengine::core::PageId;
use ifengine::view::View;
use leptos::prelude::*;

use super::backend::{self, TransitionBackend};
use super::plan::{Effect, ItemKind, TransitionPlan};
use super::scheduler::{self, Action};
use super::{TransitionManager, effective_transition_classes, parse_transition_classes};

/// Identity and fallback classification for one element, supplied by the component.
pub struct ItemKey<'a> {
    pub id: Option<PageKey>,
    pub hash: u64,
    /// Content-change answer used when the item has no identity or no plan entry.
    pub changed: bool,
    pub kind: ItemKind,
    /// The element's own authored classes.
    pub classes: &'a [String],
    /// Inherited line classes, consulted when the element has no `in-*`/`out-*` token.
    pub inherited_classes: Option<&'a [String]>,
}

/// Backend-agnostic attributes emitted by a component for one item.
#[derive(Debug, Clone)]
pub struct ItemAttrs {
    /// Hash-based content change (before geometry).
    pub changed: bool,
    /// `view-transition-name` value, or an empty string for tracked containers.
    pub name: String,
    pub hash: u64,
    pub effect: Effect,
    pub kind: ItemKind,
    /// Backend-specific inline style; empty unless the backend needs one.
    pub style: String,
}

/// Backend and change-tracking state for the whole app.
pub struct TransitionRuntime {
    backend: TransitionBackend,
    manager: TransitionManager,
    plan: Option<Arc<TransitionPlan>>,
}

impl TransitionRuntime {
    pub fn new(backend: TransitionBackend, initial_page: PageId) -> Self {
        Self {
            backend,
            manager: TransitionManager::new(initial_page),
            plan: None,
        }
    }

    /// Detects the backend for the current engine.
    pub fn detect(initial_page: PageId) -> Self {
        Self::new(TransitionBackend::detect(), initial_page)
    }

    pub fn backend(&self) -> TransitionBackend {
        self.backend
    }

    fn set_backend(&mut self, backend: TransitionBackend) {
        self.backend = backend;
    }

    fn build_plan(&mut self, view: &View) -> Arc<TransitionPlan> {
        let plan = Arc::new(super::plan::build(view, &mut self.manager));
        self.plan = Some(Arc::clone(&plan));
        plan
    }

    fn item_attrs(&mut self, key: ItemKey<'_>) -> ItemAttrs {
        let entry = self
            .plan
            .as_ref()
            .and_then(|plan| key.id.and_then(|id| plan.entry(id, key.hash)));

        let (changed, name, tokens, kind) = match entry {
            Some(entry) => (
                entry.changed,
                entry.name.clone(),
                entry.tokens.clone(),
                entry.kind,
            ),
            None => {
                let is_delayed = parse_transition_classes(key.classes).in_delay.unwrap_or(0) > 0;
                let changed = match key.id {
                    Some(id) => self.manager.is_content_changed_delayed(Some(id), key.hash, is_delayed),
                    None => key.changed,
                };
                let tokens =
                    effective_transition_classes(key.classes, key.inherited_classes.unwrap_or(&[]));
                let name = if key.kind == ItemKind::Block {
                    None
                } else {
                    key.id.map(|id| format!("item-{id}"))
                };
                (changed, name, tokens, key.kind)
            }
        };

        let effect = Effect::from_changed(changed);
        let is_block_in = entry.is_some_and(|e| e.timing.in_delay > 0 && e.tokens.is_empty());
        let style = match (&name, self.backend) {
            (Some(name), TransitionBackend::ViewTransition) => {
                let class = if changed && !tokens.is_empty() {
                    format!("fade {}", tokens.join(" "))
                } else if is_block_in {
                    "fade block-in".to_string()
                } else {
                    effect.as_str().to_string()
                };
                format!("view-transition-name: {name}; view-transition-class: {class};")
            }
            _ => String::new(),
        };

        ItemAttrs {
            changed,
            name: name.unwrap_or_default(),
            hash: key.hash,
            effect,
            kind,
            style,
        }
    }
}

/// `Copy` entry point to [`TransitionRuntime`], stored in the story context.
#[derive(Clone, Copy)]
pub struct TransitionHandle(RwSignal<TransitionRuntime>);

impl TransitionHandle {
    pub fn new(runtime: TransitionRuntime) -> Self {
        Self(RwSignal::new(runtime))
    }

    pub fn backend(&self) -> TransitionBackend {
        self.0.read_untracked().backend()
    }

    /// Backend currently in use, read tracked so views subscribed to it re-render.
    pub fn selected_backend(&self) -> TransitionBackend {
        self.0.read().backend()
    }

    /// Switches the motion backend, cancels in-flight effects of the previous one and
    /// stores the choice as the reader's preference.
    pub fn set_backend(&self, backend: TransitionBackend) {
        let previous = self.backend();
        if previous == backend {
            return;
        }
        backend::reset(previous);
        self.0.write().set_backend(backend);
        backend::announce(backend);
        crate::storage::save_preferences(&crate::storage::Preferences {
            motion: Some(backend),
        });
    }

    /// Tells the change tracker which page subsequent renders belong to.
    pub fn on_view_change(&self, page: &PageId) {
        self.0.write_untracked().manager.on_view_change(page);
    }

    /// Records an item as seen on the current page once its transition begins to run.
    pub fn mark_seen(&self, id: Option<PageKey>, hash: u64) {
        self.0.write_untracked().manager.mark_seen(id, hash);
    }

    /// Builds and stores the plan for `view`, recording content-change state.
    pub fn build_plan(&self, view: &View) -> Arc<TransitionPlan> {
        self.0.write_untracked().build_plan(view)
    }

    /// Resolves the attributes for one element during render.
    ///
    /// Reads the runtime reactively: the inline style depends on the backend, so items
    /// re-render when the reader switches backends.
    pub fn item_attrs(&self, key: ItemKey<'_>) -> ItemAttrs {
        let _ = self.0.read().backend();
        self.0.write_untracked().item_attrs(key)
    }

    /// Runs `update` as the primary (same-page) transition.
    pub fn commit(&self, plan: Arc<TransitionPlan>, update: impl FnOnce() + 'static) {
        let epoch = {
            self.0.write_untracked().plan = Some(plan);
            scheduler::with(|scheduler| scheduler.begin_primary())
        };
        self.dispatch(epoch, false, vec![Box::new(update)]);
    }

    /// Schedules `update` for `delay_ms` after the primary transition, batched by milestone.
    pub fn schedule(&self, delay_ms: u64, update: impl FnOnce() + 'static) {
        if delay_ms == 0 {
            update();
            return;
        }
        let action = scheduler::with(|scheduler| {
            scheduler.schedule(delay_ms, Box::new(update), scheduler::now_ms())
        });
        self.run_action(action);
    }

    /// Kicks off delayed batches once the page entrance transition completed.
    pub fn finish_page_transition(&self) {
        let action = scheduler::with(|scheduler| scheduler.primary_finished(scheduler::now_ms()));
        self.run_action(action);
    }

    /// Cancels outstanding transitions and backend effects (fresh page navigation).
    pub fn reset(&self) {
        scheduler::with(|scheduler| scheduler.reset());
        backend::reset(self.backend());
    }

    fn delay_timer_fired(&self, epoch: u64, timer_generation: u64) {
        let action = scheduler::with(|scheduler| scheduler.delay_fired(epoch, timer_generation));
        self.run_action(action);
    }

    fn transition_finished(&self, epoch: u64, is_secondary: bool) {
        if is_secondary {
            set_secondary_transition_class(false);
        }
        let action = scheduler::with(|scheduler| {
            scheduler.transition_finished(epoch, is_secondary, scheduler::now_ms())
        });
        self.run_action(action);
    }

    fn run_action(&self, action: Action) {
        match action {
            Action::None => {}
            Action::Batch { callbacks, epoch } => self.dispatch(epoch, true, callbacks),
            Action::Timer {
                timer_generation,
                wait_ms,
            } => self.set_timer(timer_generation, wait_ms),
        }
    }

    fn dispatch(&self, epoch: u64, is_secondary: bool, callbacks: Vec<Box<dyn FnOnce()>>) {
        set_secondary_transition_class(is_secondary);
        let (backend, plan) = {
            let runtime = self.0.read_untracked();
            (runtime.backend(), runtime.plan.clone().unwrap_or_default())
        };
        let handle = *self;
        backend::run(backend, plan, is_secondary, callbacks, move || {
            handle.transition_finished(epoch, is_secondary);
        });
    }

    fn set_timer(&self, timer_generation: u64, wait_ms: u64) {
        let epoch = scheduler::with(|scheduler| scheduler.epoch());
        let handle = *self;
        set_timeout(
            move || handle.delay_timer_fired(epoch, timer_generation),
            std::time::Duration::from_millis(wait_ms),
        );
    }
}

/// Toggles the `html.secondary-transition` marker that selects the secondary
/// `::view-transition-*` timing rules for the duration of a delayed batch.
fn set_secondary_transition_class(is_secondary: bool) {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let classes = root.class_list();
        if is_secondary {
            let _ = classes.add_1("secondary-transition");
        } else {
            let _ = classes.remove_1("secondary-transition");
        }
    }
}
