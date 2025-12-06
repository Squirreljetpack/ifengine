use std::cell::OnceCell;

use egui_snarl::Snarl;
use ifengine::run::PageRecord;
use serde::{Deserialize, Serialize};
use story::saltwrack::{new, Game};

use crate::{graph::{Node, new_snarl}, theme::Theme};

#[derive(Serialize, Deserialize)]
pub struct App {
    pub game: Game,
    pub state: GUIState,
}

impl App {
    pub fn new() -> Self {
        Self {
            game: new(),
            state: GUIState::default(),
        }
    }
}

// -----------------
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GUIState {
    pub show_menu_button: bool,
    pub show_menu: bool,
    pub show_graph: bool,

    pub graph_viewer: Option<(Snarl<Node>, crate::graph::GraphViewer)>,

}

impl GUIState {
}

// -----------------

impl std::ops::Deref for App {
    type Target = GUIState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl std::ops::DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

//This is to make App debuggable, as Snarl is not debuggable
impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
        .field("game", &self.game)
        .field("state", &self.state)
        .finish()
    }
}
