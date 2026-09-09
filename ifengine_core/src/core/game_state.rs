use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::core::PageId;

/// Internal key-value state store for all pages in the game.
///
/// Manages persistent component state across re-renders. Frontend renderers
/// register click handlers on interactive elements and apply user input via
/// [`Game::interact`](crate::core::Game::interact).
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameState {
    inner: HashMap<PageId, PageMap>,
    #[cfg_attr(feature = "serde", serde(skip))]
    // Force a PageMap (i.e. embeds)
    shared: Option<PageMap>,
}

impl GameState {
    /// Increment the value at the given key by 1.
    /// Initializes the chapter or entry to 0 if it does not exist.
    /// The click action uses [`was_zero`](crate::core::PageState::was_zero) to run its closure exactly once.
    pub fn inc(&mut self, key: &InternalKey) {
        let (chapter_id, entry_key) = key;
        let chapter = self
            .inner
            .entry(chapter_id.clone())
            .or_insert_with(|| PageMap {
                inner: HashMap::new(),
            });
        chapter
            .inner
            .entry(entry_key.clone())
            .and_modify(|v| *v += 1) // increment if already exists
            .or_insert(0);
    }

    /// Insert a specific value at the given key.
    /// Creates the chapter if it does not exist.
    pub fn insert(&mut self, key: InternalKey, value: u64) {
        let (chapter_id, entry_key) = key;
        let chapter = self.inner.entry(chapter_id).or_insert_with(|| PageMap {
            inner: HashMap::new(),
        });
        chapter.inner.insert(entry_key, value);
    }

    /// Remove a specific value at the given key.
    pub fn remove(&mut self, key: &InternalKey) {
        let (chapter_id, entry_key) = key;

        if let Some(chapter) = self.inner.get_mut(chapter_id) {
            chapter.inner.remove(&entry_key);

            // Optional: remove chapter if it is now empty
            if chapter.inner.is_empty() {
                self.inner.remove(chapter_id);
            }
        }
    }

    /// Treating the contained value as a bitmask, set the specified position to true.
    /// Creates the chapter or entry if it does not exist.
    /// pos is u8 but max value should be 64
    pub fn set_bit(&mut self, key: InternalKey, pos: u8) {
        let (chapter_id, entry_key) = key;

        let chapter = self.inner.entry(chapter_id).or_insert_with(|| PageMap {
            inner: HashMap::new(),
        });

        let current = chapter.inner.get(&entry_key).copied().unwrap_or(0);
        let updated = current | (1 << pos);

        chapter.inner.insert(entry_key, updated);
    }

    /// Get a reference to the chapter state for a given chapter ID.
    pub fn get_page(&mut self, pageid: impl Into<PageId>) -> &PageMap {
        if let Some(ref map) = self.shared {
            return map;
        }
        self.inner.entry(pageid.into()).or_default()
    }

    /// Get a mutable reference to the chapter state for a given chapter ID.
    pub fn get_page_mut(&mut self, pageid: impl Into<PageId>) -> &mut PageMap {
        if let Some(ref mut map) = self.shared {
            return map;
        }
        self.inner.entry(pageid.into()).or_default()
    }

    /// Creates a transient [`GameState`] that routes all page queries to `shared`.
    pub fn new_with_shared(shared: PageMap) -> Self {
        Self {
            inner: HashMap::new(),
            shared: Some(shared),
        }
    }

    /// Takes the shared [`PageMap`], leaving `None`.
    pub fn take_shared(&mut self) -> Option<PageMap> {
        self.shared.take()
    }

    /// Returns `true` if this [`GameState`] is routing page queries to a shared parent map.
    pub fn is_transient(&self) -> bool {
        self.shared.is_some()
    }
}

// --------------------------------------------------------

/// Persistent key-value store for an individual page, mapping each [`PageKey`] to a `u64` state value.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PageMap {
    inner: HashMap<PageKey, u64>,
}

pub type InternalKey = (PageId, PageKey);
/// The key used by [`PageState`](crate::core::PageState) to track state
pub type PageKey = u64;

// ---------------- BOILERPLATE ----------------------------

impl GameState {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            shared: None,
        }
    }
}

impl Deref for PageMap {
    type Target = HashMap<PageKey, u64>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PageMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
