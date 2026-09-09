use std::cell::RefCell;

use crate::{
    Game,
    core::{
        GameContext, GameTags, Page, PageId, Response,
        game_state::{PageKey, PageMap},
    },
    view::{Object, View},
};

/// The mask for user keys (top 16 bits must be 0, allowing up to 48-bit user payloads).
const USER_KEY_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// The `#[ifview]` decorator instantiates this from a reference to [`Game`](struct@crate::Game), using it to add elements which read and write to [`GameState`](crate::core::game_state::GameState).
/// Will produce a [`View`] if the decorated function doesn't exit early.
#[derive(Debug)]
pub struct PageState<'a> {
    view: View,
    page_state: RefCell<&'a mut PageMap>, // to allow simultaneous method accesses, safe because chapter_state doesn't produce refs
    #[cfg(feature = "rand")]
    pub seed: Option<u64>,
    fresh: bool,
    game_tags: &'a mut GameTags,
    /// [`simulate`](crate::Game::simulate)
    pub simulating: bool,
    call_site_counters: RefCell<std::collections::HashMap<u64, u16>>,
}

impl<'a> PageState<'a> {
    /// Creates a new `PageState` instance for constructing a page view.
    ///
    /// Typically invoked automatically by the expanded code of the `#[ifview]` macro.
    ///
    /// # Arguments
    /// - `name`: The identifier or path for the page view.
    /// - `fresh`: Whether this is the initial render of the page.
    /// - `simulating`: Whether the view is being evaluated in simulation mode.
    /// - `page_state`: Mutable reference to the backing [`PageMap`] storing page key-value state.
    /// - `game_tags`: Mutable reference to the global [`GameTags`] set.
    pub fn new(
        name: impl Into<PageId>,
        fresh: bool,
        simulating: bool,
        page_state: &'a mut PageMap,
        game_tags: &'a mut GameTags,
    ) -> Self {
        Self {
            view: View::new(name.into()),
            page_state: RefCell::new(page_state),
            #[cfg(feature = "rand")]
            seed: None,
            fresh,
            game_tags,
            simulating,
            call_site_counters: RefCell::new(std::collections::HashMap::new()),
        }
    }
}

/// Computes a 16-bit FNV-1a hash of a byte slice.
const fn fnv1a_16(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    (hash ^ (hash >> 32)) & 0xFFFF
}

/// Constructs the 48-bit location payload from file path, line number, and column number:
/// - Bits 47..32 (16 bits): File path hash
/// - Bits 31..16 (16 bits): Line number
/// - Bits 15..0  (16 bits): Column number
const fn loc_from_file_line_col(file: &str, line: u32, col: u32) -> u64 {
    let file_hash_16 = fnv1a_16(file.as_bytes());
    let line_16 = (line as u64) & 0xFFFF;
    let col_16 = (col as u64) & 0xFFFF;
    (file_hash_16 << 32) | (line_16 << 16) | col_16
}

impl<'a> PageState<'a> {
    /// Generates a deterministic key for an automatically-keyed element using the caller's location.
    ///
    /// The top 16 bits store the 1-based instance counter (`1..=65535`), ensuring that
    /// auto keys never collide with user keys (which have top 16 bits = 0).
    /// The lower 48 bits store the location payload:
    /// - Bits 47..32 (16 bits): File path hash
    /// - Bits 31..16 (16 bits): Line number
    /// - Bits 15..0  (16 bits): Column number
    #[track_caller]
    pub fn auto_key(&self) -> PageKey {
        let caller = std::panic::Location::caller();
        let loc = loc_from_file_line_col(caller.file(), caller.line(), caller.column());
        let mut counters = self.call_site_counters.borrow_mut();
        let count = counters.entry(loc).or_insert(1);
        let c = *count;
        *count = count.saturating_add(1);
        ((c as u64) << 48) | (loc & USER_KEY_MASK)
    }
    /// Appends a view [`Object`] (such as a paragraph, choice list, heading, image, or break)
    /// to the underlying [`View`].
    pub fn push(&mut self, object: Object) {
        self.view.push(object);
    }

    /// Evaluates an embedded page function using a transient [`Game`] and attaches its [`View`].
    ///
    /// If the page returns [`Response::View`], the view is pushed as an [`Object::Embed`]
    /// into the current page and returned. If it returns any other [`Response`] variant (such as
    /// [`Response::Switch`], [`Response::Back`], [`Response::Tunnel`], [`Response::Exit`], or [`Response::End`]),
    /// that response is returned directly so the caller can return it early.
    pub fn embed<C: GameContext>(&mut self, page: Page<C>, ctx: &mut C) -> Response {
        let context = std::mem::take(ctx);
        let tags = std::mem::take(self.game_tags);
        let parent_map = std::mem::take(&mut **self.page_state.borrow_mut());

        let mut transient_game = Game::new_transient_with_map(
            parent_map,
            context,
            tags,
            self.simulating,
            self.fresh,
        );

        let response = page(&mut transient_game);

        *ctx = transient_game.context;
        *self.game_tags = transient_game.tags;
        if let Some(map) = transient_game.inner.state.take_shared() {
            **self.page_state.borrow_mut() = map;
        }

        if let Response::View(mut view) = response {
            view.pageid = self.view.pageid.clone();
            self.push(Object::Embed(view.clone()));
            return Response::View(view);
        }

        response
    }

    /// Returns the unique [`PageId`] of the current page being constructed.
    pub fn id(&self) -> PageId {
        self.view.pageid.clone()
    }

    /// Consumes the `PageState` and converts its inner [`View`] into a [`Response::View`].
    ///
    /// This is invoked at the end of `#[ifview]` functions to produce the final response.
    pub fn into_response(self) -> Response {
        Response::View(self.view)
    }

    /// Returns `true` if this is the first time the page is being rendered (i.e. fresh load),
    /// or `false` if re-rendering following a user interaction on the page.
    pub fn fresh(&self) -> bool {
        self.fresh
    }

    // --------- Chapter state
    // indexing takes owned for convenience (PageKey is copy)
    /// Retrieves the raw `u64` value associated with `key` in the page's state, if present.
    pub fn get(&self, key: PageKey) -> Option<u64> {
        self.page_state.borrow().get(&key).copied()
    }

    /// Interprets the value stored at `key` as a 64-bit bitmask and returns a [`Vec`] of all
    /// bit indices `(0..64)` that are set (i.e. where bit `i` is 1).
    ///
    /// Returns an empty [`Vec`] if the key does not exist.
    pub fn get_mask_indices(&self, key: PageKey) -> Vec<usize> {
        let val = match self.page_state.borrow().get(&key).copied() {
            Some(v) => v,
            None => return Vec::new(),
        };

        let mut bits = Vec::new();
        for i in 0..64 {
            if (val & (1 << i)) != 0 {
                bits.push(i);
            }
        }
        bits
    }

    /// Interprets the value stored at `key` as a 64-bit bitmask and returns a fixed-size
    /// boolean array of length `N`, where index `i` is `true` if bit `i` is set.
    ///
    /// Returns `[false; N]` if the key does not exist.
    pub fn get_mask<const N: usize>(&self, key: PageKey) -> [bool; N] {
        let val = match self.page_state.borrow().get(&key).copied() {
            Some(v) => v,
            None => return [false; N],
        };

        std::array::from_fn(|i| i < 64 && (val & (1 << i)) != 0)
    }

    /// Reads the bitmask stored at `key` without removing it, returning the index of the
    /// least significant set bit (calculated via trailing zeros).
    ///
    /// Returns `None` if the key does not exist or if the stored value is `0`.
    pub fn get_mask_last(&self, key: PageKey) -> Option<u8> {
        let val = match self.page_state.borrow().get(&key).copied() {
            Some(v) => v,
            None => return None,
        };

        if val == 0 {
            None
        } else {
            Some(val.trailing_zeros() as u8)
        }
    }

    /// Removes the bitmask stored at `key` from the page state and returns the index of the
    /// least significant set bit (calculated via trailing zeros).
    ///
    /// Returns `None` if the key was not found or if the stored value was `0`.
    /// Used by choice macros (e.g. `dchoice!`) to consume a single click selection.
    pub fn remove_mask_last(&mut self, key: PageKey) -> Option<u8> {
        let val = match self.page_state.borrow_mut().remove(&key) {
            Some(v) => v,
            None => return None,
        };

        if val == 0 {
            None
        } else {
            Some(val.trailing_zeros() as u8)
        }
    }

    /// Selects a pseudo-random index in `0..range`, excluding any indices in `exclude`.
    ///
    /// If `self.seed` is set, uses a deterministic `StdRng` seeded with `seed`;
    /// otherwise, samples from the thread-local RNG.
    ///
    /// # Panics
    /// Panics if all numbers in `0..range` are excluded.
    #[cfg(feature = "rand")]
    pub fn rand(&self, range: usize, exclude: &[usize]) -> usize {
        use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};

        let excl: std::collections::HashSet<usize> = exclude.iter().copied().collect();

        let pool: Vec<usize> = (0..range).filter(|i| !excl.contains(i)).collect();

        if pool.is_empty() {
            panic!("rand(): range exhausted by exclusion list");
        }

        if let Some(seed) = self.seed {
            *pool.choose(&mut StdRng::seed_from_u64(seed)).unwrap()
        } else {
            *pool.choose(&mut rand::rng()).unwrap()
        }
    }

    /// Inserts or updates a raw `u64` value for `key` in the page's persistent state.
    pub fn insert(&self, key: PageKey, value: u64) {
        self.page_state.borrow_mut().insert(key, value);
    }

    /// Removes `key` from the page's persistent state and returns its previous `u64` value,
    /// or `None` if the key was not present.
    pub fn remove(&self, key: PageKey) -> Option<u64> {
        self.page_state.borrow_mut().remove(&key)
    }

    /// Checks whether the value at `key` is currently `0`.
    ///
    /// If it is `0`, updates it to `1` and returns `true`. Otherwise returns `false`.
    /// Primarily used by `click!` to ensure a click handler executes only on the initial click.
    pub fn was_zero(&self, key: PageKey) -> bool {
        if let Some(x) = self.page_state.borrow_mut().get_mut(&key)
            && *x == 0
        {
            *x = 1;
            true
        } else {
            false
        }
    }

    /// Adds a tag to both the current [`View`]'s tag list and the global [`GameTags`].
    ///
    /// Returns `true` if the tag was newly inserted into global game tags, or `false` if already present.
    pub fn tag(&mut self, s: &str) -> bool {
        let q: PageId = s.into();
        self.view.tags.push(q.clone());
        self.game_tags.insert(q)
    }

    /// Removes a tag from the global [`GameTags`].
    ///
    /// Returns `true` if the tag was present and removed, or `false` otherwise.
    pub fn untag(&mut self, s: &str) -> bool {
        self.game_tags.remove(&s.into())
    }
}

// ------------- BOILERPLATE
use std::fmt;

impl<'a> fmt::Display for PageState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.view.pageid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_auto_key_domain_and_determinism() {
        let mut page_map1 = PageMap::default();
        let mut tags1 = HashSet::new();
        let state1 = PageState::new("test", true, false, &mut page_map1, &mut tags1);

        // Helper representing the same call site inside a page function executed across renders
        fn render_elements(state: &PageState) -> Vec<PageKey> {
            let mut keys = Vec::new();
            for _ in 0..3 {
                keys.push(state.auto_key());
            }
            keys
        }

        let keys1 = render_elements(&state1);

        // Counter starts at 1 in the top 16 bits
        assert_eq!(keys1[0] >> 48, 1);
        assert_eq!(keys1[1] >> 48, 2);
        assert_eq!(keys1[2] >> 48, 3);

        // Lower 48 bits preserve the location payload across loop iterations
        assert_eq!(keys1[0] & USER_KEY_MASK, keys1[1] & USER_KEY_MASK);
        assert_eq!(keys1[1] & USER_KEY_MASK, keys1[2] & USER_KEY_MASK);

        // User keys (e.g. 6) have top 16 bits = 0
        let user_key = 6u64 & USER_KEY_MASK;
        assert_eq!(user_key >> 48, 0);

        // Keys at the same call site in a loop must be distinct
        assert_ne!(keys1[0], keys1[1]);
        assert_ne!(keys1[1], keys1[2]);
        assert_ne!(keys1[0], keys1[2]);

        // Subsequent render of the same page function produces identical keys
        let mut page_map2 = PageMap::default();
        let mut tags2 = HashSet::new();
        let state2 = PageState::new("test", false, false, &mut page_map2, &mut tags2);

        let keys2 = render_elements(&state2);

        assert_eq!(keys1, keys2);
    }
}

