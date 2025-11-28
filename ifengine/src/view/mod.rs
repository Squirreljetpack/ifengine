mod image;
mod line;
pub use image::*;
pub use line::*;

use crate::core::{PageId, game_state::PageKey};

#[derive(Debug, Clone)]
pub enum Object {
    /// A single line, rendered with wrapping.
    Text(Line),
    /// Text with single-spaced y-margin
    Paragraph(Line),
    /// A list of selectable texts which stores the selected index on click
    /// Like paragraph, this includes a single-spaced y-margin
    Choice(PageKey, Vec<(u8, Line)>),
    /// See [`Image`]
    Image(Image),
    /// Markdown heading
    Heading(Span, u8),
    /// <hr/>
    Break,
    /// empty lines
    Empty(u8),
    // Index into Span from Objects[View[Line]]
    Note(Line, (u8, u8)),
    // Quote style + optional string for quote variants/data
    Quote(Line, String),
}

/// The view returned by a [`crate::core::Page`].
/// Get it by calling [`crate::Game::view`]
/// The job of the ui library is to render this into a suitable format.
#[derive(Default, Debug)]
pub struct View {
    pub inner: Vec<Object>,
    pub name: PageId,
}

impl View {
    pub fn new(name: PageId) -> Self {
        Self {
            inner: vec![],
            name,
        }
    }

    pub fn name(&self) -> PageId {
        self.name.clone()
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
