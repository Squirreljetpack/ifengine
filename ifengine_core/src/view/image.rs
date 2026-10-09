use std::borrow::Cow;

use crate::{
    core::{Action, Actions},
    view::RenderData,
};

/// Image type.
/// See [`Object`](crate::view::Object).
#[derive(Debug, Clone)]
pub struct Image {
    pub size: [usize; 2],
    pub variant: ImageVariant,
    pub action: Actions,
    pub alt: String, // caption and alt_text
    pub render_data: RenderData,
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
            size: [0, 0],
            variant: ImageVariant::Url(url.into()),
            action: Actions::new(),
            alt: String::new(),
            render_data: "",
        }
    }

    pub fn new_local(path: impl Into<Cow<'static, str>>, bytes: &'static [u8]) -> Self {
        Image {
            size: [0, 0],
            variant: ImageVariant::Local(path.into(), bytes),
            action: Actions::new(),
            alt: String::new(),
            render_data: "",
        }
    }

    pub fn width(&self) -> usize {
        self.size[0]
    }

    pub fn height(&self) -> usize {
        self.size[1]
    }

    pub fn with_size(mut self, size: [usize; 2]) -> Self {
        self.size = size;
        self
    }

    pub fn with_alt(mut self, alt: String) -> Self {
        self.alt = alt;
        self
    }

    pub fn with_action(mut self, action: impl Into<Action>) -> Self {
        self.action.push(action.into());
        self
    }

    pub fn reset_actions(mut self) -> Self {
        self.action.clear();
        self
    }

    pub fn take_actions(&mut self) -> Option<Actions> {
        self.action.take()
    }
}
