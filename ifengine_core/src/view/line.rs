use bitflags::bitflags;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};

use crate::{
    core::{Action, game_state::PageKey},
    utils::prose,
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
    pub fn new(s: impl Into<String>) -> Self {
        Self::from(s.into())
    }

    pub fn raw(s: impl Into<String>) -> Self {
        Self {
            id: None,
            action: None,
            content: s.into(),
            variant: SpanVariant::None,
            modifiers: Modifier::empty(),
            style: HashMap::new(),
            classes: Vec::new(),
            no_sim: false,
        }
    }

    pub fn with_id(mut self, id: PageKey) -> Self {
        if self.id.is_none() {
            self.id = Some(id);
        }
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

    pub fn as_variant(mut self, variant: SpanVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Marks this span as a link ([`SpanVariant::Link`]), applying link styling and semantics.
    pub fn as_link(mut self) -> Self {
        self.variant = SpanVariant::Link;
        self
    }

    pub fn with_action(mut self, action: impl Into<Action>) -> Self {
        self.action = Some(action.into());
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

    /// Strips all classes from the span.
    pub fn clean(mut self) -> Self {
        self.classes.clear();
        self
    }

    /// Computes a non-zero 64-bit hash of the span's text content.
    ///
    /// Always returns a non-zero value.
    pub fn content_hash(&self) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        match hasher.finish() {
            0 => 1,
            h => h,
        }
    }
}

/// A collection of [`Span`]'s, rendered in a wrapped line, joined without spacing.
#[derive(Debug, Clone, Default)]
pub struct Line {
    pub spans: Vec<Span>,
    pub classes: Vec<String>,
}

impl Line {
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            classes: Vec::new(),
        }
    }

    pub fn raw(s: impl Into<String>) -> Self {
        Self::from_spans(vec![Span::raw(s)])
    }

    pub fn cls(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn classes(mut self, classes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.classes.extend(classes.into_iter().map(Into::into));
        self
    }

    /// Strips all classes from the line and all its inner spans.
    pub fn clean(mut self) -> Self {
        self.classes.clear();
        for span in &mut self.spans {
            span.classes.clear();
        }
        self
    }

    pub fn push(&mut self, span: impl Into<Span>) {
        self.spans.push(span.into());
    }

    pub fn content(&self) -> String {
        let mut s = String::new();
        for span in &self.spans {
            s.push_str(&span.content)
        }
        s
    }

    /// Computes a non-zero 64-bit hash of the line's content.
    ///
    /// For spans with an explicit [`id`](Span::id) (e.g. `count!`, `alts!`), only the span's ID
    /// is hashed rather than its dynamic content. This allows child spans to manage their own
    /// transitions independently without causing the enclosing line or paragraph to fade.
    /// Spans without an ID have their text content hashed directly.
    ///
    /// Always returns a non-zero value.
    pub fn content_hash(&self) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for span in &self.spans {
            match span.id {
                Some(id) => {
                    true.hash(&mut hasher);
                    id.hash(&mut hasher);
                }
                None => {
                    false.hash(&mut hasher);
                    span.content.hash(&mut hasher);
                }
            }
        }
        match hasher.finish() {
            0 => 1,
            h => h,
        }
    }

    pub fn from_iter<I: IntoIterator<Item = impl Into<Span>>>(iter: I) -> Self {
        Self {
            spans: iter.into_iter().map(|x| x.into()).collect(),
            classes: Vec::new(),
        }
    }

    pub fn from_spans(spans: Vec<Span>) -> Self {
        Self {
            spans,
            classes: Vec::new(),
        }
    }

    /// Interleaves unbraced text and interactive braced link spans into a [`Line`].
    pub fn from_interleaved_actions(
        parts: Vec<String>,
        action_fn: impl FnMut(usize, &Span) -> Option<Action>,
    ) -> Self {
        Self::from_spans(interleave_actions(&Span::raw(""), parts, action_fn))
    }
}

/// Interleaves unbraced text and interactive braced link spans into a sequence of [`Span`]s.
///
/// Takes a `base_span` (whose styling, classes, and metadata are cloned into each generated span),
/// a `parts` vector of interleaved strings (typically produced by [`split_braced`]), and an
/// `action_fn` closure `|i, span| -> Option<Action>` called for each non-empty braced link target.
///
/// - Even indices (0, 2, ...) represent regular, non-clickable text segments.
/// - Odd indices (1, 3, ...) represent link targets delimited by `[[...]]`.
///
/// Empty segments are skipped.
pub fn interleave_actions(
    base_span: &Span,
    parts: Vec<String>,
    mut action_fn: impl FnMut(usize, &Span) -> Option<Action>,
) -> Vec<Span> {
    let mut link_idx = 0usize;
    parts
        .into_iter()
        .enumerate()
        .filter_map(|(i, part)| {
            if part.is_empty() {
                return None;
            }

            let mut span = base_span.clone().with_text(part);
            if i % 2 == 1 {
                span.action = action_fn(link_idx, &span);
                link_idx += 1;
            }
            Some(span)
        })
        .collect()
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
        Self::raw(prose(&s.to_string()))
    }
}

impl From<Line> for Span {
    fn from(mut line: Line) -> Self {
        if line.spans.len() == 1 {
            let mut span = line.spans.remove(0);
            span.classes.extend(line.classes);
            span
        } else {
            Span {
                id: None,
                action: None,
                content: line.content(),
                variant: SpanVariant::None,
                modifiers: Modifier::empty(),
                style: HashMap::new(),
                classes: line.classes,
                no_sim: false,
            }
        }
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

impl From<()> for Line {
    fn from(_: ()) -> Self {
        Line::new()
    }
}

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

impl Add<&str> for Line {
    type Output = Self;
    fn add(mut self, rhs: &str) -> Self {
        self.push(Span::from(rhs));
        self
    }
}

impl Add<String> for Line {
    type Output = Self;
    fn add(mut self, rhs: String) -> Self {
        self.push(Span::from(rhs));
        self
    }
}

impl Add<&String> for Line {
    type Output = Self;
    fn add(self, rhs: &String) -> Self {
        self + rhs.as_str()
    }
}

impl Add<Span> for Line {
    type Output = Self;
    fn add(mut self, rhs: Span) -> Self {
        self.push(rhs);
        self
    }
}

impl Add<&Span> for Line {
    type Output = Self;
    fn add(mut self, rhs: &Span) -> Self {
        self.push(rhs.clone());
        self
    }
}

impl Add<Line> for Line {
    type Output = Self;
    fn add(mut self, mut rhs: Line) -> Self {
        self.spans.append(&mut rhs.spans);
        self.classes.append(&mut rhs.classes);
        self
    }
}

impl Add<&Line> for Line {
    type Output = Self;
    fn add(mut self, rhs: &Line) -> Self {
        self.spans.extend(rhs.spans.iter().cloned());
        self.classes.extend(rhs.classes.iter().cloned());
        self
    }
}

impl AddAssign<&str> for Line {
    fn add_assign(&mut self, rhs: &str) {
        self.push(Span::from(rhs));
    }
}

impl AddAssign<String> for Line {
    fn add_assign(&mut self, rhs: String) {
        self.push(Span::from(rhs));
    }
}

impl AddAssign<&String> for Line {
    fn add_assign(&mut self, rhs: &String) {
        *self += rhs.as_str();
    }
}

impl AddAssign<Span> for Line {
    fn add_assign(&mut self, rhs: Span) {
        self.push(rhs);
    }
}

impl AddAssign<&Span> for Line {
    fn add_assign(&mut self, rhs: &Span) {
        self.push(rhs.clone());
    }
}

impl AddAssign<Line> for Line {
    fn add_assign(&mut self, mut rhs: Line) {
        self.spans.append(&mut rhs.spans);
        self.classes.append(&mut rhs.classes);
    }
}

impl AddAssign<&Line> for Line {
    fn add_assign(&mut self, rhs: &Line) {
        self.spans.extend(rhs.spans.iter().cloned());
        self.classes.extend(rhs.classes.iter().cloned());
    }
}

impl Extend<Span> for Line {
    fn extend<T: IntoIterator<Item = Span>>(&mut self, iter: T) {
        self.spans.extend(iter);
    }
}

impl Extend<Line> for Line {
    fn extend<T: IntoIterator<Item = Line>>(&mut self, iter: T) {
        for mut line in iter {
            self.spans.append(&mut line.spans);
            self.classes.append(&mut line.classes);
        }
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

// ----------------

/// Trait to normalize varied input formats into `(Option<u8>, Line)`.
pub trait IntoNumberedLine {
    fn into_numbered_line(self) -> (Option<u8>, Line);
}

// 1. Accepts anything that can convert into `Line` (e.g. Line, String, &str)
impl<T> IntoNumberedLine for T
where
    T: Into<Line>,
{
    fn into_numbered_line(self) -> (Option<u8>, Line) {
        (None, self.into())
    }
}

// 2. Accepts `(u8, T)` where T converts into `Line`
impl<T> IntoNumberedLine for (u8, T)
where
    T: Into<Line>,
{
    fn into_numbered_line(self) -> (Option<u8>, Line) {
        (Some(self.0), self.1.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_builders() {
        let span = Span::new("hello")
            .with_id(42)
            .cls("in-100")
            .classes(["out-200", "bold"])
            .style("color", "red")
            .styles([("font-size", "14px"), ("opacity", "0.8")]);

        assert_eq!(span.id, Some(42));
        assert_eq!(span.classes, vec!["in-100", "out-200", "bold"]);
        assert_eq!(span.style.get("color").map(|s| s.as_str()), Some("red"));
        assert_eq!(
            span.style.get("font-size").map(|s| s.as_str()),
            Some("14px")
        );
        assert_eq!(span.style.get("opacity").map(|s| s.as_str()), Some("0.8"));
    }

    #[test]
    fn test_line_builders() {
        let line = Line::new().cls("my-line").classes(["line-fade"]);

        assert_eq!(line.classes, vec!["my-line", "line-fade"]);
    }

    #[test]
    fn test_content_hash() {
        let s1 = Span::new("hello");
        let s2 = Span::new("hello");
        let s3 = Span::new("world");

        assert_ne!(s1.content_hash(), 0);
        assert_eq!(s1.content_hash(), s2.content_hash());
        assert_ne!(s1.content_hash(), s3.content_hash());

        let l1 = Line::from_iter(["hello", " ", "world"]);
        let l2 = Line::from_iter(["hello", " ", "world"]);
        let l3 = Line::from_iter(["different"]);

        assert_ne!(l1.content_hash(), 0);
        assert_eq!(l1.content_hash(), l2.content_hash());
        assert_ne!(l1.content_hash(), l3.content_hash());

        // Keyed spans: content changes within a keyed span do NOT change Line::content_hash()
        let mut l_keyed1 = Line::new();
        l_keyed1.push(Span::new("prefix: "));
        l_keyed1.push(Span::new("clicks: 0").with_id(42));

        let mut l_keyed2 = Line::new();
        l_keyed2.push(Span::new("prefix: "));
        l_keyed2.push(Span::new("clicks: 1").with_id(42));

        assert_eq!(
            l_keyed1.content_hash(),
            l_keyed2.content_hash(),
            "changing dynamic content of a keyed span must not change line content hash"
        );

        // But changing the keyed span's ID DOES change Line::content_hash()
        let mut l_keyed3 = Line::new();
        l_keyed3.push(Span::new("prefix: "));
        l_keyed3.push(Span::new("clicks: 0").with_id(99));

        assert_ne!(
            l_keyed1.content_hash(),
            l_keyed3.content_hash(),
            "changing span id must change line content hash"
        );

        // And changing the unkeyed span's content DOES change Line::content_hash()
        let mut l_keyed4 = Line::new();
        l_keyed4.push(Span::new("different prefix: "));
        l_keyed4.push(Span::new("clicks: 0").with_id(42));

        assert_ne!(
            l_keyed1.content_hash(),
            l_keyed4.content_hash(),
            "changing unkeyed span content must change line content hash"
        );
    }

    #[test]
    fn test_from_interleaved_actions_skips_empty_spans() {
        let parts = vec!["".to_string(), "link".to_string(), "".to_string()];
        let line = Line::from_interleaved_actions(parts, |i, _| {
            Some(Action::SetBit(1, i as u8))
        });
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "link");
        assert_eq!(line.spans[0].variant, SpanVariant::None);
        assert!(line.spans[0].action.is_some());
    }

    #[test]
    fn test_line_add_and_add_assign() {
        let l1 = Line::from("Hello");
        let l2 = l1 + ", " + "world!";
        assert_eq!(l2.content(), "Hello, world!");
        assert_eq!(l2.spans.len(), 3);

        let mut l3 = Line::from("Base");
        l3 += " string";
        l3 += String::from(" with String");
        l3 += Span::new(" and Span");
        let l4 = Line::from(" and Line");
        l3 += l4;
        assert_eq!(l3.content(), "Base string with String and Span and Line");

        let l5 = Line::from("Part 1") + Line::from(" Part 2");
        assert_eq!(l5.content(), "Part 1 Part 2");
    }

    #[test]
    fn test_line_and_span_clean() {
        let span = Span::new("Text").cls("red").cls("bold");
        assert_eq!(span.classes, vec!["red", "bold"]);
        let cleaned_span = span.clean();
        assert!(cleaned_span.classes.is_empty());

        let line = Line::from(Span::new("Inner").cls("italic")).cls("container");
        assert_eq!(line.classes, vec!["container"]);
        assert_eq!(line.spans[0].classes, vec!["italic"]);

        let cleaned_line = line.clean();
        assert!(cleaned_line.classes.is_empty());
        assert!(cleaned_line.spans[0].classes.is_empty());
        assert_eq!(cleaned_line.content(), "Inner");
    }

    #[test]
    fn test_span_and_line_prose_and_raw() {
        let span_prose = Span::from("''Hello... world--test''");
        assert_eq!(span_prose.content, "“Hello… world—test”");

        let span_raw = Span::raw("''Hello... world--test''");
        assert_eq!(span_raw.content, "''Hello... world--test''");

        let line_prose = Line::from("''Hello... world--test''");
        assert_eq!(line_prose.content(), "“Hello… world—test”");

        let line_raw = Line::raw("''Hello... world--test''");
        assert_eq!(line_raw.content(), "''Hello... world--test''");
    }
}
