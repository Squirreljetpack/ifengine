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
    pub fn inc(&mut self, page_id: &PageId, key: PageKey) {
        let chapter = self
            .inner
            .entry(page_id.clone())
            .or_insert_with(|| PageMap {
                inner: HashMap::new(),
            });
        chapter
            .inner
            .entry(key)
            .and_modify(|v| *v += 1) // increment if already exists
            .or_insert(1);
    }

    /// Insert a specific value at the given key.
    /// Creates the chapter if it does not exist.
    pub fn insert(&mut self, page_id: &PageId, key: PageKey, value: u64) {
        let chapter = self
            .inner
            .entry(page_id.clone())
            .or_insert_with(|| PageMap {
                inner: HashMap::new(),
            });
        chapter.inner.insert(key, value);
    }

    /// Sets or increments the value at `key` based on `auto_key`.
    ///
    /// Checks if the lower 48 bits (3 * 16 bits) match `auto_key & USER_KEY_MASK`.
    /// If yes, increments the counter in the top 16 bits (masking to 15 bits); otherwise initializes to `auto_key`.
    pub fn set_inc(&mut self, page_id: &PageId, key: PageKey, auto_key: u64) {
        let loc = auto_key & USER_KEY_MASK;
        let chapter = self
            .inner
            .entry(page_id.clone())
            .or_insert_with(|| PageMap {
                inner: HashMap::new(),
            });
        match chapter.inner.get_mut(&key) {
            Some(v) if (*v & USER_KEY_MASK) == loc => {
                let count = ((*v >> 48) & 0x7FFF) as u16;
                let new_count = (count.saturating_add(1)) & 0x7FFF;
                *v = ((new_count as u64) << 48) | loc;
            }
            Some(v) => *v = auto_key,
            None => {
                chapter.inner.insert(key, auto_key);
            }
        }
    }

    /// Remove a specific value at the given key.
    pub fn remove(&mut self, page_id: &PageId, key: PageKey) {
        if let Some(chapter) = self.inner.get_mut(page_id) {
            chapter.inner.remove(&key);

            // Optional: remove chapter if it is now empty
            if chapter.inner.is_empty() {
                self.inner.remove(page_id);
            }
        }
    }

    /// Treating the contained value as a bitmask, set the specified position to true.
    /// Creates the chapter or entry if it does not exist.
    /// pos is u8 but max value should be 64
    pub fn set_bit(&mut self, page_id: &PageId, key: PageKey, pos: u8) {
        let chapter = self
            .inner
            .entry(page_id.clone())
            .or_insert_with(|| PageMap {
                inner: HashMap::new(),
            });

        let current = chapter.inner.get(&key).copied().unwrap_or(0);
        let updated = current | (1 << pos);

        chapter.inner.insert(key, updated);
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

/// The key used by [`PageState`](crate::core::PageState) to track state
pub type PageKey = u64;

/// The mask for user keys and location payloads (lower 48 bits, top 16 bits must be 0).
pub const USER_KEY_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Top bit reserved for string-hashed lookup keys.
pub const HASH_KEY_BIT: u64 = 1u64 << 63;

/// Computes a 64-bit FNV-1a hash of a string with the top bit (bit 63) set to 1.
pub const fn hash_key(s: &str) -> PageKey {
    const_fnv1a_hash::fnv1a_hash_str_64(s) | HASH_KEY_BIT
}

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
