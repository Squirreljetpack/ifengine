use std::cell::RefCell;

use crate::{
    core::{
        GameTags, PageId, Response,
        game_state::{PageKey, PageMap},
    },
    view::{Object, View},
};

/// The [`crate::ifview`] decorator instantiates this from a reference to [`struct@crate::Game`], using it to add [elements](crate::elements) which read and write to [`crate::core::game_state::GameState`].
/// Will produce a [`View`] if the decorated function doesn't exit early.
#[derive(Debug)]
pub struct PageState<'a> {
    view: View,
    page_state: RefCell<&'a mut PageMap>, // to allow simultaneous method accesses, safe because chapter_state doesn't produce refs
    #[cfg(feature = "rand")]
    pub seed: Option<u64>,
    fresh: bool,
    game_tags: &'a mut GameTags,
    /// [`crate::Game::simulate`]
    pub simulating: bool,
}

impl<'a> PageState<'a> {
    pub fn new(
        name: impl Into<PageId>,
        page_state: &'a mut PageMap,
        game_tags: &'a mut GameTags,
        fresh: bool,
        simulating: bool,
    ) -> Self {
        Self {
            view: View::new(name.into()),
            page_state: RefCell::new(page_state),
            #[cfg(feature = "rand")]
            seed: None,
            fresh,
            game_tags,
            simulating,
        }
    }
}

impl<'a> PageState<'a> {
    pub fn push(&mut self, object: Object) {
        self.view.push(object);
    }

    pub fn id(&self) -> PageId {
        self.view.pageid.clone()
    }

    pub fn into_response(self) -> Response {
        Response::View(self.view)
    }

    pub fn fresh(&self) -> bool {
        self.fresh
    }

    // --------- Chapter state
    // indexing takes owned for convenience (PageKey is copy)
    pub fn get(&self, key: PageKey) -> Option<u64> {
        self.page_state.borrow().get(&key).copied()
    }

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

    pub fn get_mask<const N: usize>(&self, key: PageKey) -> [bool; N] {
        let val = match self.page_state.borrow().get(&key).copied() {
            Some(v) => v,
            None => return [false; N],
        };

        std::array::from_fn(|i| i < 64 && (val & (1 << i)) != 0)
    }

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

    pub fn insert(&self, key: PageKey, value: u64) {
        self.page_state.borrow_mut().insert(key, value);
    }

    pub fn remove(&self, key: PageKey) -> Option<u64> {
        self.page_state.borrow_mut().remove(&key)
    }

    pub fn tag(&mut self, s: &str) -> bool {
        let q: PageId = s.into();
        self.view.tags.push(q.clone());
        self.game_tags.insert(q)
    }

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
