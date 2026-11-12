use std::cell::RefCell;

use crate::{
    Game,
    core::{
        GameContext, GameTags, Page, PageHandle, PageId, PageKey, Response,
        game_state::PageMap,
        key::{
            DIRTY_BIT, IntoPageKey, LINK_COUNT_MASK, LINK_LOC_MASK, USER_KEY_MASK, link_loc_bits,
        },
    },
    view::{Object, RenderData, StampedObject, View},
};

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
    pre_embed: Option<PageHandle>,
    post_embed: Option<PageHandle>,
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
        Self::new_with_embeds(name, fresh, simulating, page_state, game_tags, None, None)
    }

    /// Creates a [`PageState`] with optional views to render around the page.
    ///
    /// The hooks are evaluated with the same context and page state as the
    /// page being rendered. A hook that returns a [`Response::View`] is
    /// wrapped in an [`Object::Embed`](crate::view::Object::Embed) and placed
    /// before or after the page view when [`Self::into_response_with_context`]
    /// is called.
    pub fn new_with_embeds(
        name: impl Into<PageId>,
        fresh: bool,
        simulating: bool,
        page_state: &'a mut PageMap,
        game_tags: &'a mut GameTags,
        pre_embed: Option<PageHandle>,
        post_embed: Option<PageHandle>,
    ) -> Self {
        Self {
            view: View::new(name.into()),
            page_state: RefCell::new(page_state),
            #[cfg(feature = "rand")]
            seed: None,
            fresh,
            game_tags,
            pre_embed,
            post_embed,
            simulating,
            call_site_counters: RefCell::new(std::collections::HashMap::new()),
        }
    }
}

/// Constructs the 48-bit location payload from file path, line number, and column number:
/// - Bits 47..32 (16 bits): File path hash (16-bit folded FNV-1a)
/// - Bits 31..16 (16 bits): Line number
/// - Bits 15..0  (16 bits): Column number
const fn loc_from_file_line_col(file: &str, line: u32, col: u32) -> u64 {
    let hash = const_fnv1a_hash::fnv1a_hash_str_64(file);
    let file_hash_16 = (hash ^ (hash >> 32)) & 0xFFFF;
    let line_16 = (line as u64) & 0xFFFF;
    let col_16 = (col as u64) & 0xFFFF;
    (file_hash_16 << 32) | (line_16 << 16) | col_16
}

impl<'a> PageState<'a> {
    /// Generates a deterministic key for an automatically-keyed element using the caller's location.
    ///
    /// Bits 48..62 store the 1-based instance counter (`1..=32767`), ensuring that
    /// auto keys never collide with user keys (top 16 bits = 0) or hashed keys (bit 63 = 1).
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
        (((c as u64) & 0x7FFF) << 48) | (loc & USER_KEY_MASK)
    }
    /// Appends a view element (wrapped in [`StampedObject`]) to the underlying [`View`].
    pub fn push(&mut self, item: impl Into<StampedObject>) {
        self.view.push(item.into());
    }

    /// Clears all elements from the underlying [`View`].
    pub fn clear(&mut self) {
        self.view.clear();
    }

    /// Returns a mutable reference to the last stamped object in the page view, if any.
    pub fn last_mut(&mut self) -> Option<&mut StampedObject> {
        self.view.last_mut()
    }

    /// Returns a reference to the last stamped object in the page view, if any.
    pub fn last(&self) -> Option<&StampedObject> {
        self.view.last()
    }

    /// Evaluates an embedded page function using a transient [`struct@Game`] and attaches its [`View`].
    ///
    /// If the page returns [`Response::View`], the view is pushed as an [`Object::Embed`]
    /// into the current page and returned. If it returns any other [`Response`] variant (such as
    /// [`Response::Switch`], [`Response::Back`], [`Response::Tunnel`], [`Response::Exit`], or [`Response::End`]),
    /// that response is returned directly so the caller can return it early.
    pub fn embed<C: GameContext>(&mut self, page: Page<C>, ctx: &mut C) -> Response {
        self.embed_with_render_data(page, ctx, "")
    }

    /// Evaluates an embedded page function with attached [`RenderData`].
    pub fn embed_with_render_data<C: GameContext>(
        &mut self,
        page: Page<C>,
        ctx: &mut C,
        render_data: RenderData,
    ) -> Response {
        let context = std::mem::take(ctx);
        let tags = std::mem::take(self.game_tags);
        let parent_map = std::mem::take(&mut **self.page_state.borrow_mut());

        let mut transient_game = Game::new_transient_with_map(
            parent_map,
            self.view.pageid.clone(),
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

        if let Response::View(view) = response {
            if view.inner.is_empty() {
                return Response::View(view);
            }
            let key = self.auto_key();
            self.push(StampedObject::new(Object::Embed(view.clone(), render_data)).with_id(key));
            return Response::View(view);
        }

        response
    }

    /// Pushes an empty embedded view serving as an extensible custom marker.
    pub fn push_empty_embed(&mut self, render_data: RenderData) {
        let key = self.auto_key();
        let empty_view = View::new(self.view.pageid.clone());
        self.push(StampedObject::new(Object::Embed(empty_view, render_data)).with_id(key));
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

    /// Consumes the [`PageState`] and applies its configured pre/post hooks
    /// before returning the rendered response.
    pub fn into_response_with_context<C: GameContext>(mut self, ctx: &mut C) -> Response {
        if let Some(pre_embed) = self.pre_embed.take() {
            match self.eval_page_handle(&pre_embed, ctx) {
                Response::View(embed_view) => {
                    if !embed_view.inner.is_empty() {
                        self.view
                            .inner
                            .insert(0, StampedObject::new(Object::Embed(embed_view, "")));
                    }
                }
                response => return response,
            }
        }

        if let Some(post_embed) = self.post_embed.take() {
            match self.eval_page_handle(&post_embed, ctx) {
                Response::View(embed_view) => {
                    if !embed_view.inner.is_empty() {
                        self.view
                            .inner
                            .push(StampedObject::new(Object::Embed(embed_view, "")));
                    }
                }
                response => return response,
            }
        }

        Response::View(self.view)
    }

    /// Evaluates a page hook using a transient game that shares this page's
    /// context, tags, and state map.
    fn eval_page_handle<C: GameContext>(&mut self, page: &PageHandle, ctx: &mut C) -> Response {
        let context = std::mem::take(ctx);
        let tags = std::mem::take(self.game_tags);
        let parent_map = std::mem::take(&mut **self.page_state.borrow_mut());

        let mut transient_game = Game::new_transient_with_map(
            parent_map,
            self.view.pageid.clone(),
            context,
            tags,
            self.simulating,
            self.fresh,
        );

        let response = page.call(&mut transient_game);

        *ctx = transient_game.context;
        *self.game_tags = transient_game.tags;
        if let Some(map) = transient_game.inner.state.take_shared() {
            **self.page_state.borrow_mut() = map;
        }

        response
    }

    /// Returns `true` if this is the first time the page is being rendered (i.e. fresh load),
    /// or `false` if re-rendering following a user interaction on the page.
    pub fn fresh(&self) -> bool {
        self.fresh
    }

    // --------- Chapter state
    // indexing takes owned for convenience (PageKey is copy)
    /// Retrieves the raw `u64` value associated with `key` in the page's state, if present.
    pub fn get(&self, key: impl IntoPageKey) -> Option<u64> {
        let key = key.into_page_key();
        self.page_state.borrow().get(&key).copied()
    }

    /// Interprets the value stored at `key` as a 64-bit bitmask and returns a [`Vec`] of all
    /// bit indices `(0..64)` that are set (i.e. where bit `i` is 1).
    ///
    /// Returns an empty [`Vec`] if the key does not exist.
    pub fn get_mask_indices(&self, key: impl IntoPageKey) -> Vec<usize> {
        let key = key.into_page_key();
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
    pub fn get_mask<const N: usize>(&self, key: impl IntoPageKey) -> [bool; N] {
        let key = key.into_page_key();
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
    pub fn get_mask_last(&self, key: impl IntoPageKey) -> Option<u8> {
        let key = key.into_page_key();
        let val = self.page_state.borrow().get(&key).copied()?;

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
    pub fn remove_mask_last(&mut self, key: impl IntoPageKey) -> Option<u8> {
        let key = key.into_page_key();
        let val = self.page_state.borrow_mut().remove(&key)?;

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
    pub fn insert(&self, key: impl IntoPageKey, value: u64) {
        let key = key.into_page_key();
        self.page_state.borrow_mut().insert(key, value);
    }

    /// Removes `key` from the page's persistent state and returns its previous `u64` value,
    /// or `None` if the key was not present.
    pub fn remove(&self, key: impl IntoPageKey) -> Option<u64> {
        let key = key.into_page_key();
        self.page_state.borrow_mut().remove(&key)
    }

    /// Checks whether the value at `key` is currently `0`.
    ///
    /// If it is `0`, updates it to `1` and returns `true`. Otherwise returns `false`.
    /// Primarily used by `click!` to ensure a click handler executes only on the initial click.
    pub fn was_zero(&self, key: impl IntoPageKey) -> bool {
        let key = key.into_page_key();
        if let Some(x) = self.page_state.borrow_mut().get_mut(&key)
            && *x == 0
        {
            *x = 1;
            true
        } else {
            false
        }
    }

    /// Checks if the dirty bit (bit 63) is set for `key`.
    ///
    /// Claims a pending interaction for `key`.
    ///
    /// The dirty bit (bit 63) gates the claim. With `owner` as `None` the value below the dirty bit
    /// is a plain click counter, incremented by one. With `owner` as `Some(paragraph)` the value is a
    /// link value (`DIRTY_BIT | loc | count`, see [`crate::core::key`]): the claim additionally
    /// requires the location field to name `paragraph`, and the count field is incremented
    /// (saturating at [`LINK_COUNT_MASK`], so it never spills into the location bits).
    /// Returns `true` if the dirty bit was set and the count was `< max_clicks` before incrementing.
    /// `max_clicks` is the number of permitted clicks: `0` denies the claim, and larger values
    /// permit more clicks.
    /// A plain click is not incremented if it would carry into the topmost bit (bit 63).
    pub fn poll_click(
        &self,
        key: impl IntoPageKey,
        max_clicks: u64,
        owner: Option<PageKey>,
    ) -> bool {
        let key = key.into_page_key();
        if let Some(x) = self.page_state.borrow_mut().get_mut(&key)
            && (*x & DIRTY_BIT) != 0
        {
            let (clicks, next_val) = match owner {
                Some(paragraph) => {
                    // Links are keyed by their text, so only the paragraph whose location payload
                    // is stored with the pending click may claim it.
                    if (*x & LINK_LOC_MASK) != link_loc_bits(paragraph) {
                        return false;
                    }
                    let clicks = *x & LINK_COUNT_MASK;
                    let count = (clicks + 1).min(LINK_COUNT_MASK);
                    (clicks, (*x & !(DIRTY_BIT | LINK_COUNT_MASK)) | count)
                }
                None => {
                    let clicks = *x & !DIRTY_BIT;
                    let next_val = clicks.saturating_add(1);
                    let next_val = if (next_val & DIRTY_BIT) == 0 {
                        next_val
                    } else {
                        clicks
                    };
                    (clicks, next_val)
                }
            };
            *x = next_val;
            return clicks < max_clicks;
        }
        false
    }

    /// Attaches a tag within the given [`TagScope`].
    ///
    /// Returns `true` if the tag was newly inserted into the persisted game tags.
    pub fn tag(&mut self, s: &str, scope: TagScope) -> bool {
        let q: PageId = s.into();
        if scope != TagScope::Game {
            self.view.tags.push(q.clone());
        }
        if scope == TagScope::Once {
            return false;
        }
        self.game_tags.insert(q)
    }

    /// Removes a tag within the given [`TagScope`].
    ///
    /// Returns `true` if the tag was removed from the current view or the persisted game tags.
    pub fn untag(&mut self, s: &str, scope: TagScope) -> bool {
        let q: PageId = s.into();
        let mut removed = false;
        if scope != TagScope::Once {
            removed |= self.game_tags.remove(&q);
        }
        if scope != TagScope::Game {
            let before = self.view.tags.len();
            self.view.tags.retain(|t| *t != q);
            removed |= self.view.tags.len() != before;
        }
        removed
    }

    /// Reports whether `s` is present in the global [`GameTags`].
    pub fn is_tagged(&self, s: &str) -> bool {
        self.game_tags.contains(&s.into())
    }
}

/// Where a tag is recorded or removed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TagScope {
    /// The current render only.
    Once,
    /// The current render and the persisted game tags.
    Sticky,
    /// The persisted game tags only.
    Game,
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
    use crate::core::key::{HASH_KEY_BIT, hash_key};
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

        // Auto keys always have bit 63 = 0
        for k in &keys1 {
            assert_eq!(k & HASH_KEY_BIT, 0, "auto key must not have top bit set");
        }

        // String hash keys always have bit 63 = 1
        let hk = hash_key("my_variable");
        assert_eq!(
            hk & HASH_KEY_BIT,
            HASH_KEY_BIT,
            "hashed key must have top bit set"
        );
        assert_ne!(hk, hash_key("other_variable"));
    }

    #[test]
    fn test_link_value_layout() {
        use crate::core::game_state::GameState;
        use crate::core::key::{link_loc_bits, link_owner};

        let page: PageId = "test".into();
        let link = hash_key("target");
        let paragraph: PageKey = 6;
        let mut game_state = GameState::default();

        // A pending click stores the owner bits with the count still at zero
        game_state.set_dirty(&page, link, paragraph);
        {
            let mut tags = HashSet::new();
            let state = PageState::new(
                "test",
                false,
                false,
                game_state.get_page_mut(page.clone()),
                &mut tags,
            );
            let pending = state.get(link).unwrap();
            assert_eq!(pending & (DIRTY_BIT | LINK_LOC_MASK), link_owner(paragraph));
            assert_eq!(pending & LINK_COUNT_MASK, 0);

            // Another paragraph does not claim the pending click
            assert!(!state.poll_click(link, u64::MAX, Some(paragraph + 1)));
            assert_eq!(state.get(link), Some(link_owner(paragraph)));

            // The owning paragraph claims it once, clearing the dirty marker
            assert!(state.poll_click(link, u64::MAX, Some(paragraph)));
            let claimed = state.get(link).unwrap();
            assert_eq!(claimed & LINK_COUNT_MASK, 1);
            assert_eq!(claimed & DIRTY_BIT, 0);
            assert_eq!(claimed & LINK_LOC_MASK, link_loc_bits(paragraph));
            assert!(!state.poll_click(link, u64::MAX, Some(paragraph)));
        }

        // set_dirty overwrites the owner but preserves the count
        game_state.set_dirty(&page, link, paragraph + 2);
        {
            let mut tags = HashSet::new();
            let state = PageState::new(
                "test",
                false,
                false,
                game_state.get_page_mut(page.clone()),
                &mut tags,
            );
            let pending = state.get(link).unwrap();
            assert_eq!(
                pending & (DIRTY_BIT | LINK_LOC_MASK),
                link_owner(paragraph + 2)
            );
            assert_eq!(pending & LINK_COUNT_MASK, 1, "count survives set_dirty");

            // The new owner claims the click and increments the count
            assert!(state.poll_click(link, u64::MAX, Some(paragraph + 2)));
            assert_eq!(state.get(link).unwrap() & LINK_COUNT_MASK, 2);
        }
    }

    #[test]
    fn test_zero_max_clicks_denies_claim() {
        use crate::core::game_state::GameState;

        let page: PageId = "test".into();
        let link = hash_key("target");
        let paragraph: PageKey = 6;
        let mut game_state = GameState::default();

        game_state.set_dirty(&page, link, paragraph);

        let mut tags = HashSet::new();
        let state = PageState::new(
            "test",
            false,
            false,
            game_state.get_page_mut(page.clone()),
            &mut tags,
        );

        // A zero limit never reports a successful claim, and the pending click is drained so
        // it cannot fire if the limit later rises.
        assert!(!state.poll_click(link, 0, Some(paragraph)));
        assert_eq!(state.get(link).unwrap() & DIRTY_BIT, 0);
        assert_eq!(state.get(link).unwrap() & LINK_COUNT_MASK, 1);
    }

    #[test]
    fn test_link_count_saturates() {
        use crate::core::game_state::GameState;

        let page: PageId = "test".into();
        let link = hash_key("target");
        let paragraph: PageKey = 6;
        let mut game_state = GameState::default();

        game_state.insert(&page, link, LINK_COUNT_MASK);
        game_state.set_dirty(&page, link, paragraph);

        let mut tags = HashSet::new();
        let state = PageState::new(
            "test",
            false,
            false,
            game_state.get_page_mut(page.clone()),
            &mut tags,
        );
        assert!(state.poll_click(link, u64::MAX, Some(paragraph)));
        let saturated = state.get(link).unwrap();
        assert_eq!(saturated & LINK_COUNT_MASK, LINK_COUNT_MASK);
        assert_eq!(
            saturated & LINK_LOC_MASK,
            link_loc_bits(paragraph),
            "count must not spill into the location bits"
        );
        assert_eq!(saturated & DIRTY_BIT, 0);
    }
}
