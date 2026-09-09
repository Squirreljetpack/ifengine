use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::Game;
use crate::core::GameContext;
use crate::view::View;

/// Static functions. These implement [`PageErased`].
/// You can create one by annotating a bare fn(&mut C) with `#[ifview]`.
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
// todo: add struct that implements this, i.e. parsed from dsl

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
/// Identifies a [`Page`]
/// Pairs with a [`PageErased`] to form a [`PageHandle`]
pub struct PageId(pub Arc<str>);

impl PageId {
    pub fn clear(&mut self) {
        self.0 = "".into()
    }
}

/// The transition or output produced by a [`Page`].
///
/// [`Game::view`] will repeatedly execute transitions until a [`Response::View`] is produced.
pub enum Response {
    /// Yields the rendered [`View`] for the page.
    View(View),
    /// Immediately transitions to the specified destination page.
    Switch(PageHandle),
    /// Navigates back `n` steps in navigation history.
    Back(usize),
    /// Calls another page as a subroutine / tunnel, returning here when finished.
    Tunnel(PageHandle),
    /// Exits the current tunnel and resumes the calling page.
    Exit,
    /// Terminates story execution.
    End,
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

/// Compile-time registration entry for pages decorated with `#[ifview]`.
pub struct RegisteredPage {
    pub id: &'static str,
    pub factory: fn(PageId) -> PageHandle,
}

inventory::collect!(RegisteredPage);

/// Resolves a page identifier against the global compile-time registry.
pub fn resolve_page(id: &str) -> Option<PageHandle> {
    for page in inventory::iter::<RegisteredPage> {
        if page.id == id {
            return Some((page.factory)(page.id.into()));
        }
    }
    None
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

    struct PageIdVisitor;

    impl<'de> serde::de::Visitor<'de> for PageIdVisitor {
        type Value = PageId;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representing a page id")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(PageId::from(v))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(PageId::from(v))
        }
    }

    impl<'de> Deserialize<'de> for PageId {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            d.deserialize_str(PageIdVisitor)
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
