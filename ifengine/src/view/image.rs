use std::path::PathBuf;

use crate::core::Action;

#[derive(Debug, Clone)]
pub struct Image {
    pub size: [usize; 2],
    pub variant: ImageVariant,
    pub action: Option<Action>,
    pub alt: String, // caption and alt_text
}



#[derive(Debug, Clone)]
pub enum ImageVariant {
    Url(String),
    Local(PathBuf), // unimplemented
}

impl Image {
    pub fn new_url(url: impl Into<String>) -> Self {
        Image {
            size: [0, 0],
            variant: ImageVariant::Url(url.into()),
            action: None,
            alt: String::new()
        }
    }

    pub fn new_local(url: impl Into<PathBuf>) -> Self {
        Image {
            size: [0, 0],
            variant: ImageVariant::Local(url.into()),
            action: None,
            alt: String::new()
        }
    }

    pub fn width(&self) -> usize {
        self.size[0]
    }

    pub fn height(&self) -> usize {
        self.size[1]
    }

    pub fn with_alt(mut self, alt: String) -> Self {
        self.alt = alt;
        self
    }
}
