use bitflags::bitflags;
use std::collections::HashMap;

use crate::{
    core::{
        Action,
        game_state::{InternalKey, PageKey},
    },
    utils::linguate,
};

/// Abstract span. Similar in principle to an HTML element/egui TextFormat.
#[derive(Debug, Clone, Default)]
pub struct Span {
    pub id: Option<PageKey>,
    pub action: Option<Action>,
    pub content: String,

    /// Applies a style preset, because links should be a logical concept
    pub variant: SpanVariant,

    // styling
    pub modifiers: Modifier,
    // is it worth collapsing to single Vec<(String, String)> for efficiency?
    pub style: HashMap<String, String>,
    pub classes: Vec<String>,
    pub no_sim: bool,
}

/// Applies a style preset.
#[derive(Debug, Default, Clone)]
pub enum SpanVariant {
    #[default]
    None,
    /// Underline
    Link,
    /// Muted
    Muted,
    // The primary of the supported colors of style["color"]
    Secondary,
}

impl Span {
    pub fn new(s: String) -> Self {
        Self {
            id: None,
            action: None,
            content: s,
            variant: SpanVariant::None,
            modifiers: Modifier::empty(),
            style: HashMap::new(),
            classes: Vec::new(),
            no_sim: false,
        }
    }

    pub fn with_id(mut self, id: PageKey) -> Self {
        self.id = Some(id);
        self
    }

    pub fn cls(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn classes(mut self, classes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.classes.extend(classes.into_iter().map(Into::into));
        self
    }

    pub fn style(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.style.insert(key.into(), value.into());
        self
    }

    pub fn styles(
        mut self,
        styles: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (k, v) in styles {
            self.style.insert(k.into(), v.into());
        }
        self
    }

    pub fn from_lingual(v: impl Into<Self>) -> Self {
        v.into().lingual()
    }

    pub fn as_variant(mut self, variant: SpanVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn as_link(mut self) -> Self {
        self.variant = SpanVariant::Link;
        self
    }

    pub fn with_action(mut self, action: impl Into<Action>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn lingual(mut self) -> Self {
        self.content = linguate(&self.content);
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.content = text;
        self
    }

    /// Hides the span during simulation.
    ///
    /// Element macros with actions likely to be cyclic specify this by default to prune choices when running simulations.
    /// That can be overridden with no_sim().
    pub fn no_sim(mut self) -> Self {
        self.no_sim = true;
        self
    }

    /// Enforce that this span's action is not omitted during simulation.
    pub fn sim(mut self) -> Self {
        self.no_sim = false;
        self
    }
}

/// A collection of [`Span`]'s, rendered in a wrapped line, joined without spacing.
#[derive(Debug, Clone, Default)]
pub struct Line {
    pub id: Option<PageKey>,
    pub spans: Vec<Span>,
    pub classes: Vec<String>,
}

impl Line {
    pub fn new() -> Self {
        Self {
            id: None,
            spans: Vec::new(),
            classes: Vec::new(),
        }
    }

    pub fn with_id(mut self, id: PageKey) -> Self {
        self.id = Some(id);
        self
    }

    pub fn cls(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn classes(mut self, classes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.classes.extend(classes.into_iter().map(Into::into));
        self
    }

    pub fn content(&self) -> String {
        let mut s = String::new();
        for span in &self.spans {
            s.push_str(&span.content)
        }
        s
    }

    pub fn from_lingual(v: impl Into<Self>) -> Self {
        let mut ret = v.into();
        for s in ret.spans.iter_mut() {
            s.content = linguate(&s.content)
        }
        ret
    }

    pub fn from_iter<I: IntoIterator<Item = impl Into<Span>>>(iter: I) -> Self {
        Self {
            id: None,
            spans: iter.into_iter().map(|x| x.into()).collect(),
            classes: Vec::new(),
        }
    }

    pub fn from_spans(spans: Vec<Span>) -> Self {
        Self {
            id: None,
            spans,
            classes: Vec::new(),
        }
    }

    pub fn from_interleaved_actions<const MASK: bool>(
        key: InternalKey,
        parts: Vec<String>,
    ) -> Self {
        let mut spans = Vec::new();

        for (i, part) in parts.into_iter().enumerate() {
            // odd indices are braced, even indices are unbraced
            if i % 2 == 1 {
                if MASK {
                    spans.push(
                        Span::from_lingual(part)
                            .as_link()
                            .with_action(Action::SetBit(key.clone(), i as u8 / 2)),
                    );
                } else {
                    let h: u64;

                    #[cfg(feature = "rand")]
                    {
                        h = const_fnv1a_hash::fnv1a_hash_str_64(&part);
                    }
                    #[cfg(not(feature = "rand"))]
                    {
                        h = (i / 2) as u64;
                    }

                    spans.push(
                        Span::from_lingual(part)
                            .as_link()
                            .with_action(Action::Set(key.clone(), h)),
                    );
                }
            } else {
                spans.push(Span::from_lingual(part));
            }
        }

        Line {
            id: None,
            spans,
            classes: Vec::new(),
        }
    }
}

bitflags! {
    // #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]

    /// Applies a set of styles to [`Span`].
    ///
    /// The effect of these styles (if any) depends on the frontend implementation
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
    pub struct Modifier: u16 {
        const BOLD              = 0b0000_0000_0001;
        const DIM               = 0b0000_0000_0010;
        const ITALIC            = 0b0000_0000_0100;
        const UNDERLINE        = 0b0000_0000_1000;
        const SUPER_SCRIPT        = 0b0000_0001_0000;
        const SUBSCRIPT       = 0b0000_0010_0000;
        const REVERSED          = 0b0000_0100_0000;
        const HIDDEN            = 0b0000_1000_0000;
        const STRIKETHROUGH       = 0b0001_0000_0000;
    }
}

// ------------ BOILERPLATE ----------------

impl<'a> IntoIterator for &'a Line {
    type Item = &'a Span;
    type IntoIter = std::slice::Iter<'a, Span>;

    fn into_iter(self) -> Self::IntoIter {
        self.spans.iter()
    }
}

// Into<Span>
// impl From<String> for Span {
//     fn from(s: String) -> Self {
//         Self::new(s)
//     }
// }

// impl From<&str> for Span {
//     fn from(s: &str) -> Self {
//         Self::from(s.to_owned())
//     }
// }

impl<T: ToString> From<T> for Span {
    fn from(s: T) -> Self {
        Self::new(s.to_string())
    }
}

// Line: From Into<Span>
impl From<&str> for Line {
    fn from(item: &str) -> Self {
        Line::from_iter(std::iter::once(item))
    }
}

impl From<String> for Line {
    fn from(item: String) -> Self {
        Line::from_iter(std::iter::once(item))
    }
}

impl From<Span> for Line {
    fn from(item: Span) -> Self {
        Line::from_iter(std::iter::once(item))
    }
}

// note: as part of ! becoming a type, !; doesn't evaluate to ()
// impl From<!> for Line {
//     fn from(item: !) -> Self {
//         unreachable!()
//     }
// }

// From Vec of &str
impl From<Vec<&str>> for Line {
    fn from(items: Vec<&str>) -> Self {
        Line::from_iter(items)
    }
}

// From Vec of String
impl From<Vec<String>> for Line {
    fn from(items: Vec<String>) -> Self {
        Line::from_iter(items)
    }
}

// From slice of &str
impl From<&[&str]> for Line {
    fn from(items: &[&str]) -> Self {
        Line::from_iter(items.iter().copied())
    }
}

// From slice of String
impl From<&[String]> for Line {
    fn from(items: &[String]) -> Self {
        Line::from_iter(items.iter().cloned())
    }
}

// From array of &str
impl<const N: usize> From<[&str; N]> for Line {
    fn from(items: [&str; N]) -> Self {
        Line::from_iter(items)
    }
}

// From array of String
impl<const N: usize> From<[String; N]> for Line {
    fn from(items: [String; N]) -> Self {
        Line::from_iter(items)
    }
}

impl<const N: usize> From<[Span; N]> for Line {
    fn from(items: [Span; N]) -> Self {
        Line::from_iter(items)
    }
}

// this is too broad
// impl<U, S: Into<Span>> From<U> for Line
// where
//     U: IntoIterator<Item = S>,
// {
//     fn from(items: U) -> Self {
//         Line::from_iter(items)
//     }
// }
//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_builders() {
        let span = Span::new("hello".into())
            .with_id(42)
            .cls("fade")
            .classes(["delay-100", "bold"])
            .style("color", "red")
            .styles([("font-size", "14px"), ("opacity", "0.8")]);

        assert_eq!(span.id, Some(42));
        assert_eq!(span.classes, vec!["fade", "delay-100", "bold"]);
        assert_eq!(span.style.get("color").map(|s| s.as_str()), Some("red"));
        assert_eq!(
            span.style.get("font-size").map(|s| s.as_str()),
            Some("14px")
        );
        assert_eq!(span.style.get("opacity").map(|s| s.as_str()), Some("0.8"));
    }

    #[test]
    fn test_line_builders() {
        let line = Line::new()
            .with_id(100)
            .cls("my-line")
            .classes(["line-fade"]);

        assert_eq!(line.id, Some(100));
        assert_eq!(line.classes, vec!["my-line", "line-fade"]);
    }
}
