use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::core::PageId;

/// Used by ifengine to track component states
/// It's methods should not be called directly in code
/// During rendering, the ui should register hooks on Spans with actions,
/// and call [`crate::Game::handle_action`] on click
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameState {
    inner: HashMap<PageId, PageMap>,
}

impl GameState {
    /// Increment the value at the given key by 1.
    /// Creates the chapter or entry if it does not exist.
    pub fn inc(&mut self, key: &InternalKey) {
        let (chapter_id, entry_key) = key;
        let chapter = self
            .inner
            .entry(chapter_id.into())
            .or_insert_with(|| PageMap {
                inner: HashMap::new(),
            });
        *chapter.inner.entry(entry_key.clone()).or_insert(0) += 1;
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
    pub fn get_chapter(&mut self, chapter_id: &PageId) -> &PageMap {
        self.inner.entry(chapter_id.into()).or_default()
    }

    /// Get a mutable reference to the chapter state for a given chapter ID.
    pub fn get_chapter_mut(&mut self, chapter_id: &PageId) -> &mut PageMap {
        self.inner.entry(chapter_id.into()).or_default()
    }
}

// --------------------------------------------------------

/// Nothing more than a Hashmap
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PageMap {
    inner: HashMap<PageKey, u64>,
}

pub type InternalKey = (PageId, PageKey);
/// The key used by [`crate::core::PageState`] to track state
pub type PageKey = u64;

// ---------------- BOILERPLATE ----------------------------

impl GameState {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
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
