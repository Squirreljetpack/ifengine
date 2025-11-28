use std::collections::HashMap;

use crate::GameError;
use crate::core::game_state::{GameState, InternalKey};
use crate::core::{Page, PageHandle, Response};
use crate::view::View;

/// Used to manage custom state
pub type StringMap = HashMap<String, String>;

pub trait GameContext: Default + std::fmt::Debug + 'static {}
impl<T> GameContext for T where T: Default + std::fmt::Debug + 'static {}

/// The core ifengine game object
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameInner {
    pub state: GameState,
    #[serde(skip)]
    pub pages: PageStack,
    pub fresh: bool,
    pub iterations: usize, // todo
}

/// When processing for rendering, use game.inner instead
/// The context is exposed to your pages allowing you to flexibly manage state however you want
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game<C: GameContext = StringMap> {
    pub inner: GameInner,
    pub context: C,
}

impl<C: GameContext> Game<C> {
    pub fn new_with_page(page_name: String, page: Page<C>) -> Self {
        let widget = PageHandle::new(page_name, page);

        let inner = GameInner {
            state: GameState::new(),
            pages: PageStack::new_with_page(widget),
            fresh: true,
            iterations: 0,
        };

        Self {
            context: Default::default(),
            inner,
        }
    }

    pub fn view(&mut self) -> Result<View, GameError> {
        let Some(mut page) = self.pages.current() else {
            return Err(GameError::NoPage);
        };

        if page.name.is_empty() {
            self.pages.clear() // drop the initial page for the next rendered (possibly same) page. In particular, it will have the fully resolved name.
        }

        let view = loop {
            let r = page.call(self);
            match r {
                Response::View(view) => {
                    page.name = view.name.clone();

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
                page.name = "".into(); // Only rendered pages go into history + this is not the full name
                self.pages.push(page)?;
            }
            Action::Back(n) => {
                self.pages.pop_n(n)?;
            }
            Action::Tunnel(mut page) => {
                self.pages.adv_stack();
                page.name = "".into(); // Only rendered pages go into history + this is not the full name
                self.pages.push(page)?;
            }
            Action::Exit => {
                self.pages.pop_stack().ok_or(GameError::End)?;
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! Game {
    ($f:path) => {
        $crate::core::Game::new_with_page("".into(), $f)
    };
}

// -------------- Page Stack -----------------

#[derive(Default, Debug)]
pub struct PageStack(Vec<Vec<PageHandle>>);

impl PageStack {
    fn new_with_page(page: PageHandle) -> Self {
        Self(vec![vec![page]])
    }

    pub fn current(&self) -> Option<PageHandle> {
        self.0.last()?.last().cloned()
    }

    pub fn push(&mut self, page: PageHandle) -> Result<(), GameError> {
        let stack = self.0.last_mut().ok_or(GameError::NoStack)?;

        let fresh = match stack.last() {
            Some(last) => last.name != page.name,
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

        if n <= stack.len() {
            stack.truncate(stack.len() - n);
            self.pop().ok_or(GameError::NoPage)
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

// -------------- Action --------------------
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Action {
    None,
    SetBit(InternalKey, u8),
    Set(InternalKey, u64),
    Inc(InternalKey),
    Reset(InternalKey),
    /// The name of the handle here is just for debug, and NOT guaranteed to be the actual id of the page, see [`crate::core::PageState`]
    Next(PageHandle), // Arc for easy cloning,
    Back(usize),
    Tunnel(PageHandle),
    Exit,
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
