use std::collections::{HashMap, HashSet};

use crate::{Action, GameError};
use crate::core::game_state::{GameState, InternalKey};
use crate::core::{Page, PageHandle, PageId, Response};
use crate::view::View;

/// Used to manage custom state
pub type StringMap = HashMap<String, String>;
/// Stores tags (See [`crate::core::PageState`])
pub type GameTags = HashSet<PageId>; // just want the Arc<str>

pub trait GameContext: Default + Clone + std::fmt::Debug + 'static {}
impl<T> GameContext for T where T: Default + Clone + std::fmt::Debug + 'static {}

/// The core ifengine game object
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameInner {
    pub state: GameState,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub pages: PageStack,
    pub fresh: bool,
    pub iterations: usize, // todo
}

/// Wraps [`GameInner`] with customizable a context used to represent the game-specific state.
///
/// The context is exposed to your [pages](crate::core::Page), allowing you to interact with your game state within them.
///
/// # Example
/// ```rust
/// use ifengine::{GameError, View};
/// use story::chap1;
///
/// let game = ifengine::Game!(chap1::p1);
///
/// let view = match game.view() {
///    Ok(view: View) => view,
///    Err(e: GameError) => {
///        panic!("Unhandled err: {e}");
///    }
/// };
///
/// ui.render(view, &mut game);
///
/// ```
///
/// # Additional
/// When processing for rendering, you can use the inner field directly.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game<C = StringMap> {
    pub inner: GameInner,
    pub context: C,
    pub tags: GameTags,
    pub(crate) simulating: bool
}

impl<C: GameContext> Game<C> {
    pub fn new_with_page(page_name: impl Into<PageId>, page: Page<C>) -> Self {
        let widget = PageHandle::new(page_name.into(), page);

        let inner = GameInner {
            state: GameState::new(),
            pages: PageStack::new_with_page(widget),
            fresh: true,
            iterations: 0,
        };

        Self {
            context: Default::default(),
            tags: Default::default(),
            inner,
            simulating: false
        }
    }

    pub fn simulating(&self) -> bool {
        self.simulating
    }

    /// Calls the active [page](PageHandle) in a loop, until a [`View`] is produced.
    pub fn view(&mut self) -> Result<View, GameError> {
        let Some(mut page) = self.pages.current() else {
            return Err(GameError::NoPage);
        };

        if page.id.is_empty() {
            self.pages.pop(); // drop the initial page for the next rendered and (possibly same) page. In particular, it will have the fully resolved name, (while i.e. the pagehandles produced by link! in handle_action don't).
        }

        let view = loop {
            let r = page.call(self);
            match r {
                Response::View(view) => {
                    page.id = view.pageid.clone(); // id the page by the fully resolved name

                    self.pages.push(page)?; // only rendered pages get added to history
                    break view;
                }
                Response::Switch(next) => {
                    page = next;
                }
                Response::Back(n) => {
                    page = self.pages.pop_n(n)?;
                }
                Response::Tunnel(next) => {
                    self.pages.adv_stack();
                    page = next;
                }
                Response::Exit => {
                    self.pages.pop_stack().ok_or(GameError::End)?;
                    page = self.pages.current().ok_or(GameError::NoPage)?;
                }
                Response::End => return Err(GameError::End),
            }
        };

        Ok(view)
    }

    pub fn id(&self) -> Option<PageId> {
        let mut test = self.clone();


        let Response::View(view) = self.pages.current()?.call(&mut test) else {
            return None;
        };

        Some(view.pageid)
    }
}

impl GameInner {
    // --------------- action handling -----------------------
    pub fn handle_choice(&mut self, key: InternalKey, index: u8) {
        self.state.set_bit(key, index)
    }

    pub fn handle_action(&mut self, action: Action) -> Result<(), GameError> {
        match action {
            Action::None => {}
            Action::SetBit(k, v) => {
                self.state.set_bit(k, v);
            }
            Action::Set(k, v) => {
                self.state.insert(k, v);
            }
            Action::Inc(k) => {
                self.state.inc(&k);
            }
            Action::Reset(k) => {
                self.state.remove(&k);
            }
            Action::Next(mut page) => {
                page.id = "".into(); // Only rendered pages go into history + this is not the full name
                self.pages.push(page)?;
            }
            Action::Back(n) => {
                self.pages.pop_n(n)?;
            }
            Action::Tunnel(mut page) => {
                self.pages.adv_stack();
                page.id = "".into(); // Only rendered pages go into history + this is not the full name
                self.pages.push(page)?;
            }
            Action::Exit => {
                self.pages.pop_stack().ok_or(GameError::End)?;
            }
        }
        Ok(())
    }
}

/// Instantiate a [`Game`] from a function decorated with [`crate::ifview`].
///
/// # Example
/// ```rust
/// use story::chap1;
///
/// pub type Game = ifengine::Game<State>;
/// pub fn new() -> Game {
///     ifengine::Game!(chap1::p1)
/// }
/// ```
#[macro_export]
macro_rules! Game {
    ($f:path) => {
        $crate::core::Game::new_with_page("", $f)
    };
}

// -------------- Page Stack -----------------

/// The struct representing game history.
/// Each tunnel is a seperate stack.
#[derive(Default, Debug, Clone)]
pub struct PageStack(Vec<Vec<PageHandle>>);

impl PageStack {
    pub fn new_with_page(page: PageHandle) -> Self {
        Self(vec![vec![page]])
    }

    pub fn current(&self) -> Option<PageHandle> {
        self.0.last()?.last().cloned()
    }

    pub fn current_mut(&mut self) -> Option<&mut PageHandle> {
        self.0.last_mut()?.last_mut()
    }

    pub fn push(&mut self, page: PageHandle) -> Result<(), GameError> {
        let stack = self.0.last_mut().ok_or(GameError::NoStack)?;

        let fresh = match stack.last() {
            Some(last) => last.id != page.id,
            None => true,
        };

        if fresh {
            stack.push(page);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        let Some(stack) = self.0.last_mut() else {
            return;
        };
        stack.clear();
    }

    pub fn len(&self) -> usize {
        let Some(stack) = self.0.last() else {
            return 0;
        };
        stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn pop(&mut self) -> Option<PageHandle> {
        self.0.last_mut()?.pop()
    }

    pub fn pop_n(&mut self, n: usize) -> Result<PageHandle, GameError> {
        let stack = self.0.last_mut().ok_or(GameError::NoStack)?;

        if n < stack.len() {
            stack.truncate(stack.len() - n);
            stack.last().cloned().ok_or(GameError::NoPage)
        } else {
            Err(GameError::NoPage)
        }
    }

    // ------------------------------
    // Stack frame management
    // ------------------------------

    pub fn adv_stack(&mut self) {
        self.0.push(vec![]);
    }

    pub fn pop_stack(&mut self) -> Option<Vec<PageHandle>> {
        self.0.pop()
    }
}


// ----------------------- BOILERPLATE ---------------------------------------------------
// impl Game {
//     pub fn test_1() -> Self {
//         // note: put the parameter explicitly
//         let initial_widget = |_app: &mut Game| {
//             let mut view = View::default();
//             view.inner
//                 .push(Object::Paragraph(Line::from_iter(["text1", "text2"])));
//             view.inner.push(Object::Heading("text1".into(), 3));
//             view.inner
//                 .push(Object::Paragraph(Line::from_iter(["text1", "text2"])));

//             view.inner.push(Object::Break);

//             view.inner.push(Object::Choice(32, vec![]));
//             view.inner.push(Object::Image(Image::new_url("https://upload.wikimedia.org/wikipedia/commons/thumb/b/b6/SIPI_Jelly_Beans_4.1.07.tiff/lossy-page1-256px-SIPI_Jelly_Beans_4.1.07.tiff.jpg".to_string())));
//             Response::View(view)
//         };
//         Game::new_with_page("name".into(), initial_widget)
//     }
// }

impl<C: GameContext> std::ops::Deref for Game<C> {
    type Target = GameInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C: GameContext> std::ops::DerefMut for Game<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

