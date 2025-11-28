use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::Game;
use crate::core::GameContext;
use crate::view::View;

pub type Page<C> = fn(&mut Game<C>) -> Response;

pub trait PageErased: Send + Sync {
    fn call(&self, game: &mut dyn Any) -> Response;
}

impl<C: GameContext> PageErased for Page<C> {
    fn call(&self, game: &mut dyn Any) -> Response {
        let game = game.downcast_mut::<Game<C>>().expect("Game type mismatch");
        self(game)
    }
}

pub type PageId = String;

pub enum Response {
    View(View),
    Switch(PageHandle),
    Back(usize),
    Tunnel(PageHandle),
    Exit,
    End, // thread?
}

#[derive(Clone)]
pub struct PageHandle {
    pub widget: Arc<dyn PageErased>,
    pub name: PageId,
}

impl PageHandle {
    pub fn new<C: GameContext>(name: PageId, widget: Page<C>) -> Self {
        Self {
            widget: Arc::new(widget), // no closure needed
            name,
        }
    }

    pub fn call<C: GameContext>(&self, game: &mut Game<C>) -> Response {
        self.widget.call(game as &mut dyn Any)
    }
}

// ----------------------- BOILERPLATE ---------------------------------------------------

impl fmt::Debug for PageHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Page").field("name", &self.name).finish()
    }
}
