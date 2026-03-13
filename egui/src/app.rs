use egui_snarl::Snarl;
pub use story::{Game, new};

use crate::graph::Node;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct App {
    pub game: Game,
    pub state: GUIState,
}

impl App {
    pub fn new() -> Self {
        Self {
            game: new(),
            state: GUIState::new(),
        }
    }

    pub fn header(&self) -> Vec<String> {
        if self.game.context.miles != 0 {
            vec![
                format!("Day: {}", self.game.context.days),
                format!("Miles travelled: {}", self.game.context.miles),
                format!("Rations: {}", self.game.context.rations),
            ]
        } else {
            vec![]
        }
    }
}

// -----------------
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GUIState {
    pub show_menu_button: bool,
    pub show_menu: bool,
    pub show_graph: bool,

    pub transitioning: bool,

    pub graph_viewer: Option<(Snarl<Node>, crate::graph::GraphViewer)>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub last_view: Option<ifengine::View>,
    pub fade_duration: [f32; 2],
}

impl GUIState {
    pub fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            fade_duration: [0.0; 2],
            #[cfg(not(debug_assertions))]
            fade_duration: [0.5; 2],
            ..Default::default()
        }
    }
}

// ----------------- BOILERPLATE -----------------

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
