use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::Game;
use crate::core::GameContext;
use crate::view::View;

/// Static functions. These implement [`PageErased`]
/// You can create one by annotating a bare fn(&mut C) with (`[#ifview]`)[crate::ifview]
pub type Page<C> = fn(&mut Game<C>) -> Response;

/// The trait which defines a page
/// Capable of (eventually) producing a [`View`] when (repeatedly) called by [`Game::view`]
pub trait PageErased: Send + Sync + 'static {
    fn call(&self, game: &mut dyn Any) -> Response;
}

impl<C: GameContext> PageErased for Page<C> {
    fn call(&self, game: &mut dyn Any) -> Response {
        let game = game.downcast_mut::<Game<C>>().expect("Game type mismatch");
        self(game)
    }
}
// todo: add struct impl

// newtype over alias just because arc doesn't have serialize, this is very annoying
// On the plus side makes typing a bit stronger...

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
/// Identifies a [`Page`]
/// Pairs with a [`PageErased`] to form a [`PageHandle`]
pub struct PageId(pub Arc<str>);

impl PageId {
    pub fn clear(&mut self) {
        self.0 = "".into()
    }
}

/// The type returned by a [`PageErased`]
/// [`Game::view`] will repeatedly call the active [page](PageHandle), until a [`View`] is produced.
pub enum Response {
    View(View),
    Switch(PageHandle),
    Back(usize),
    Tunnel(PageHandle),
    Exit,
    End, // thread?
}

/// Capable of (eventually) producing a [`View`]
#[derive(Clone)]
pub struct PageHandle {
    pub widget: Arc<dyn PageErased>,
    pub id: PageId,
}

impl iddqd::IdHashItem for PageHandle {
    type Key<'a> = &'a PageId;

    fn key(&self) -> Self::Key<'_> {
        &self.id
    }

    iddqd::id_upcast!();
}

impl PageHandle {
    pub fn new<C: GameContext>(id: PageId, widget: Page<C>) -> Self {
        Self {
            widget: Arc::new(widget), // no closure needed
            id,
        }
    }

    pub fn new_erased<T: PageErased>(id: PageId, widget: impl Into<T>) -> Self {
        Self {
            widget: Arc::new(widget.into()), // no closure needed
            id,
        }
    }

    pub fn call<C: GameContext>(&self, game: &mut Game<C>) -> Response {
        self.widget.call(game as &mut dyn Any)
    }
}

// ----------------------- BOILERPLATE ---------------------------------------------------

impl fmt::Debug for PageHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Page").field("name", &self.id).finish()
    }
}

impl<T: Into<Arc<str>>> From<T> for PageId {
    fn from(s: T) -> Self {
        PageId(s.into())
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for PageId {
        fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            s.serialize_str(&self.0)
        }
    }

    impl<'de> Deserialize<'de> for PageId {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s: &str = Deserialize::deserialize(d)?;
            Ok(PageId::from(s))
        }
    }
}
impl std::ops::Deref for PageId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
