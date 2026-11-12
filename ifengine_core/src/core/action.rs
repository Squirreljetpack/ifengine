use crate::core::{PageHandle, PageKey};

/// Adds an effect to a [`Span`](crate::view::Span)
/// Spans with an action occlude their containing object (i.e. [`Choice`](crate::view::Object::Choice))
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Action {
    #[default]
    None,
    SetBit(PageKey, u8),
    Set(PageKey, u64),
    SetDirty(PageKey, u64),
    Inc(PageKey),
    Reset(PageKey),
    /// The name of the handle here is purely descriptive, and NOT guaranteed to be the actual id of the page, see [`PageState`](crate::core::PageState)
    Next(PageHandle), // Arc for easy cloning
    Back(usize),
    Tunnel(PageHandle),
    Exit,
}

/// A sequence of actions associated with an element.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Actions(pub Vec<Action>);

impl Actions {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn single(action: impl Into<Action>) -> Self {
        Self(vec![action.into()])
    }

    pub fn take(&mut self) -> Option<Self> {
        if self.0.is_empty() {
            None
        } else {
            Some(std::mem::take(self))
        }
    }
}

impl std::ops::Deref for Actions {
    type Target = Vec<Action>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Actions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Actions {
    type Item = Action;
    type IntoIter = std::vec::IntoIter<Action>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Actions {
    type Item = &'a Action;
    type IntoIter = std::slice::Iter<'a, Action>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Vec<Action>> for Actions {
    fn from(actions: Vec<Action>) -> Self {
        Self(actions)
    }
}

impl From<Action> for Actions {
    fn from(action: Action) -> Self {
        Self(vec![action])
    }
}

impl From<&Action> for Actions {
    fn from(action: &Action) -> Self {
        Self(vec![action.clone()])
    }
}

impl From<&Actions> for Actions {
    fn from(actions: &Actions) -> Self {
        actions.clone()
    }
}

impl From<&[Action]> for Actions {
    fn from(actions: &[Action]) -> Self {
        Self(actions.to_vec())
    }
}

impl From<Actions> for Vec<Action> {
    fn from(actions: Actions) -> Self {
        actions.0
    }
}

impl Extend<Action> for Actions {
    fn extend<T: IntoIterator<Item = Action>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl FromIterator<Action> for Actions {
    fn from_iter<T: IntoIterator<Item = Action>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
