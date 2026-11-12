//! Transition scheduling state.
//!
//! Pure state machine: epoch/generation invalidation, milestone batching, and
//! sequential non-overlapping execution. The runtime owns a [`Scheduler`] and turns
//! the returned [`Action`]s into timers or backend runs. Time is passed in so the
//! decisions are unit-testable.

use std::collections::{BTreeMap, VecDeque};

/// What the caller should do after a scheduler transition.
pub(crate) enum Action {
    /// Nothing outstanding.
    None,
    /// Run this batch (secondary transition) now.
    Batch {
        callbacks: Vec<Box<dyn FnOnce()>>,
        epoch: u64,
    },
    /// Arm a timer for `wait_ms`; call [`Scheduler::delay_fired`] with these tokens when it fires.
    Timer { timer_generation: u64, wait_ms: u64 },
}

#[derive(Default)]
pub(crate) struct Scheduler {
    epoch: u64,
    timer_generation: u64,
    scheduled_milestone: Option<u64>,
    active_transition: bool,
    primary_finished_at: Option<f64>,
    pending_batches: BTreeMap<u64, Vec<Box<dyn FnOnce()>>>,
    ready_queue: VecDeque<Vec<Box<dyn FnOnce()>>>,
}

impl Scheduler {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Invalidates all outstanding timers and batches.
    pub fn reset(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.timer_generation = self.timer_generation.wrapping_add(1);
        self.scheduled_milestone = None;
        self.pending_batches.clear();
        self.ready_queue.clear();
        self.active_transition = false;
        self.primary_finished_at = None;
    }

    /// Begins a primary (same-page) transition, invalidating pending secondary work.
    pub fn begin_primary(&mut self) -> u64 {
        self.epoch = self.epoch.wrapping_add(1);
        self.timer_generation = self.timer_generation.wrapping_add(1);
        self.scheduled_milestone = None;
        self.pending_batches.clear();
        self.ready_queue.clear();
        self.active_transition = true;
        self.primary_finished_at = None;
        self.epoch
    }

    /// Registers `callbacks` for `delay_ms` after the primary transition completed.
    pub fn schedule(&mut self, delay_ms: u64, callbacks: Box<dyn FnOnce()>, now: f64) -> Action {
        self.pending_batches
            .entry(delay_ms)
            .or_default()
            .push(callbacks);

        let base_time = match self.primary_finished_at {
            Some(time) => time,
            None => return Action::None,
        };
        if self.active_transition {
            return Action::None;
        }

        let Some(&earliest) = self.pending_batches.keys().next() else {
            return Action::None;
        };
        if let Some(current_milestone) = self.scheduled_milestone
            && earliest >= current_milestone
        {
            return Action::None;
        }

        self.timer_generation = self.timer_generation.wrapping_add(1);
        self.scheduled_milestone = Some(earliest);

        let elapsed = now - base_time;
        let wait_ms = (earliest as f64 - elapsed).max(0.0) as u64;
        Action::Timer {
            timer_generation: self.timer_generation,
            wait_ms,
        }
    }

    /// Called when a delay timer fires.
    pub fn delay_fired(&mut self, epoch: u64, timer_generation: u64) -> Action {
        if self.epoch != epoch || self.timer_generation != timer_generation {
            return Action::None;
        }
        self.scheduled_milestone = None;

        let Some(&earliest_delay) = self.pending_batches.keys().next() else {
            return Action::None;
        };
        let Some(callbacks) = self.pending_batches.remove(&earliest_delay) else {
            return Action::None;
        };

        if self.active_transition {
            self.ready_queue.push_back(callbacks);
            Action::None
        } else {
            self.active_transition = true;
            Action::Batch {
                callbacks,
                epoch: self.epoch,
            }
        }
    }

    /// Marks the primary page entrance finished, arming the earliest delayed batch.
    pub fn primary_finished(&mut self, now: f64) -> Action {
        self.primary_finished_at = Some(now);
        if self.active_transition {
            return Action::None;
        }
        let Some(&earliest) = self.pending_batches.keys().next() else {
            return Action::None;
        };

        self.timer_generation = self.timer_generation.wrapping_add(1);
        self.scheduled_milestone = Some(earliest);
        Action::Timer {
            timer_generation: self.timer_generation,
            wait_ms: earliest,
        }
    }

    /// Called when a transition (primary or secondary) finished.
    pub fn transition_finished(&mut self, epoch: u64, is_secondary: bool, now: f64) -> Action {
        if self.epoch != epoch {
            return Action::None;
        }
        if !is_secondary {
            self.primary_finished_at = Some(now);
        }

        if let Some(next) = self.ready_queue.pop_front() {
            return Action::Batch {
                callbacks: next,
                epoch: self.epoch,
            };
        }

        self.active_transition = false;

        let Some(base_time) = self.primary_finished_at else {
            return Action::None;
        };
        let Some(&next_delay) = self.pending_batches.keys().next() else {
            return Action::None;
        };

        let elapsed = now - base_time;
        let wait_ms = (next_delay as f64 - elapsed).max(0.0) as u64;

        if wait_ms == 0 {
            let callbacks = self.pending_batches.remove(&next_delay).unwrap();
            self.active_transition = true;
            Action::Batch {
                callbacks,
                epoch: self.epoch,
            }
        } else {
            self.timer_generation = self.timer_generation.wrapping_add(1);
            self.scheduled_milestone = Some(next_delay);
            Action::Timer {
                timer_generation: self.timer_generation,
                wait_ms,
            }
        }
    }
}

use std::cell::RefCell;

thread_local! {
    static SCHEDULER: RefCell<Scheduler> = RefCell::new(Scheduler::default());
}

/// Runs `f` against the page-global scheduler state.
pub(crate) fn with<R>(f: impl FnOnce(&mut Scheduler) -> R) -> R {
    SCHEDULER.with(|state| f(&mut state.borrow_mut()))
}

/// Monotonic wall-clock in milliseconds, as seen by the browser.
pub(crate) fn now_ms() -> f64 {
    js_sys::Date::now()
}

/// Reads a CSS time custom property (e.g. `0.6s` or `400ms`) from the document root
/// and returns it in milliseconds, falling back when it is unavailable.
pub(crate) fn read_css_time_ms(var: &str, fallback: u64) -> u64 {
    let Some(window) = web_sys::window() else {
        return fallback;
    };
    let Some(root) = window.document().and_then(|d| d.document_element()) else {
        return fallback;
    };
    let Ok(Some(style)) = window.get_computed_style(&root) else {
        return fallback;
    };
    let Ok(value) = style.get_property_value(var) else {
        return fallback;
    };
    let value = value.trim();
    if let Some(ms) = value.strip_suffix("ms") {
        return ms.trim().parse().unwrap_or(fallback);
    }
    if let Some(secs) = value.strip_suffix('s') {
        return (secs
            .trim()
            .parse::<f64>()
            .unwrap_or(fallback as f64 / 1000.0)
            * 1000.0)
            .round() as u64;
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_milestone_callbacks_batch_and_milestones_sequence() {
        let mut scheduler = Scheduler::default();

        // Registered before the primary finishes: no timer yet.
        assert!(matches!(
            scheduler.schedule(300, Box::new(|| {}), 100.0),
            Action::None
        ));
        assert!(matches!(
            scheduler.schedule(300, Box::new(|| {}), 100.0),
            Action::None
        ));
        assert!(matches!(
            scheduler.schedule(500, Box::new(|| {}), 100.0),
            Action::None
        ));

        // Primary finishes at t=1000; the 300ms milestone is 300ms away.
        let Action::Timer {
            wait_ms,
            timer_generation,
        } = scheduler.primary_finished(1_000.0)
        else {
            panic!("expected timer");
        };
        assert_eq!(wait_ms, 300);

        let epoch = scheduler.epoch();
        let Action::Batch { callbacks, .. } = scheduler.delay_fired(epoch, timer_generation) else {
            panic!("expected batch");
        };
        assert_eq!(callbacks.len(), 2, "same milestone callbacks run together");

        // While a transition is active, a later milestone is queued, not timer-armed.
        assert!(matches!(
            scheduler.schedule(100, Box::new(|| {}), 1_000.0),
            Action::None
        ));

        // A finished secondary keeps the primary base time, so the queued milestone
        // is already due and runs immediately.
        let Action::Batch { callbacks, .. } = scheduler.transition_finished(epoch, true, 1_100.0)
        else {
            panic!("expected immediate batch");
        };
        assert_eq!(callbacks.len(), 1);

        // The 500ms milestone is still 400ms out.
        let Action::Timer {
            wait_ms,
            timer_generation,
        } = scheduler.transition_finished(epoch, true, 1_100.0)
        else {
            panic!("expected timer");
        };
        assert_eq!(wait_ms, 400);
        let Action::Batch { callbacks, .. } = scheduler.delay_fired(epoch, timer_generation) else {
            panic!("expected batch");
        };
        assert_eq!(callbacks.len(), 1);

        let Action::None = scheduler.transition_finished(epoch, true, 1_500.0) else {
            panic!("no work left");
        };
    }

    #[test]
    fn reset_invalidates_inflight_timers() {
        let mut scheduler = Scheduler::default();
        let _ = scheduler.schedule(200, Box::new(|| {}), 0.0);
        let Action::Timer {
            timer_generation, ..
        } = scheduler.primary_finished(0.0)
        else {
            panic!("expected timer");
        };
        let epoch = scheduler.epoch();

        scheduler.reset();
        assert!(matches!(
            scheduler.delay_fired(epoch, timer_generation),
            Action::None
        ));
    }

    #[test]
    fn milestone_after_primary_completion_uses_elapsed_time() {
        let mut scheduler = Scheduler::default();
        assert!(matches!(scheduler.primary_finished(5_000.0), Action::None));

        let Action::Timer { wait_ms, .. } = scheduler.schedule(400, Box::new(|| {}), 5_250.0)
        else {
            panic!("expected timer");
        };
        assert_eq!(wait_ms, 150);
    }
}
