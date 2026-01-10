//! The [`View`] and its constituents.

mod image;
mod line;

pub use image::*;
pub use line::*;

#[allow(unused)]
use crate::core::{Page, PageId, game_state::PageKey};

/// Some [`Object`] variants may want additional data to specify custom styles for the frontend.
/// That can be specified here.
///
/// Most variants don't include this in order to encourage a more unified ui experience.
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
    /// A list of selectable texts which stores the selected index on click.
    ///
    /// Like the paragraph variant, this includes a single-spaced y-margin
    Choice(PageKey, Vec<(u8, Line)>),
    /// See [`Image`]
    Image(Image),
    /// Markdown heading
    Heading(Span, u8),
    /// Horizontal line
    /// `<hr/>`
    Break,
    /// empty lines.
    Empty(u8),
    /// Represents different types of notes.
    ///
    /// Inputs:
    ///   - `Line`: The content to display.
    ///   - `(u8, u8)`: Indices into a `Span` from `View[Line[Span]]`, e.g., for annotations.
    Note(Line, (u8, u8)),
    /// Quote style.
    Quote(Line, RenderData),
    /// Custom marker.
    /// For example, can be used signal to the frontend to play music when this object enters the screen.
    Custom(RenderData),
}

/// The view returned by a [`Page`].
///
/// The job of the ui library is to process its objects (i.e. by rendering).
///
/// # Additional
/// Produced [`crate::Game::view`].
#[derive(Default, Debug)]
pub struct View {
    pub inner: Vec<Object>,
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
    type Target = Vec<Object>;
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
    type Item = Object;
    type IntoIter = std::vec::IntoIter<Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a View {
    type Item = &'a Object;
    type IntoIter = std::slice::Iter<'a, Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut View {
    type Item = &'a mut Object;
    type IntoIter = std::slice::IterMut<'a, Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}
