use bitflags::bitflags;
use std::collections::HashMap;

use crate::{
    core::{Action, game_state::InternalKey},
    utils::linguate,
}; // Added InternalKey

/// Abstract span, guided by HTML element/egui TextFormat
#[derive(Debug, Clone, Default)]
pub struct Span {
    pub action: Option<Action>,
    pub content: String,

    /// Applies a style preset, because links should be a logical concept
    pub variant: SpanVariant,

    // styling
    pub modifiers: Modifier,
    // is it worth collapsing to single Vec<(String, String)> for efficiency?
    pub style: HashMap<String, String>,
    pub classes: Vec<String>,
}

/// Applies a style preset
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
            action: None,
            content: s,
            variant: SpanVariant::None,
            modifiers: Modifier::empty(),
            style: HashMap::new(),
            classes: Vec::new(),
        }
    }

    pub fn from_lingual(v: impl Into<Self>) -> Self {
        v.into().lingual()
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

    /// Used by element macros on likely cyclic actions to prune choices when running simulations
    /// (todo) make it possible to append .hide_if(false) to ensure it never gets hidden
    pub fn hide_if(self, hide: bool) -> Self {
        if hide {
            Self::default()
        } else {
            self
        }
    }
}

/// A collection of [`Span`]'s, rendered in a wrapped line, joined without spacing
#[derive(Debug, Clone)]
pub struct Line {
    pub spans: Vec<Span>,
}

impl Line {
    pub fn new() -> Self {
        Self { spans: Vec::new() }
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
            spans: iter.into_iter().map(|x| x.into()).collect(),
        }
    }

    pub fn from_spans(spans: Vec<Span>) -> Self {
        Self { spans }
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

        Line { spans }
    }
}

bitflags! {
    // #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
impl From<String> for Span {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for Span {
    fn from(s: &str) -> Self {
        Self::from(s.to_owned())
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
