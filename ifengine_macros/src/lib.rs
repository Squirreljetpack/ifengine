//! Procedural macros for the `ifengine` interactive fiction framework.
//!
//! See [`elements`](ifengine::elements) for usage details.

extern crate ifengine_core as ifengine;

use proc_macro::TokenStream;

mod choices;
mod elements;
pub(crate) mod helpers;
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
// View
// =========================================================================

/// Push an [`Object`](ifengine::view::Object) to the current [`View`](ifengine::View).
#[proc_macro]
pub fn push(input: TokenStream) -> TokenStream {
    view::push(input)
}

/// Clear all elements from the current [`View`](ifengine::View).
#[proc_macro]
pub fn clear(input: TokenStream) -> TokenStream {
    view::clear(input)
}

/// Append additional content to the last object in the current [`View`](ifengine::View).
///
/// Supports explicit category prefixes (`"choice":`, `"object":`, `"span":` or their unquoted equivalents)
/// to target different view object variants. If no prefix is supplied, it defaults to extending paragraphs.
///
/// # Prefixes & Targets
/// - **`"span":` (or omitted default)**: Appends one or more lines, spans, or string literals to a preceding [`Object::Paragraph`](ifengine::view::Object::Paragraph).
/// - **`"choice":`**: Appends choices to a preceding [`Object::Choice`](ifengine::view::Object::Choice).
///   Accepts any type implementing [`IntoNumberedLine`](ifengine::view::IntoNumberedLine), such as `(u8, Into<Line>)`
///   or directly `Into<Line>` (`&str`, `String`, `Line`, etc.). If the index is omitted (`None`), it automatically
///   assigns `previous index + 1` (or `0` if the choice list is empty).
/// - **`"object":`**: Appends objects (or [`StampedObject`](ifengine::view::StampedObject)s) to a preceding
///   embedded subpage view ([`Object::Embed`](ifengine::view::Object::Embed)).
///
/// If the target object variant does not match the specified category, or if the view is empty, `extend!` safely does nothing.
///
/// # Examples
/// ```rust,ignore
/// // Extend a paragraph with text and spans
/// p!("Hello,");
/// extend!(" {player.name}!");
/// extend!("span": s!(" Welcome!").as_link());
///
/// // Extend a choice menu with auto-incrementing or explicit indices
/// choice! {
///     "Take the left path" => left_room,
///     "Take the right path" => right_room,
/// };
/// extend!("choice": "Inspect the door", (5, "Return to camp"));
///
/// // Extend an embedded view with additional objects
/// EMBED!(subpage);
/// extend!("object": Object::Break);
/// ```
#[proc_macro]
pub fn extend(input: TokenStream) -> TokenStream {
    view::extend(input)
}

/// Push a single paragraph block ([`Object::Paragraph`](ifengine::view::Object::Paragraph)) to the view.
///
/// Constructed from one or more lines, spans, or string literals.
/// Supports optional trailing `:: "metadata"` to attach [`RenderData`](ifengine::view::RenderData).
///
/// # Example
/// ```rust,ignore
/// p!("A dark hallway stretches before you.");
/// p!("HP: {hp}/{max_hp}" :: "stat-line");
/// ```
#[proc_macro]
pub fn p(input: TokenStream) -> TokenStream {
    view::paragraph(input)
}

/// Push multiple separate paragraph blocks ([`Object::Paragraph`](ifengine::view::Object::Paragraph)) to the view.
///
/// Each argument is its own block with standard vertical spacing.
/// Supports optional trailing `:: "metadata"` to attach [`RenderData`](ifengine::view::RenderData).
///
/// # Example
/// ```rust,ignore
/// ps!(
///     "First paragraph.",
///     "Second paragraph.",
/// );
/// ps!("Line 1", "Line 2" :: "stat-block");
/// ```
#[proc_macro]
pub fn ps(input: TokenStream) -> TokenStream {
    view::paragraphs(input)
}

/// Push a single paragraph block ([`Object::Paragraph`](ifengine::view::Object::Paragraph)) to the view with standard vertical margins.
///
/// Constructed from one or more lines, spans, or string literals.
/// Supports optional trailing `:: "metadata"` to attach [`RenderData`](ifengine::view::RenderData).
///
/// # Example
/// ```rust,ignore
/// paragraph!("A dark hallway stretches before you.");
///
/// // paragraph variant (i.e. dialogue)
/// paragraph!("Very well" :: ":Adam");
///
/// // pure signal
/// paragraph!(:: "music-1");
/// ```
#[proc_macro]
pub fn paragraph(input: TokenStream) -> TokenStream {
    view::paragraph(input)
}

/// Push multiple separate paragraph blocks ([`Object::Paragraph`](ifengine::view::Object::Paragraph)) to the view.
///
/// Each argument is its own block with standard vertical spacing.
/// Supports optional trailing `:: "metadata"` to attach [`RenderData`](ifengine::view::RenderData).
///
/// # Example
/// ```rust,ignore
/// paragraphs!(
///     "First paragraph.",
///     "Second paragraph.",
/// );
/// ```
#[proc_macro]
pub fn paragraphs(input: TokenStream) -> TokenStream {
    view::paragraphs(input)
}

/// Markdown heading.
///
/// # Example
/// ```rust,ignore
/// h!("Chapter {chap}: The Journey Begins", 1);
/// ```
#[proc_macro]
pub fn h(input: TokenStream) -> TokenStream {
    view::h(input)
}

/// Horizontal rule (`<hr/>`).
#[proc_macro]
pub fn hr(input: TokenStream) -> TokenStream {
    view::hr(input)
}

/// Push an image from a string literal.
///
/// # Example
/// ```rust,ignore
/// img!("assets/logo.png");
/// img!("https://example.com/logo.png", (100, 50));
/// ```
#[proc_macro]
pub fn img(input: TokenStream) -> TokenStream {
    view::img(input)
}

/// Immediately yield a [`Response::View`](ifengine::core::Response::View) with the current [`View`](ifengine::View).
///
/// Can optionally accept a state key: `YIELD!(key)`. If present, checks if the value stored under
/// that key `.is_some()`, and yields if not (i.e. if `None`).
#[proc_macro]
#[allow(non_snake_case)]
pub fn r#YIELD(input: TokenStream) -> TokenStream {
    view::r#YIELD(input)
}

/// Embed a sub-page view into the current page.
///
/// Calls the target page with a transient Game.
/// If the target page returns `Response::View`, the view is embedded as an `Object::Embed`
/// into the current page and returned as the expression value.
/// If the target page returns any other `Response` variant (`Switch`, `Back`, `Tunnel`, `Exit`, `End`),
/// it is returned immediately from the enclosing page function.
#[proc_macro]
#[allow(non_snake_case)]
pub fn EMBED(input: TokenStream) -> TokenStream {
    view::embed(input)
}

// =========================================================================
// Elements
// =========================================================================

/// Create a [`Span`](ifengine::view::Span).
///
/// # Example
/// ```rust,ignore
/// s!("Gold: {player.gold}")
/// ```
#[proc_macro]
pub fn s(input: TokenStream) -> TokenStream {
    elements::s(input)
}

/// Create a [`Line`](ifengine::view::Line) from `{var}` interpolation, spans, lines, or string literals.
///
/// # Example
/// ```rust,ignore
/// l!("Player {name} (Level {level})")
/// ```
#[proc_macro]
pub fn l(input: TokenStream) -> TokenStream {
    elements::l(input)
}

/// Create a clickable link [`Span`](ifengine::view::Span) navigating to a destination page.
///
/// # Example
/// ```rust,ignore
/// link!("Visit {vendor}'s shop", shop_page)
/// link!("text", target_page)
///
/// // link style without target
/// link!("text")
/// ```
#[proc_macro]
pub fn link(input: TokenStream) -> TokenStream {
    elements::link(input)
}

/// Create a tunnel or exit link [`Span`](ifengine::view::Span).
///
/// # Example
/// ```rust,ignore
/// tun!("Consult with {mentor}", mentor_tunnel)
/// // push a new stack frame
/// tun!("text", target_page)
/// // exit the current tunnel (pop the stack)
/// tun!("text")
/// ```
#[proc_macro]
pub fn tun(input: TokenStream) -> TokenStream {
    elements::tun(input)
}

/// Create a link [`Span`](ifengine::view::Span) that navigates backward.
///
/// - `$e`: Display text.
/// - `$n`: Optional number of steps to go back (defaults to 1).
///
/// # Example
/// ```rust,ignore
/// back!("Return to {previous_room}")
/// ```
#[proc_macro]
pub fn back(input: TokenStream) -> TokenStream {
    elements::back(input)
}

/// Cycle between multiple alternative spans on click.
///
/// Supports three cycling behaviors (defaults to `Stop`):
/// - `Stop`: Advances on click until reaching the last alternative, then stops advancing.
/// - `Cycle`: Advances on click and wraps around to the beginning indefinitely.
/// - `Shuffle`: Picks a random alternative on click (excluding the current one).
///
/// Takes an optional final closure `|idx| { ... }` that receives the current index
/// into the alternatives list, allowing reactive game state updates on click or render.
///
/// # Examples
/// ```ignore
/// // Array literal:
/// alts!(["Look around", "Open the door", "Wait"])
///
/// // Cycle variant with closure capturing current index:
/// alts!(["Red", "Green", "Blue"], Cycle, |idx| {
///     state.color_idx = idx;
/// })
///
/// // Passing a variable or collection expression:
/// let weathers = ["sunny", "cloudy", "rainy"];
/// alts!(weathers, Shuffle, |idx| state.weather = weathers[idx].to_string())
/// ```
#[proc_macro]
pub fn alts(input: TokenStream) -> TokenStream {
    elements::alts(input)
}

/// Use a closure to compute a span based on how many times the span has been clicked.
///
/// # Syntax
/// ```rust,ignore
/// let span_count = read_key!(6);
/// let span = count!((6), |n| format!("Clicked {n} times"));
/// ```
#[proc_macro]
pub fn count(input: TokenStream) -> TokenStream {
    elements::count(input)
}

/// Run code on click.
///
/// An optional `max_clicks` parameter can be provided in final position.
///
/// # Syntax
/// ```rust,ignore
/// p!(click!(span, block))
/// p!(click!(span, block, max_clicks))
/// p!(click!((maybe_key), span, block, max_clicks))
/// ```
///
/// # Note
/// The block only runs on the render after a click, meaning
/// that if the block is used to toggle the visibility of the click element,
/// its definition must execute unconditionally
/// (i.e. be lifted out from where it is pushed).
/// Also see: [`replace!`].
///
/// ```rust,ignore
/// let dismiss = click!("Dismiss", {
///     state.popup = false;
/// });
///
/// if state.popup {
///     p!("--- Popup Notice ---");
///     p!("Atmospheric disturbance detected in sector 7.");
///     p!(dismiss);
/// }
/// ```
#[proc_macro]
pub fn click(input: TokenStream) -> TokenStream {
    elements::click(input)
}

// =========================================================================
// Choices
// =========================================================================

/// Conditionally display one of several choices based on user selection.
///
/// Display a choice menu and morph into the selected choice's paragraph.
///
/// Returns `true` if it has resolved, otherwise `false`.
///
/// # Description
/// The [`choice`] macro takes a list of arms in the form `LHS => RHS`, where both
/// sides implement `Into<`[`Line`](ifengine::view::Line)`>`.
///
/// It works as follows:
/// - If no arm is selected, the LHS values are displayed as a list of clickable choices.
/// - Once a choice is selected, subsequent renders execute the corresponding RHS expression (or closure) and
///   display its result as a [`Paragraph`](ifengine::view::Object::Paragraph).
/// - If `=> RHS` is omitted for an arm, selecting that choice keeps the cleaned LHS line as the paragraph.
///
/// # Optional Key Override
/// An optional key (surrounded in parentheses) can be specified as the first argument.
/// When a choice is clicked, it sets the value of its key to (the u8 value of) its id in [`PageState`](ifengine::core::PageState).
/// By default, a deterministic key is automatically assigned.
/// Multiple LHS values can be specified for the same RHS using `|`.
///
/// # Examples
/// ```rust,ignore
/// choice! {
///     // 1. Static choice without side effects
///     "Take the left path",
///
///     // 2. Closure keeping choice with state side effects:
///     "Search the desk" => |l| {
///         state.found_key = true;
///         l
///     },
///
///     // 3. Extending choice line with '+':
///     "Open the chest" => |l| {
///         l + " — inside you find a gleaming silver dagger."
///     },
///
///     // 4. Multiple LHS values with a closure:
///     "North" | "South" => |l| l + " path taken.",
///
///     // 5. Standard replacement:
///     "Flee" => "You ran away cowardly.",
/// };
/// ```
#[proc_macro]
pub fn choice(input: TokenStream) -> TokenStream {
    choices::choice(input)
}

/// Execute a set of conditional expressions based on user-selected choices.
///
/// Each arm has the form `Choice => Expr`. If a choice was selected, its
/// corresponding expression (the RHS) is executed (in arm order) and evaluated to a [`Line`](ifengine::view::Line).
/// If the evaluated line is non-empty, it is pushed to the page as a paragraph. Any remaining
/// visible choices are pushed last at the bottom.
///
/// Each LHS key is a [`ChoiceVariant`](ifengine::elements::ChoiceVariant), dictating its visibility.
/// Any type that implements `Into<`[`Line`](ifengine::view::Line)`>` will coerce to `Choice::Once`.
/// Any `Option<Into<Line>>` will coerce to `Choice::None` or `Choice::Always`.
///
/// The return type is a `[bool; n]` representing which of the options were hidden (NOT displayed).
///
/// # Example
/// ```rust,ignore
/// // Match arms evaluating to paragraphs, with remaining options shown last:
/// mchoice! {
///     "Ask about the map" => "He shows you the weathered parchment.",
///     "Ask about the dungeon" => "He warns you of the shadows lurking below.",
/// }
///
/// // Dynamic visibility with completion check:
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

/// Display a dynamic list of choices and return the selected choice's identifier.
///
/// # Overview
/// Pushes a choice object constructed from an iterable collection of `(Id, Line)` pairs
/// to the current view.
///
/// Prefer [`dchoice!`] as the more concise constructor of this pattern.
///
/// # Return Value
/// Returns `Option<T>`:
/// - `Some(id)`: The identifier of the selected choice (transmuted from `u8`).
/// - `None`: No choice was selected on this turn (or it was already consumed).
///
/// # Syntax
/// ```text
/// dynamic_choice!((maybe_key), expr)
/// ```
///
/// # Arguments
/// - **MaybeKey**: `(key)` surrounded in parentheses as the first argument. By default, a deterministic key is automatically assigned.
/// - **Choices expression**: Any expression implementing `IntoIterator<Item = (T, L)>`, where:
///   - `T`: The identifier type, which can be cast `as u8` and transmuted from `u8` (e.g., `u8` or a `#[repr(u8)]` unit enum).
///   - `L`: The display label implementing `Into<`[`Line`](ifengine::view::Line)`>`.
///
/// # Example
/// ```rust,ignore
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// #[repr(u8)]
/// enum MenuChoice {
///     Attack,
///     Defend,
///     Flee,
/// }
///
/// let choices = vec![
///     (MenuChoice::Attack, "Attack"),
///     (MenuChoice::Defend, "Defend"),
///     (MenuChoice::Flee, "Flee"),
/// ];
///
/// if let Some(choice) = dynamic_choice!(choices) {
///     match choice {
///         MenuChoice::Attack => { /* handle attack */ }
///         MenuChoice::Defend => { /* handle defend */ }
///         MenuChoice::Flee => { /* handle flee */ }
///     }
/// }
/// ```
#[proc_macro]
pub fn dynamic_choice(input: TokenStream) -> TokenStream {
    choices::dynamic_choice(input)
}

/// A version of [`dynamic_choice!`] with abbreviated syntax.
/// Any match arm for a choice which does not need handling (i.e. one containing a link) can be omitted.
///
/// # Example
/// ```rust,ignore
/// let choices = [
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

/// Interactive paragraph with clickable links delimited by `[[target]]`.
///
/// Returns the clicked link's target `&str` (once per click), or `""` if no link
/// has been clicked yet. Empty bracket pairs `[[]]` are skipped, joining surrounding text.
///
/// # Syntax
/// ```text
/// dparagraph!((maybe_key), expr1, expr2, ..., exprN)
/// ```
///
/// # Example
/// ```rust,ignore
/// match dparagraph!("Go to the [[forest]] or the [[inn]].") {
///     "forest" => NEXT!(p_forest),
///     "inn" => NEXT!(p_inn),
///     _ => {}
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
/// mparagraph!((maybe_key), expr1, expr2, ..., exprN)
/// ```
///
/// # Example
/// ```rust,ignore
/// let clicked = mparagraph!("You see a [[lantern]], a [[rope]], and a [[dagger]].");
/// if clicked[0] {
///     // lantern was clicked
/// }
/// ```
#[proc_macro]
pub fn mparagraph(input: TokenStream) -> TokenStream {
    choices::mparagraph(input)
}

/// Interactive paragraph that disappears or is replaced with new content when clicked.
///
/// Words enclosed in `[[brackets]]` become clickable targets. If no brackets are present,
/// the entire paragraph is clickable.
///
/// Clicking a link displays the replacement text or expression in place of the original paragraph.
/// The replacement can be a string, a [`Line`](ifengine::view::Line), or a closure `|l| ...` that receives
/// the cleaned original [`Line`](ifengine::view::Line).
/// If no replacement is provided, the paragraph disappears.
///
/// The MaybeKey will only override the [`Object`](ifengine::view::Object)'s Id, not the internal state key.
///
/// Returns `true` once clicked and replaced, or `false` beforehand.
///
/// # Syntax
/// ```text
/// replace!((maybe_key), string_expr [=> replacement])
/// ```
///
/// # Examples
/// ```rust,ignore
/// // Disappears on click (no replacement)
/// replace!("The old chest is [[locked]].");
///
/// // Replaces paragraph on click
/// replace!("The gate is [[closed]]." => "The gate swings open.");
///
/// // Extends original text on click using a closure:
/// replace!("The chest is [[shut tight]]." => |l| {
///     l + " You forced it open with a crowbar!"
/// });
///
/// // Conditionally show following content after replacement
/// if replace!("Click [[here]] to reveal options" => "Options revealed:") {
///     choice! {
///         "Option A",
///         "Option B",
///     };
/// }
/// ```
#[proc_macro]
pub fn replace(input: TokenStream) -> TokenStream {
    choices::replace(input)
}

/// Interactive replacement line that does not push to the view, returning a [`Line`](ifengine::view::Line).
///
/// Intended for nesting inside [`replace!`] (or another [`replace_line!`]), allowing chained
/// multi-stage in-place replacements sharing the parent paragraph's transition identity.
///
/// # Syntax
/// ```text
/// replace_line!(string_expr [=> replacement])
/// ```
///
/// # Examples
/// ```rust,ignore
/// replace!("The lock is [[sealed]]." => repl!("The doorway is [[open]]." => "You step inside."));
/// ```
#[proc_macro]
pub fn replace_line(input: TokenStream) -> TokenStream {
    choices::replace_line(input)
}

/// Alias for [`replace_line!`].
#[proc_macro]
pub fn repl(input: TokenStream) -> TokenStream {
    choices::replace_line(input)
}

/// Deprecated alias for [`replace_line!`].
#[proc_macro]
pub fn replacement(input: TokenStream) -> TokenStream {
    choices::replace_line(input)
}

/// Interactive replacement span that does not push to the view, returning a [`Span`](ifengine::view::Span).
///
/// # Syntax
/// ```text
/// replace_span!(string_expr [=> replacement])
/// ```
///
/// # Examples
/// ```rust,ignore
/// p!("The lock is ", replace_span!("sealed" => "open"), ".");
/// p!("Status: ", reps!("active" => "inactive"));
/// ```
#[proc_macro]
pub fn replace_span(input: TokenStream) -> TokenStream {
    choices::replace_span(input)
}

/// Alias for [`replace_span!`].
#[proc_macro]
pub fn reps(input: TokenStream) -> TokenStream {
    choices::replace_span(input)
}

// =========================================================================
// State
// =========================================================================

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

/// Read or evaluate an expression based on persistent page state.
///
/// - `get!(key)`: Reads `Option<u64>` for the given key.
/// - `get!(key, when_some)`: If `key` is present in state (`Some`), evaluates to `when_some` cast to [`Span`](ifengine::view::Span); otherwise default.
/// - `get!(key, when_some, when_none)`: If `key` is present, evaluates to `when_some` cast to [`Span`](ifengine::view::Span); otherwise `when_none` cast to [`Span`](ifengine::view::Span).
///
/// Accepts strings or ints.
///
/// # Examples
/// ```rust,ignore
/// let val = get!("my_key");
/// p!(get!("opened_chest", "The chest stands open.", "A locked chest sits here."));
/// p!(get!("has_gold", "You have gold!"));
/// ```
#[proc_macro]
pub fn get(input: TokenStream) -> TokenStream {
    state::get(input)
}

/// Set a persistent page state key.
///
/// - `set!(key)`: Inserts `0u64` for `key` (so that `get!(key)` returns `Some(0)`).
/// - `set!(key, val)`: Inserts `val` for `key`.
///
/// Accepts strings or ints.
///
/// # Examples
/// ```rust,ignore
/// set!("chest_opened");
/// set!("gold", 50);
/// ```
#[proc_macro]
pub fn set(input: TokenStream) -> TokenStream {
    state::set(input)
}

/// Read a key as a bitmask.
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

/// Increment the value of a key.
///
/// # Example
/// ```rust,ignore
/// inc_key!(my_key);
/// ```
#[proc_macro]
pub fn inc_key(input: TokenStream) -> TokenStream {
    state::inc_key(input)
}

/// Reset a key (remove from state).
///
/// Accepts strings or ints.
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

// =========================================================================
// Debug
// =========================================================================

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
