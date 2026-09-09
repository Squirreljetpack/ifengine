use std::borrow::Cow;

use crate::core::{Action, game_state::PageKey};

/// Image type.
/// See [`Object`](crate::view::Object).
#[derive(Debug, Clone)]
pub struct Image {
    pub id: Option<PageKey>,
    pub size: [usize; 2],
    pub variant: ImageVariant,
    pub action: Option<Action>,
    pub alt: String, // caption and alt_text
}

/// Local or Remote image.
/// See [`Image`].
#[derive(Debug, Clone)]
pub enum ImageVariant {
    Url(String),
    Local(Cow<'static, str>, &'static [u8]), // unimplemented
}

impl Image {
    pub fn new_url(url: impl Into<String>) -> Self {
        Image {
            id: None,
            size: [0, 0],
            variant: ImageVariant::Url(url.into()),
            action: None,
            alt: String::new(),
        }
    }

    pub fn new_local(path: impl Into<Cow<'static, str>>, bytes: &'static [u8]) -> Self {
        Image {
            id: None,
            size: [0, 0],
            variant: ImageVariant::Local(path.into(), bytes),
            action: None,
            alt: String::new(),
        }
    }

    pub fn width(&self) -> usize {
        self.size[0]
    }

    pub fn height(&self) -> usize {
        self.size[1]
    }

    pub fn with_id(mut self, id: PageKey) -> Self {
        if self.id.is_none() {
            self.id = Some(id);
        }
        self
    }

    pub fn with_size(mut self, size: [usize; 2]) -> Self {
        self.size = size;
        self
    }

    pub fn with_alt(mut self, alt: String) -> Self {
        self.alt = alt;
        self
    }
}
