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
    Choice(PageKey, Vec<(u8, Line)>),
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
    /// Custom marker.
    /// For example, can be used to signal the frontend to play music when this object enters the screen.
    Custom(RenderData),
    /// An embedded sub-page view.
    Embed(View),
}

/// The resolved visual representation of a page returned by [`Game::view`](crate::Game::view).
///
/// A `View` contains an ordered list of [`Object`]s to be rendered by the frontend,
/// along with the active page's [`PageId`] and any associated tags.
///
/// `View` implements [`Deref<Target = [Object]>`](std::ops::Deref) and [`IntoIterator`],
/// allowing direct iteration over its elements.
///
/// # Example
/// ```rust,ignore
/// let view = game.view()?;
/// println!("Page: {}", view.pageid.0);
///
/// for object in &view {
///     match object {
///         Object::Paragraph(line) => render_line(line),
///         Object::Choice(key, choices) => render_choices(key, choices),
///         Object::Heading(span, level) => render_heading(span, *level),
///         Object::Break => render_divider(),
///         _ => {}
///     }
/// }
/// ```
#[derive(Default, Debug, Clone)]
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
