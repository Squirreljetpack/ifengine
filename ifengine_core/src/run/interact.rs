use std::borrow::Cow;

use crate::{
    Action, Game, GameError, View,
    core::{GameContext, PageId, game_state::PageKey},
    view::{Line, Object, Span},
};

/// An interactive target extracted from a [`View`] that can trigger a state mutation or navigation.
#[derive(Debug, Clone, Copy)]
pub enum Interactable<'a> {
    /// A choice item identified by its parent [`PageKey`], the full choice line list, and the choice index.
    Choice(&'a PageKey, &'a Vec<(u8, Line)>, u8),
    /// A clickable span within an [`Object`].
    Span(&'a Object, &'a Span),
}

impl<'a> Interactable<'a> {
    /// Returns the text content of the interactable element.
    pub fn content(&self) -> Cow<'a, str> {
        match self {
            Interactable::Choice(_, lines, idx) => lines
                .iter()
                .find_map(|(i, x)| if i == idx { Some(x) } else { None })
                .unwrap()
                .content()
                .into(),
            Interactable::Span(_, s) => Cow::Borrowed(&s.content),
        }
    }
}

impl View {
    /// Nested interactables: each Object maps to a vector of interactables.
    /// Choice objects expand to:
    ///   [Interactable::Choice(parent, idx),
    ///    Interactable::Span(parent, span…),
    ///    Interactable::Span(parent, span…),
    ///    ...]
    ///
    /// Choices which contain an interactable element are ignored!
    pub fn interactables(&self) -> Vec<Vec<Interactable<'_>>> {
        let mut out = Vec::new();

        for obj in &self.inner {
            let mut bucket = Vec::new();

            match obj {
                Object::Text(line, _)
                | Object::Paragraph(line)
                | Object::Note(line, _)
                | Object::Quote(line, _) => {
                    for span in &line.spans {
                        if span.action.is_some() {
                            bucket.push(Interactable::Span(obj, span));
                        }
                    }
                }

                Object::Heading(span, _level) => {
                    if span.action.is_some() {
                        bucket.push(Interactable::Span(obj, span));
                    }
                }

                Object::Choice(key, choices) => {
                    for (i, line) in choices {
                        let ignore = {
                            if true {
                                line.spans.iter().any(|span| span.action.is_some())
                                    || !line.spans.is_empty()
                            } else {
                                // all spans have actions
                                line.spans.iter().all(|span| span.action.is_some())
                            }
                        };

                        if !ignore {
                            bucket.push(Interactable::Choice(&key, &choices, *i));
                        }

                        for span in &line.spans {
                            if span.action.is_some() {
                                bucket.push(Interactable::Span(obj, span));
                            }
                        }
                    }
                }

                Object::Embed(sub_view) => {
                    for nested_bucket in sub_view.interactables() {
                        bucket.extend(nested_bucket);
                    }
                }

                Object::Image(_) | Object::Break | Object::Empty(_) | Object::Custom(_) => {
                    // no interactables
                }
            }

            out.push(bucket);
        }

        out
    }

    /// Flatten all nested interactables into one linear Vec
    /// Filters out None Actions
    pub fn interactables_sim(&self) -> Vec<Interactable<'_>> {
        self.interactables()
            .into_iter()
            .flat_map(|v| {
                v.into_iter().filter(|e| {
                    if let Interactable::Span(_, span) = e
                        && (span.no_sim || matches!(span.action, Some(Action::None)))
                    {
                        false
                    } else {
                        true
                    }
                })
            })
            .collect()
    }
}

impl<C: GameContext> Game<C> {
    /// Applies an interaction event on the game state for the given page ID.
    pub fn interact(&mut self, e: Interactable<'_>, _pageid: &PageId) -> Result<(), GameError> {
        match e {
            Interactable::Choice(key, _, index) => {
                self.handle_choice(*key, index);
                Ok(())
            }
            Interactable::Span(_, s) => {
                let action = s.action.as_ref().unwrap();
                self.handle_action(action.clone()).map(|_| {})
            }
        }
    }

    /// Forks the game state for each interactable in the view and applies it.
    pub fn interact_all<F>(&self, view: View) -> Vec<Result<Self, GameError>> {
        view.interactables_sim()
            .into_iter()
            .map(|e| {
                let mut g = self.clone();
                g.interact(e, &view.pageid).map(|_| g)
            })
            .collect()
    }
}
