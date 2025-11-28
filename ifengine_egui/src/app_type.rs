use story::saltwrack::{Game, new};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub inner: Game,
}

impl App {
    pub fn new() -> Self {
        Self { inner: new() }
    }
}

impl std::ops::Deref for App {
    type Target = Game;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
