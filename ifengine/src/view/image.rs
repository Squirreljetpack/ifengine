use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Image {
    Url(String, [usize; 2]),
    Local(PathBuf, [usize; 2]), // unimplemented
}

impl Image {
    pub fn new_url(url: impl Into<String>) -> Self {
        Image::Url(url.into(), [0, 0])
    }

    pub fn new_local(path: impl Into<PathBuf>) -> Self {
        Image::Local(path.into(), [0, 0])
    }

    pub fn width(&self) -> usize {
        match self {
            Image::Url(_, size) => size[0],
            Image::Local(_, size) => size[0],
        }
    }

    pub fn height(&self) -> usize {
        match self {
            Image::Url(_, size) => size[1],
            Image::Local(_, size) => size[1],
        }
    }
}
