//! Procedural macros for the `ifengine` interactive fiction framework.
//!
//! See [`elements`](ifengine::elements) for usage details.

extern crate ifengine_core as ifengine;

use proc_macro::TokenStream;

mod choices;
mod elements;
mod nodes;
mod state;
mod view;

// =========================================================================
// Attributes
// =========================================================================

/// Decorate your page functions with this attribute.
///
/// The function must take your game state as a parameter, and return `()`.
/// This macro will rewrite your function to receive a `&mut` [`struct@ifengine::Game`] and return a [`Response`](ifengine::core::Response),
/// as well as enabling usage of [`elements`](ifengine::elements) to produce that response (which in most cases will be a [`View`](ifengine::View)).
///
/// # Examples
/// ```rust,ignore
/// #[ifview]
/// pub fn p1(s: &mut State) {
///     h!("SALTWRACK", 3); // heading level 3
///     p!(link!("BEGIN", p2)); // Link to the next page
/// }
///
/// // ----- mod.rs -----
/// pub type Game = ifengine::Game<State>;
/// pub fn new() -> Game {
///    ifengine::Game!(chap1::p1)
/// }
/// ```
#[proc_macro_attribute]
pub fn ifview(attr: TokenStream, item: TokenStream) -> TokenStream {
    view::ifview(attr, item)
}

// =========================================================================
// Choices
// =========================================================================

/// Conditionally display one of several choices based on user selection.
///
/// Returns `true` if it has resolved, otherwise `false`.
///
/// # Description
/// The [`choice`] macro takes a list of arms in the form `LHS => RHS`, where both
/// sides implement `Into<`[`Line`](ifengine::view::Line)`>`. It works as follows:
///
/// - If no arm is selected, the LHS values are displayed as a list of lines.
/// - Once a choice is selected, subsequent renders execute the corresponding RHS expression and
///   display its result.
///
/// # Optional Key Override
/// An optional key (surrounded in parentheses) can be specified as the first argument.
/// When a choice is clicked, it sets the value of its key to (the u8 value of) its id in [`PageState`](ifengine::core::PageState).
/// By default, a deterministic key is automatically assigned.
/// Multiple LHS values can be specified for the same RHS using `|`.
///
/// # Example
/// ```rust,ignore
/// choice! {
///     "1" => "Chose 1",
///     "2" | "3" => {
///         "Chose 2 or 3"
///     },
/// };
/// ```
#[proc_macro]
pub fn choice(input: TokenStream) -> TokenStream {
    choices::choice(input)
}

/// Execute a set of conditional expressions based on user-selected choices.
///
/// Each arm has the form `Choice => Expr`. If a choice was selected, its
/// corresponding expression (the RHS) is executed (executions occur in order), regardless of whether
/// the choice's key (the LHS) is currently visible.
///
/// Each LHS key is a [`ChoiceVariant`](ifengine::elements::ChoiceVariant), dictating its visibility.
/// Any type that implements `Into<`[`Line`](ifengine::view::Line)`>` will coerce to `Choice::Once`.
/// Any `Option<Into<Line>>` will coerce to `Choice::None` or `Choice::Always`.
///
/// The return type is a `[bool; n]` representing which of the options were hidden (NOT displayed).
///
/// # Example
/// ```rust,ignore
/// if mchoice! {
///    s.c1.name.is_empty().then_some(link!("member_1_choice_1", _oracle_1)),
///    s.c1.name.is_empty().then_some(link!("member_1_choice_2", _oracle_2)),
///    s.c2.name.is_empty().then_some(link!("member_2_choice_1", _walker_1)),
///    s.c2.name.is_empty().then_some(link!("member_2_choice_2", _walker_2)),
///    (!s.part1.seen.contains("special_event")).then_some(link!("special_event", _interpreter_2))
/// }.all() {
///    NEXT!(p6)
/// }
/// ```
#[proc_macro]
pub fn mchoice(input: TokenStream) -> TokenStream {
    choices::mchoice(input)
}

/// Executes code for a set of selectable choices. Prefer to use [`dchoice!`] for brevity.
///
/// # Overview
/// This macro displays a list of choices, and registers a corresponding handler
/// for each selection. The handler is specified as a `match` expression, where
/// each arm corresponds to a choice and contains the code to execute when
/// that choice is selected. Unlike the other choice elements ([`choice!`], [`mchoice!`]),
/// the conditional expression is evaluated only the first time its choice is selected.
/// The intent is that the arms are used to set values for the user's custom [`GameContext`](ifengine::core::GameContext).
///
/// # Arguments
/// - Optional key (surrounded in parentheses)
/// - **Choices list**: A `Vec<(Id, Line)>` representing the selectable options. The Id can either be a `#[repr(u8)]` Unit Enum or a pure `u8`.
/// - **Handler**: A `match` statement handling each choice.
///
/// # Example
/// ```rust,ignore
/// let choices = vec![
///     (0, l!("A")),
///     (1, l!("B")),
///     (2, l!("C")),
/// ];
///
/// if let Some(x) = dynamic_choice!(choices) {
///     match x {
///         0 => "A clicked",
///         1 => "B clicked",
///         2 => "C clicked",
///     }
/// }
/// ```
#[proc_macro]
pub fn dynamic_choice(input: TokenStream) -> TokenStream {
    choices::dynamic_choice(input)
}

/// A version of [`dynamic_choice!`] with slightly abbreviated syntax.
///
/// # Example
/// ```rust,ignore
/// let choices = vec![
///     l!("A"),
///     l!("B"),
///     l!("C"),
/// ];
/// dchoice! { choices,
///     0 => "A clicked",
///     1 => "B clicked",
///     2 => "C clicked",
/// }
/// ```
#[proc_macro]
pub fn dchoice(input: TokenStream) -> TokenStream {
    choices::dchoice(input)
}

/// Create an interactive paragraph parsing wiki-style links with single-selection tracking.
///
/// Interactive text sections are automatically added from text delimited by `[[target]]` or `[[target|label]]` (Also see: [`mparagraph!`]).
/// Clicking any link records the selection under the macro's internal key and re-renders the page.
///
/// On render, this macro returns `Option<String>`: `Some(target)` with the clicked link's token value,
/// or `None` if no link has been clicked yet.
///
/// # Syntax
/// ```text
/// dparagraph!((maybe_key), expr1, expr2, ..., exprN)
/// ```
///
/// # Example
/// ```rust,ignore
/// if let Some(target) = dparagraph!("Go to [[forest]] or [[inn|the cozy inn]].") {
///     match target.as_str() {
///         "forest" => NEXT!(p_forest),
///         "inn" => NEXT!(p_inn),
///         _ => {}
///     }
/// }
/// ```
#[proc_macro]
pub fn dparagraph(input: TokenStream) -> TokenStream {
    choices::dparagraph(input)
}

/// Create an interactive paragraph parsing wiki-style links with multi-selection tracking.
///
/// Interactive text sections are automatically added from text delimited by `[[target]]` (Also see: [`dparagraph!`]).
/// Multiple links can be clicked across interactions; their clicked states are tracked concurrently via a bitmask in page state.
///
/// On render, this macro returns a `Vec<bool>` where each boolean reflects whether the corresponding
/// bracketed link has been clicked since page load.
///
/// # Syntax
/// ```text
/// mparagraph!((maybe_key), expr)
/// ```
///
/// # Example
/// ```rust,ignore
/// let clicked = mparagraph!("You see a [[lantern]], a [[rope]], and a [[dagger]].");
/// if clicked.get(0) == Some(&true) {
///     // lantern was clicked
/// }
/// ```
#[proc_macro]
pub fn mparagraph(input: TokenStream) -> TokenStream {
    choices::mparagraph(input)
}

// =========================================================================
// Elements
// =========================================================================

/// Push an [`Object`](ifengine::view::Object) to the current [`View`](ifengine::View).
#[proc_macro]
pub fn push(input: TokenStream) -> TokenStream {
    elements::push(input)
}

/// Push an unspaced plain text line ([`Object::Text`](ifengine::view::Object::Text)) to the view without paragraph margins.
///
/// Constructed from one or more spans or lingual string expressions.
///
/// # Custom Styling Metadata
/// A trailing [`RenderData`](ifengine::view::RenderData) (`&'static str`) can be specified following `::`.
///
/// # Example
/// ```rust,ignore
/// text!("Hello, world!");
/// text!("Hello, ", "world!" :: "my_render_data");
/// ```
#[proc_macro]
pub fn text(input: TokenStream) -> TokenStream {
    elements::text(input)
}

/// Push multiple unspaced plain text lines ([`Object::Text`](ifengine::view::Object::Text)) in sequence to the view.
///
/// Each argument is added as a separate line without paragraph vertical margins. See [`text!`].
///
/// # Example
/// ```rust,ignore
/// texts!("Line 1", "Line 2");
/// ```
#[proc_macro]
pub fn texts(input: TokenStream) -> TokenStream {
    elements::texts(input)
}

/// Push a single paragraph block ([`Object::Paragraph`](ifengine::view::Object::Paragraph)) to the view with standard vertical paragraph margins.
///
/// Constructed from one or more spans or lingual string expressions.
///
/// # Example
/// ```rust,ignore
/// paragraph!("A dark hallway stretches before you.");
/// paragraph!(s!("Gold: "), s!(player.gold));
/// ```
#[proc_macro]
pub fn paragraph(input: TokenStream) -> TokenStream {
    elements::paragraph(input)
}

/// Push multiple separate paragraph blocks ([`Object::Paragraph`](ifengine::view::Object::Paragraph)) to the view.
///
/// Each argument is pushed as its own paragraph block with standard vertical spacing between blocks.
///
/// # Example
/// ```rust,ignore
/// paragraphs!(
///     "First paragraph of the scene.",
///     "Second paragraph following a vertical margin.",
/// );
/// ```
#[proc_macro]
pub fn paragraphs(input: TokenStream) -> TokenStream {
    elements::paragraphs(input)
}

/// Create a [`Span`](ifengine::view::Span) with an automatic element key.
#[proc_macro]
pub fn s(input: TokenStream) -> TokenStream {
    elements::s(input)
}

/// Create a [`Line`](ifengine::view::Line) from one or more [`Span`](ifengine::view::Span)s with an automatic element key.
#[proc_macro]
pub fn l(input: TokenStream) -> TokenStream {
    elements::l(input)
}

/// Create a clickable link [`Span`](ifengine::view::Span) with an automatic element key.
///
/// - `link!("Click me", NextPage)`
/// - `link!("Click me")`
#[proc_macro]
pub fn link(input: TokenStream) -> TokenStream {
    elements::link(input)
}

/// Create a tunnel or exit link [`Span`](ifengine::view::Span) with an automatic element key.
///
/// - `tun!("Next", TargetPage)`
/// - `tun!("Exit")`
#[proc_macro]
pub fn tun(input: TokenStream) -> TokenStream {
    elements::tun(input)
}

// =========================================================================
// View Elements
// =========================================================================

/// Push an image from a string literal.
///
/// # Example
/// ```rust,ignore
/// img!("assets/logo.png");
/// img!("https://example.com/logo.png", (100, 50));
/// ```
#[proc_macro]
pub fn img(input: TokenStream) -> TokenStream {
    elements::img(input)
}

/// Markdown heading.
///
/// # Example
/// ```rust,ignore
/// h!("Title", 2);
/// ```
#[proc_macro]
pub fn h(input: TokenStream) -> TokenStream {
    elements::h(input)
}

/// Horizontal rule (`<hr/>`).
///
/// # Example
/// ```rust,ignore
/// hr!();
/// ```
#[proc_macro]
pub fn hr(input: TokenStream) -> TokenStream {
    elements::hr(input)
}

// =========================================================================
// State & Navigation
// =========================================================================

/// Cycle between multiple alternative spans on click.
///
/// # Examples
/// ```ignore
/// alts!([
///     "Look around",
///     "Open the door",
///     "Wait",
/// ])
/// ```
#[proc_macro]
pub fn alts(input: TokenStream) -> TokenStream {
    state::alts(input)
}

/// Use a closure to compute a span based on how many times the span has been clicked.
///
/// # Syntax
/// ```rust,ignore
/// let span_count = read_key!(6);
/// let span = count!((6), |val| "span");
/// ```
#[proc_macro]
pub fn count(input: TokenStream) -> TokenStream {
    state::count(input)
}

/// Run code on click.
///
/// # Syntax
/// ```rust,ignore
/// p!(click!(span, block))
/// ```
#[proc_macro]
pub fn click(input: TokenStream) -> TokenStream {
    state::click(input)
}

/// Run a function only once when the page is first loaded.
///
/// # Syntax
/// ```rust,ignore
/// fresh!(|| { /* code */ });
/// ```
#[proc_macro]
pub fn fresh(input: TokenStream) -> TokenStream {
    state::fresh(input)
}

/// Create a link [`Span`](ifengine::view::Span) that navigates backward.
///
/// - `$e`: Display text.
/// - `$n`: Optional number of steps to go back (defaults to 1).
#[proc_macro]
pub fn back(input: TokenStream) -> TokenStream {
    state::back(input)
}

/// Immediately yield a [`Response::View`](ifengine::core::Response::View) with the current [`View`](ifengine::View).
///
/// This returns `!`, exiting the current function.
#[proc_macro]
#[allow(non_snake_case)]
pub fn r#YIELD(input: TokenStream) -> TokenStream {
    state::r#YIELD(input)
}

/// Read the value of a key in the internal [`PageState`](ifengine::core::PageState).
///
/// # Example
/// ```rust,ignore
/// let value = read_key!(my_key);
/// ```
#[proc_macro]
pub fn read_key(input: TokenStream) -> TokenStream {
    state::read_key(input)
}

/// Read a key as a bitmask. See [`read_key!`].
///
/// # Example
/// ```rust,ignore
/// let mask = read_key_mask!(my_key); // [bool; 64]
/// let mask = read_key_mask!(my_key, 5); // [bool; 5]
/// ```
#[proc_macro]
pub fn read_key_mask(input: TokenStream) -> TokenStream {
    state::read_key_mask(input)
}

/// Set a key to a value. See [`read_key!`].
///
/// # Example
/// ```rust,ignore
/// set_key!(my_key, 42);
/// ```
#[proc_macro]
pub fn set_key(input: TokenStream) -> TokenStream {
    state::set_key(input)
}

/// Set individual bits of a key to true. See [`read_key_mask!`].
///
/// # Example
/// ```rust,ignore
/// set_key_mask!(my_key, 0, 2, 4);
/// ```
#[proc_macro]
pub fn set_key_mask(input: TokenStream) -> TokenStream {
    state::set_key_mask(input)
}

/// Clear individual bits of a key. See [`read_key_mask!`].
///
/// # Example
/// ```rust,ignore
/// unset_key_mask!(my_key, 1, 3);
/// ```
#[proc_macro]
pub fn unset_key_mask(input: TokenStream) -> TokenStream {
    state::unset_key_mask(input)
}

/// Increment the value of a key. See [`read_key!`].
///
/// # Example
/// ```rust,ignore
/// inc_key!(my_key);
/// ```
#[proc_macro]
pub fn inc_key(input: TokenStream) -> TokenStream {
    state::inc_key(input)
}

/// Reset (remove) a key from state. See [`read_key!`].
///
/// # Example
/// ```rust,ignore
/// reset_key!(my_key);
/// ```
#[proc_macro]
pub fn reset_key(input: TokenStream) -> TokenStream {
    state::reset_key(input)
}

/// Tags the current page (see [`GameTags`](ifengine::core::GameTags)).
///
/// Pass `Sticky` to persist the tag between pages.
///
/// # Examples
/// ```rust,ignore
/// tag!(my_value);          // non-sticky tag
/// tag!(my_value, Sticky);  // sticky tag
/// tag!(my_value, Once);    // apply only once
/// ```
#[proc_macro]
pub fn tag(input: TokenStream) -> TokenStream {
    state::tag(input)
}

/// Removes a tag from [`GameTags`](ifengine::core::GameTags).
#[proc_macro]
pub fn untag(input: TokenStream) -> TokenStream {
    state::untag(input)
}

/// Returns whether the current function is running in a [`Simulation`](ifengine::run::Simulation).
#[proc_macro]
pub fn in_sim(input: TokenStream) -> TokenStream {
    state::in_sim(input)
}

/// Debug display the current [`PageState`](ifengine::core::PageState).
#[proc_macro]
pub fn page_dbg(input: TokenStream) -> TokenStream {
    state::page_dbg(input)
}

/// Debug display the current view.
#[proc_macro]
pub fn view_dbg(input: TokenStream) -> TokenStream {
    state::view_dbg(input)
}
