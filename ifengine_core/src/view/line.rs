use bitflags::bitflags;
use std::collections::HashMap;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::{Action, Actions, PageKey, utils::prose};

/// Abstract span. Similar in principle to an HTML element/egui TextFormat.
#[derive(Debug, Clone, Default)]
pub struct Span {
    pub id: Option<PageKey>,
    pub action: Actions,
    pub content: String,
    pub prose: bool,

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
        Self {
            id: None,
            action: Actions::new(),
            content: s.into(),
            prose: true,
            variant: SpanVariant::None,
            modifiers: Modifier::empty(),
            style: HashMap::new(),
            classes: Vec::new(),
            no_sim: false,
        }
    }

    /// Disables prose processing on this span.
    pub fn without_prose(mut self) -> Self {
        self.prose = false;
        self
    }

    /// Returns the text content, applying prose typesetting if enabled.
    pub fn content(&self) -> std::borrow::Cow<'_, str> {
        if self.prose {
            std::borrow::Cow::Owned(prose(&self.content))
        } else {
            std::borrow::Cow::Borrowed(&self.content)
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
        self.action.push(action.into());
        self
    }

    pub fn with_actions(mut self, actions: impl IntoIterator<Item = Action>) -> Self {
        self.action.extend(actions);
        self
    }

    pub fn reset_actions(mut self) -> Self {
        self.action.clear();
        self
    }

    /// Whether this span has any attached actions.
    pub fn active(&self) -> bool {
        !self.action.is_empty()
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

    /// Computes a 64-bit FNV-1a hash of the span's text content.
    pub fn content_hash(&self) -> u64 {
        const_fnv1a_hash::fnv1a_hash_str_64(&self.content)
    }
}

impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.content.hash(state);
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

    /// Ensures the line begins and ends with spans containing `id` by inserting
    /// empty delimiter spans if needed, framing the group boundary.
    pub fn delimit_group(mut self, id: PageKey) -> Self {
        if self.spans.first().and_then(|s| s.id) != Some(id) {
            self.spans.insert(0, Span::new("").with_id(id));
        }
        if self.spans.last().and_then(|s| s.id) != Some(id) {
            self.spans.push(Span::new("").with_id(id));
        }
        self
    }

    pub fn content(&self) -> String {
        let mut s = String::new();
        for span in &self.spans {
            s.push_str(&span.content())
        }
        s
    }

    /// Subtracts a string suffix from the line, truncating or popping trailing spans.
    pub fn sub_str(&mut self, rhs: &str) {
        if rhs.is_empty() {
            return;
        }
        let total_raw: String = self.spans.iter().map(|s| s.content.as_str()).collect();
        if !total_raw.ends_with(rhs) {
            return;
        }
        let mut remaining = rhs.len();
        while remaining > 0 && !self.spans.is_empty() {
            let last_len = self.spans.last().unwrap().content.len();
            if last_len <= remaining {
                remaining -= last_len;
                self.spans.pop();
            } else {
                let new_len = last_len - remaining;
                self.spans.last_mut().unwrap().content.truncate(new_len);
                remaining = 0;
            }
        }
    }

    /// Scans spans without actions, splits bracketed `[target]` links (with `[[` as a literal `[`), and attaches actions via `action_fn`.
    ///
    /// - If `FALLBACK` is `true` and no bracketed targets are found across the entire line,
    ///   all un-actioned spans are assigned actions via `action_fn`.
    /// - If `FALLBACK` is `false` and no bracketed targets are found, spans without actions are left as plain text.
    pub fn interleave_actions<const FALLBACK: bool>(
        mut self,
        mut action_fn: impl FnMut(usize, &Span) -> Actions,
    ) -> Self {
        let mut link_idx = 0usize;
        let mut new_spans = Vec::with_capacity(self.spans.len());

        for span in self.spans {
            if span.active() || span.content.is_empty() {
                new_spans.push(span);
            } else {
                let parts = crate::utils::split_braced(&span.content);
                for (i, part) in parts.into_iter().enumerate() {
                    if part.is_empty() {
                        continue;
                    }
                    let is_link = i % 2 == 1;
                    let mut s = span.clone().with_text(part);
                    if is_link {
                        let actions = action_fn(link_idx, &s);
                        for act in actions {
                            s.action.push(act);
                        }
                        link_idx += 1;
                    }
                    new_spans.push(s);
                }
            }
        }

        if FALLBACK && link_idx == 0 {
            for span in &mut new_spans {
                if !span.active() {
                    let actions = action_fn(link_idx, span);
                    for act in actions {
                        span.action.push(act);
                    }
                    link_idx += 1;
                }
            }
        }

        self.spans = new_spans;
        self
    }

    /// Clears actions on inner spans that satisfy the given predicate.
    pub fn clear_actions(&mut self, mut predicate: impl FnMut(&Span) -> bool) -> &mut Self {
        for span in &mut self.spans {
            if predicate(span) {
                span.action.clear();
            }
        }
        self
    }
}

impl std::hash::Hash for Line {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for span in &self.spans {
            match span.id {
                Some(id) => {
                    true.hash(state);
                    id.hash(state);
                }
                None => {
                    false.hash(state);
                    span.hash(state);
                }
            }
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

// ------------ Span Conversions ------------

impl From<&str> for Span {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Span {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&String> for Span {
    fn from(s: &String) -> Self {
        Self::new(s.as_str())
    }
}

impl From<std::borrow::Cow<'_, str>> for Span {
    fn from(s: std::borrow::Cow<'_, str>) -> Self {
        Self::new(s.as_ref())
    }
}

impl From<&Span> for Span {
    fn from(s: &Span) -> Self {
        s.clone()
    }
}

// ------------ Line Conversions ------------

impl<T: Into<Span>> From<T> for Line {
    fn from(item: T) -> Self {
        Line {
            spans: vec![item.into()],
            classes: Vec::new(),
        }
    }
}

impl From<&Line> for Line {
    fn from(item: &Line) -> Self {
        item.clone()
    }
}

impl From<()> for Line {
    fn from(_: ()) -> Self {
        Line::new()
    }
}

impl From<Vec<&str>> for Line {
    fn from(items: Vec<&str>) -> Self {
        Line::from_iter(items)
    }
}

impl From<Vec<String>> for Line {
    fn from(items: Vec<String>) -> Self {
        Line::from_iter(items)
    }
}

impl From<&[&str]> for Line {
    fn from(items: &[&str]) -> Self {
        Line::from_iter(items.iter().copied())
    }
}

impl From<&[String]> for Line {
    fn from(items: &[String]) -> Self {
        Line::from_iter(items.iter().cloned())
    }
}

impl<const N: usize> From<[&str; N]> for Line {
    fn from(items: [&str; N]) -> Self {
        Line::from_iter(items)
    }
}

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

impl<T: Into<Line>> FromIterator<T> for Line {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut out = Line::new();
        for item in iter {
            let mut line = item.into();
            if !line.classes.is_empty() {
                for span in &mut line.spans {
                    span.classes.extend(line.classes.clone());
                }
            }
            out.spans.append(&mut line.spans);
        }
        out
    }
}

impl<T: Into<Line>> Add<T> for Line {
    type Output = Self;
    fn add(mut self, rhs: T) -> Self {
        let mut line = rhs.into();
        if !line.classes.is_empty() {
            for span in &mut line.spans {
                span.classes.extend(line.classes.clone());
            }
        }
        self.spans.append(&mut line.spans);
        self
    }
}

impl<T: Into<Line>> Add<T> for Span {
    type Output = Line;
    fn add(self, rhs: T) -> Line {
        Line::from(self) + rhs
    }
}

impl<T: Into<Line>> Add<T> for &Span {
    type Output = Line;
    fn add(self, rhs: T) -> Line {
        Line::from(self) + rhs
    }
}

impl<T: Into<Line>> AddAssign<T> for Line {
    fn add_assign(&mut self, rhs: T) {
        let mut line = rhs.into();
        if !line.classes.is_empty() {
            for span in &mut line.spans {
                span.classes.extend(line.classes.clone());
            }
        }
        self.spans.append(&mut line.spans);
    }
}

impl<T: Into<Line>> Extend<T> for Line {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            let mut line = item.into();
            if !line.classes.is_empty() {
                for span in &mut line.spans {
                    span.classes.extend(line.classes.clone());
                }
            }
            self.spans.append(&mut line.spans);
        }
    }
}

impl Sub<&str> for Line {
    type Output = Self;
    fn sub(mut self, rhs: &str) -> Self {
        self.sub_str(rhs);
        self
    }
}

impl Sub<String> for Line {
    type Output = Self;
    fn sub(mut self, rhs: String) -> Self {
        self.sub_str(&rhs);
        self
    }
}

impl Sub<&String> for Line {
    type Output = Self;
    fn sub(mut self, rhs: &String) -> Self {
        self.sub_str(rhs.as_str());
        self
    }
}

impl Sub<char> for Line {
    type Output = Self;
    fn sub(mut self, rhs: char) -> Self {
        let mut buf = [0u8; 4];
        self.sub_str(rhs.encode_utf8(&mut buf));
        self
    }
}

impl Sub<Line> for Line {
    type Output = Self;
    fn sub(mut self, rhs: Line) -> Self {
        let rhs_raw: String = rhs.spans.iter().map(|s| s.content.as_str()).collect();
        self.sub_str(&rhs_raw);
        self
    }
}

impl Sub<&Line> for Line {
    type Output = Self;
    fn sub(mut self, rhs: &Line) -> Self {
        let rhs_raw: String = rhs.spans.iter().map(|s| s.content.as_str()).collect();
        self.sub_str(&rhs_raw);
        self
    }
}

impl Sub<Span> for Line {
    type Output = Self;
    fn sub(mut self, rhs: Span) -> Self {
        self.sub_str(&rhs.content);
        self
    }
}

impl Sub<&Span> for Line {
    type Output = Self;
    fn sub(mut self, rhs: &Span) -> Self {
        self.sub_str(&rhs.content);
        self
    }
}

impl SubAssign<&str> for Line {
    fn sub_assign(&mut self, rhs: &str) {
        self.sub_str(rhs);
    }
}

impl SubAssign<String> for Line {
    fn sub_assign(&mut self, rhs: String) {
        self.sub_str(&rhs);
    }
}

impl SubAssign<&String> for Line {
    fn sub_assign(&mut self, rhs: &String) {
        self.sub_str(rhs.as_str());
    }
}

impl SubAssign<char> for Line {
    fn sub_assign(&mut self, rhs: char) {
        let mut buf = [0u8; 4];
        self.sub_str(rhs.encode_utf8(&mut buf));
    }
}

impl SubAssign<Line> for Line {
    fn sub_assign(&mut self, rhs: Line) {
        let rhs_raw: String = rhs.spans.iter().map(|s| s.content.as_str()).collect();
        self.sub_str(&rhs_raw);
    }
}

impl SubAssign<&Line> for Line {
    fn sub_assign(&mut self, rhs: &Line) {
        let rhs_raw: String = rhs.spans.iter().map(|s| s.content.as_str()).collect();
        self.sub_str(&rhs_raw);
    }
}

impl SubAssign<Span> for Line {
    fn sub_assign(&mut self, rhs: Span) {
        self.sub_str(&rhs.content);
    }
}

impl SubAssign<&Span> for Line {
    fn sub_assign(&mut self, rhs: &Span) {
        self.sub_str(&rhs.content);
    }
}

impl Sub<&str> for Span {
    type Output = Line;
    fn sub(self, rhs: &str) -> Line {
        Line::from(self) - rhs
    }
}

impl Sub<String> for Span {
    type Output = Line;
    fn sub(self, rhs: String) -> Line {
        Line::from(self) - rhs
    }
}

impl Sub<&String> for Span {
    type Output = Line;
    fn sub(self, rhs: &String) -> Line {
        Line::from(self) - rhs
    }
}

impl Sub<char> for Span {
    type Output = Line;
    fn sub(self, rhs: char) -> Line {
        Line::from(self) - rhs
    }
}

impl Sub<&str> for &Span {
    type Output = Line;
    fn sub(self, rhs: &str) -> Line {
        Line::from(self.clone()) - rhs
    }
}

impl Sub<&str> for &Line {
    type Output = Line;
    fn sub(self, rhs: &str) -> Line {
        self.clone() - rhs
    }
}

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

        let hash_line = |l: &Line| -> u64 {
            use std::hash::{DefaultHasher, Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            l.hash(&mut hasher);
            hasher.finish()
        };

        let l1 = Line::from_iter(["hello", " ", "world"]);
        let l2 = Line::from_iter(["hello", " ", "world"]);
        let l3 = Line::from_iter(["different"]);

        assert_ne!(hash_line(&l1), 0);
        assert_eq!(hash_line(&l1), hash_line(&l2));
        assert_ne!(hash_line(&l1), hash_line(&l3));

        // Keyed spans: content changes within a keyed span do NOT change Line hash
        let mut l_keyed1 = Line::new();
        l_keyed1.push(Span::new("prefix: "));
        l_keyed1.push(Span::new("clicks: 0").with_id(42));

        let mut l_keyed2 = Line::new();
        l_keyed2.push(Span::new("prefix: "));
        l_keyed2.push(Span::new("clicks: 1").with_id(42));

        assert_eq!(
            hash_line(&l_keyed1),
            hash_line(&l_keyed2),
            "changing dynamic content of a keyed span must not change line content hash"
        );

        // But changing the keyed span's ID DOES change Line hash
        let mut l_keyed3 = Line::new();
        l_keyed3.push(Span::new("prefix: "));
        l_keyed3.push(Span::new("clicks: 0").with_id(99));

        assert_ne!(
            hash_line(&l_keyed1),
            hash_line(&l_keyed3),
            "changing span id must change line content hash"
        );

        // And changing the unkeyed span's content DOES change Line hash
        let mut l_keyed4 = Line::new();
        l_keyed4.push(Span::new("different prefix: "));
        l_keyed4.push(Span::new("clicks: 0").with_id(42));

        assert_ne!(
            hash_line(&l_keyed1),
            hash_line(&l_keyed4),
            "changing unkeyed span content must change line content hash"
        );
    }

    #[test]
    fn test_line_interleave_actions() {
        // Without fallback: bracketed text becomes links, plain text is untouched
        let line = Line::from("Go to [forest] or [inn]")
            .interleave_actions::<false>(|i, _| Actions::single(Action::SetBit(1, i as u8)));
        assert_eq!(line.spans.len(), 4);
        assert_eq!(line.spans[0].content, "Go to ");
        assert!(!line.spans[0].active());
        assert_eq!(line.spans[1].content, "forest");
        assert_eq!(line.spans[1].action.as_slice(), [Action::SetBit(1, 0)]);
        assert_eq!(line.spans[2].content, " or ");
        assert!(!line.spans[2].active());
        assert_eq!(line.spans[3].content, "inn");
        assert_eq!(line.spans[3].action.as_slice(), [Action::SetBit(1, 1)]);

        // Without fallback and no brackets: no links
        let line_nobrackets = Line::from("No links here")
            .interleave_actions::<false>(|i, _| Actions::single(Action::SetBit(1, i as u8)));
        assert_eq!(line_nobrackets.spans.len(), 1);
        assert!(!line_nobrackets.spans[0].active());

        // With fallback and no brackets: entire line becomes clickable link
        let line_fallback = Line::from("The old chest is locked.")
            .interleave_actions::<true>(|i, _| Actions::single(Action::SetBit(1, i as u8)));
        assert_eq!(line_fallback.spans.len(), 1);
        assert_eq!(line_fallback.spans[0].content, "The old chest is locked.");
        assert_eq!(
            line_fallback.spans[0].action.as_slice(),
            [Action::SetBit(1, 0)]
        );

        // Preserves prototype classes and styles
        let proto_line = Line::from(Span::new("See [item] now").cls("highlight"))
            .interleave_actions::<false>(|i, _| Actions::single(Action::SetBit(1, i as u8)));
        assert_eq!(proto_line.spans.len(), 3);
        assert_eq!(proto_line.spans[0].classes, vec!["highlight"]);
        assert_eq!(proto_line.spans[1].classes, vec!["highlight"]);
        assert_eq!(
            proto_line.spans[1].action.as_slice(),
            [Action::SetBit(1, 0)]
        );
        assert_eq!(proto_line.spans[2].classes, vec!["highlight"]);
    }

    #[test]
    fn test_span_multiple_actions_and_reset() {
        let span = Span::new("test")
            .with_action(Action::SetBit(1, 0))
            .with_action(Action::Exit);
        assert_eq!(span.action.len(), 2);
        assert_eq!(span.action.as_slice(), [Action::SetBit(1, 0), Action::Exit]);
        assert!(span.active());

        let span_cleared = span.reset_actions();
        assert!(!span_cleared.active());
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
        assert_eq!(span_prose.content(), "“Hello… world—test”");
        assert_eq!(span_prose.content, "''Hello... world--test''");
        assert_eq!(
            span_prose.without_prose().content(),
            "''Hello... world--test''"
        );

        let line_prose = Line::from("''Hello... world--test''");
        assert_eq!(line_prose.content(), "“Hello… world—test”");
    }

    #[test]
    fn test_from_iterator_and_extend() {
        // Collect from &str
        let l1: Line = vec!["Hello", ", ", "world!"].into_iter().collect();
        assert_eq!(l1.content(), "Hello, world!");
        assert_eq!(l1.spans.len(), 3);

        // Collect from String
        let l2: Line = vec!["A".to_string(), "B".to_string()].into_iter().collect();
        assert_eq!(l2.content(), "AB");

        // Collect from Span
        let spans = vec![Span::new("Span1"), Span::new("Span2")];
        let l3: Line = spans.iter().collect();
        assert_eq!(l3.content(), "Span1Span2");

        // Collect from Line and &Line
        let lines = vec![Line::from("Line1 "), Line::from("Line2")];
        let l4: Line = lines.iter().collect();
        assert_eq!(l4.content(), "Line1 Line2");

        // Extend with &Span, &Line, and &str
        let mut base = Line::from("Base: ");
        base.extend(&spans);
        assert_eq!(base.content(), "Base: Span1Span2");
        base.extend(&lines);
        assert_eq!(base.content(), "Base: Span1Span2Line1 Line2");
        base.extend([" - extra", " items"]);
        assert_eq!(base.content(), "Base: Span1Span2Line1 Line2 - extra items");
    }

    #[test]
    fn test_line_sub_and_sub_assign() {
        let mut line = Line::from(Span::new("Hello ")) + Span::new("world!");
        assert_eq!(line.spans.len(), 2);

        // Sub string truncating within last span
        let l2 = line.clone() - "!";
        assert_eq!(l2.spans.len(), 2);
        assert_eq!(l2.spans[1].content, "world");

        // Sub string popping entire last span
        let l3 = line.clone() - "world!";
        assert_eq!(l3.spans.len(), 1);
        assert_eq!(l3.spans[0].content, "Hello ");

        // Sub string spanning multiple spans
        let l4 = line.clone() - "o world!";
        assert_eq!(l4.spans.len(), 1);
        assert_eq!(l4.spans[0].content, "Hell");

        // Sub full line content
        let l5 = line.clone() - "Hello world!";
        assert_eq!(l5.spans.len(), 0);

        // Non-matching suffix does nothing
        let l6 = line.clone() - "foo";
        assert_eq!(l6.spans.len(), 2);
        assert_eq!(l6.content(), "Hello world!");

        // SubAssign
        line -= "!";
        assert_eq!(line.content(), "Hello world");
        line -= " world";
        assert_eq!(line.content(), "Hello");
        assert_eq!(line.spans.len(), 1);

        // Span - &str
        let span_sub = Span::new("testing") - "ing";
        assert_eq!(span_sub.content(), "test");
    }
}
