use std::borrow::Cow;

use crate::{
    Action, Game, GameError, SimEnd, View,
    core::{GameContext, PageHandle, PageId, PageStack, game_state::PageKey},
    view::{Line, Object, Span},
};

#[derive(Debug, Clone, Copy)]
pub enum Interactable<'a> {
    Choice(&'a PageKey, &'a Vec<(u8, Line)>, u8),     // parent, index of the choice
    Span(&'a Object, &'a Span), // parent, the span
}

impl<'a> Interactable<'a> {
    pub fn content(&self) -> Cow<'a, str> {
        match self {
            Interactable::Choice(_, lines, idx) => {
                lines.iter().find_map(|(i, x)| if i == idx { Some(x) } else { None }).unwrap().content().into()
            }
            Interactable::Span(_, s) => Cow::Borrowed(&s.content)
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
                        let all_spans_have_action = line.spans.iter().all(|span| span.action.is_some());

                        if !all_spans_have_action {
                            bucket.push(Interactable::Choice(&key, &choices, *i));
                        }

                        for span in &line.spans {
                            if span.action.is_some() {
                                bucket.push(Interactable::Span(obj, span));
                            }
                        }
                    }
                }

                Object::Image(_) | Object::Break | Object::Empty(_) => {
                    // no interactables
                }
            }

            out.push(bucket);
        }

        out
    }

    /// Flatten all nested interactables into one linear Vec
    /// Filters out None Actions
    pub fn interactables_flat(&self) -> Vec<Interactable<'_>> {
        self.interactables()
        .into_iter()
        .flat_map(|v| {
            v.into_iter().filter(|e| {
                if let Interactable::Span(_, span) = e
                && matches!(span.action, Some(Action::None))
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
    pub fn interact(&mut self, e: Interactable<'_>, pageid: &PageId) -> Result<(), GameError> {
        match e {
            Interactable::Choice(key, _, index) => {
                self.handle_choice((pageid.clone(), *key), index);
                Ok(())
            }
            Interactable::Span(_, s) => {
                let action = s.action.as_ref().unwrap();
                self.handle_action(action.clone())
            }
        }
    }

    pub fn interact_all<F>(&self, view: View) -> Vec<Result<Self, GameError>> {
        view.interactables_flat()
        .into_iter()
        .map(|e| {
            let mut g = self.clone();
            g.interact(e, &view.pageid).map(|_| g)
        })
        .collect()
    }
}
