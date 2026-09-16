use bitflags::bitflags;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};

use crate::{Action, PageKey, utils::prose};

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
        Self {
            id: None,
            action: None,
            content: prose(&s.into()),
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

    /// Scans spans without actions, splits bracketed `[[target]]` links, and attaches actions via `action_fn`.
    ///
    /// - If `FALLBACK` is `true` and no bracketed targets are found across the entire line,
    ///   all un-actioned spans are assigned actions via `action_fn`.
    /// - If `FALLBACK` is `false` and no bracketed targets are found, spans without actions are left as plain text.
    pub fn interleave_actions<const FALLBACK: bool>(
        mut self,
        mut action_fn: impl FnMut(usize, &Span) -> Option<Action>,
    ) -> Self {
        let mut link_idx = 0usize;
        let mut new_spans = Vec::with_capacity(self.spans.len());

        for span in self.spans {
            if span.action.is_some() {
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
                        s.action = action_fn(link_idx, &s);
                        link_idx += 1;
                    }
                    new_spans.push(s);
                }
            }
        }

        if FALLBACK && link_idx == 0 {
            for span in &mut new_spans {
                if span.action.is_none() {
                    span.action = action_fn(link_idx, span);
                    link_idx += 1;
                }
            }
        }

        self.spans = new_spans;
        self
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
            out.spans.append(&mut line.spans);
            out.classes.append(&mut line.classes);
        }
        out
    }
}

impl<T: Into<Line>> Add<T> for Line {
    type Output = Self;
    fn add(mut self, rhs: T) -> Self {
        let mut line = rhs.into();
        self.spans.append(&mut line.spans);
        self.classes.append(&mut line.classes);
        self
    }
}

impl<T: Into<Line>> AddAssign<T> for Line {
    fn add_assign(&mut self, rhs: T) {
        let mut line = rhs.into();
        self.spans.append(&mut line.spans);
        self.classes.append(&mut line.classes);
    }
}

impl<T: Into<Line>> Extend<T> for Line {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            let mut line = item.into();
            self.spans.append(&mut line.spans);
            self.classes.append(&mut line.classes);
        }
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
    fn test_line_interleave_actions() {
        // Without fallback: bracketed text becomes links, plain text is untouched
        let line = Line::from("Go to [[forest]] or [[inn]]")
            .interleave_actions::<false>(|i, _| Some(Action::SetBit(1, i as u8)));
        assert_eq!(line.spans.len(), 4);
        assert_eq!(line.spans[0].content, "Go to ");
        assert!(line.spans[0].action.is_none());
        assert_eq!(line.spans[1].content, "forest");
        assert!(matches!(line.spans[1].action, Some(Action::SetBit(1, 0))));
        assert_eq!(line.spans[2].content, " or ");
        assert!(line.spans[2].action.is_none());
        assert_eq!(line.spans[3].content, "inn");
        assert!(matches!(line.spans[3].action, Some(Action::SetBit(1, 1))));

        // Without fallback and no brackets: no links
        let line_nobrackets = Line::from("No links here")
            .interleave_actions::<false>(|i, _| Some(Action::SetBit(1, i as u8)));
        assert_eq!(line_nobrackets.spans.len(), 1);
        assert!(line_nobrackets.spans[0].action.is_none());

        // With fallback and no brackets: entire line becomes clickable link
        let line_fallback = Line::from("The old chest is locked.")
            .interleave_actions::<true>(|i, _| Some(Action::SetBit(1, i as u8)));
        assert_eq!(line_fallback.spans.len(), 1);
        assert_eq!(line_fallback.spans[0].content, "The old chest is locked.");
        assert!(matches!(line_fallback.spans[0].action, Some(Action::SetBit(1, 0))));

        // Preserves prototype classes and styles
        let proto_line = Line::from(Span::new("See [[item]] now").cls("highlight"))
            .interleave_actions::<false>(|i, _| Some(Action::SetBit(1, i as u8)));
        assert_eq!(proto_line.spans.len(), 3);
        assert_eq!(proto_line.spans[0].classes, vec!["highlight"]);
        assert_eq!(proto_line.spans[1].classes, vec!["highlight"]);
        assert!(matches!(proto_line.spans[1].action, Some(Action::SetBit(1, 0))));
        assert_eq!(proto_line.spans[2].classes, vec!["highlight"]);
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
}
