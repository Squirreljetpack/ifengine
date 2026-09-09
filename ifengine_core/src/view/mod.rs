//! The [`View`] and its constituents.

mod image;
mod line;

pub use image::*;
pub use line::*;

use crate::core::{PageId, game_state::PageKey};

/// Optional styling or layout metadata attached to an [`Object`] for consumption by custom frontends.
pub type RenderData = &'static str;

/// An object within a [`View`].
///
/// The frontend is responsible for the display of each variant, but should adhere to their description in doing so.
#[derive(Debug, Clone)]
pub enum Object {
    /// A single line, rendered with wrapping, carrying optional data which can be used for customization by the frontend.
    ///
    /// N.B. spans are allowed to carry newlines.
    Text(Line, RenderData),
    /// Text with a single-spaced y-margin.
    Paragraph(Line),
    /// A list of selectable choices which stores the selected index on click.
    ///
    /// Like the paragraph variant, this includes a single-spaced y-margin.
    Choice(Vec<(u8, Line)>),
    /// An embedded image object.
    Image(Image),
    /// Markdown heading with text content and level (1-6).
    Heading(Span, u8),
    /// Horizontal divider line (`<hr/>`).
    Break,
    /// Vertical whitespace consisting of `n` empty lines.
    Empty(u8),
    /// Represents different types of notes.
    ///
    /// Inputs:
    ///   - `Line`: The content to display.
    ///   - `(u8, u8)`: Indices into a `Span` from `View[Line[Span]]`, e.g., for annotations.
    Note(Line, (u8, u8)),
    /// Quoted block style.
    Quote(Line, RenderData),
    /// An embedded sub-page view with optional custom styling or presentation metadata (e.g. modal).
    /// An empty embedded view serves the role of an extensible custom marker.
    Embed(View, RenderData),
}

impl Object {
    /// Computes a 64-bit hash representing the content of this object,
    /// used by frontend transition managers to detect content changes across renders.
    pub fn content_hash(&self) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};
        match self {
            Object::Paragraph(line) => line.content_hash(),
            Object::Text(line, render_data) => {
                let mut hasher = DefaultHasher::new();
                line.content_hash().hash(&mut hasher);
                render_data.hash(&mut hasher);
                match hasher.finish() {
                    0 => 1,
                    h => h,
                }
            }
            Object::Choice(_) => 0,
            Object::Image(img) => {
                let mut hasher = DefaultHasher::new();
                match &img.variant {
                    ImageVariant::Url(url) => url.hash(&mut hasher),
                    ImageVariant::Local(uri, _bytes) => uri.hash(&mut hasher),
                }
                img.alt.hash(&mut hasher);
                match hasher.finish() {
                    0 => 1,
                    h => h,
                }
            }
            Object::Heading(span, level) => {
                let mut hasher = DefaultHasher::new();
                span.content_hash().hash(&mut hasher);
                level.hash(&mut hasher);
                match hasher.finish() {
                    0 => 1,
                    h => h,
                }
            }
            Object::Break => 1,
            Object::Empty(n) => *n as u64 + 1,
            Object::Note(line, indices) => {
                let mut hasher = DefaultHasher::new();
                line.content_hash().hash(&mut hasher);
                indices.hash(&mut hasher);
                match hasher.finish() {
                    0 => 1,
                    h => h,
                }
            }
            Object::Quote(line, render_data) => {
                let mut hasher = DefaultHasher::new();
                line.content_hash().hash(&mut hasher);
                render_data.hash(&mut hasher);
                match hasher.finish() {
                    0 => 1,
                    h => h,
                }
            }
            Object::Embed(embedded_view, render_data) => {
                let mut hasher = DefaultHasher::new();
                embedded_view.pageid.0.hash(&mut hasher);
                embedded_view.inner.len().hash(&mut hasher);
                render_data.hash(&mut hasher);
                match hasher.finish() {
                    0 => 1,
                    h => h,
                }
            }
        }
    }

    /// Alias for [`content_hash`](Self::content_hash).
    pub fn hash_content(&self) -> u64 {
        self.content_hash()
    }
}

/// An [`Object`] tagged with an optional [`PageKey`] identifying its element boundary for state tracking and UI transitions.
#[derive(Debug, Clone)]
pub struct StampedObject {
    pub id: Option<PageKey>,
    pub object: Object,
}

impl StampedObject {
    pub fn new(object: Object) -> Self {
        Self { id: None, object }
    }

    pub fn with_id(mut self, id: PageKey) -> Self {
        self.id = Some(id);
        self
    }

    /// Computes a 64-bit hash representing the content of this object.
    pub fn content_hash(&self) -> u64 {
        self.object.content_hash()
    }

    /// Alias for [`content_hash`](Self::content_hash).
    pub fn hash_content(&self) -> u64 {
        self.object.hash_content()
    }
}

impl From<Object> for StampedObject {
    fn from(object: Object) -> Self {
        Self::new(object)
    }
}

impl std::ops::Deref for StampedObject {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl std::ops::DerefMut for StampedObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

/// The resolved visual representation of a page returned by [`Game::view`](crate::Game::view).
///
/// A `View` contains an ordered list of [`StampedObject`]s to be rendered by the frontend,
/// along with the active page's [`PageId`] and any associated tags.
///
/// `View` implements [`Deref<Target = Vec<StampedObject>>`](std::ops::Deref) and [`IntoIterator`],
/// allowing direct iteration over its elements.
#[derive(Default, Debug, Clone)]
pub struct View {
    pub inner: Vec<StampedObject>,
    pub pageid: PageId,
    pub tags: Vec<PageId>,
}

impl View {
    pub fn new(name: PageId) -> Self {
        Self {
            inner: vec![],
            pageid: name,
            tags: vec![],
        }
    }

    pub fn name(&self) -> PageId {
        self.pageid.clone()
    }
}

// --------------- BOILERPLATE ----------------

impl std::ops::Deref for View {
    type Target = Vec<StampedObject>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for View {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl IntoIterator for View {
    type Item = StampedObject;
    type IntoIter = std::vec::IntoIter<StampedObject>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a View {
    type Item = &'a StampedObject;
    type IntoIter = std::slice::Iter<'a, StampedObject>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut View {
    type Item = &'a mut StampedObject;
    type IntoIter = std::slice::IterMut<'a, StampedObject>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}
