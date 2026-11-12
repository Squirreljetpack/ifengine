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
/// Supports explicit category target variants (`Choice`, `Object`, `Span`)
/// to target different view object variants. If no variant is supplied, it defaults to extending paragraphs (`Span`).
///
/// # Variants & Targets
/// - **`Span` (or omitted default)**: Appends one or more lines, spans, or string literals to a preceding [`Object::Paragraph`](ifengine::view::Object::Paragraph).
/// - **`Choice`**: Appends choices to a preceding [`Object::Choice`](ifengine::view::Object::Choice).
///   Accepts any type implementing [`IntoNumberedLine`](ifengine::view::IntoNumberedLine), such as `(u8, Into<Line>)`
///   or directly `Into<Line>` (`&str`, `String`, `Line`, etc.). If the index is omitted (`None`), it automatically
///   assigns `previous index + 1` (or `0` if the choice list is empty).
/// - **`Object`**: Appends objects (or [`StampedObject`](ifengine::view::StampedObject)s) to a preceding
///   embedded subpage view ([`Object::Embed`](ifengine::view::Object::Embed)).
///
/// If the target object variant does not match the specified category, or if the view is empty, `extend!` safely does nothing.
///
/// # Examples
/// ```rust,ignore
/// // Extend a paragraph with text and spans
/// p!("Hello,");
/// extend!(" {player.name}!");
/// extend!(Span, s!(" Welcome!").as_link());
///
/// // Extend a choice menu with auto-incrementing or explicit indices
/// choice! {
///     "Take the left path" => left_room,
///     "Take the right path" => right_room,
/// };
/// extend!(Choice, "Inspect the door", (5, "Return to camp"));
///
/// // Extend an embedded view with additional objects
/// EMBED!(subpage);
/// extend!(Object, Object::Break);
/// ```
#[proc_macro]
pub fn extend(input: TokenStream) -> TokenStream {
    view::extend(input)
}

/// Push a single paragraph block ([`Object::Paragraph`](ifengine::view::Object::Paragraph)) to the view.
///
/// Constructed from one or more lines, spans, or string literals with `{var}` interpolation.
/// Words enclosed in `[target]` become clickable interactive links (use `[[` for a literal `[`).
/// Returns a [`PData`](ifengine::elements::PData).
///
/// An optional prefix `max_clicks:` limits the total number of clicks across all links before
/// all links are disabled. `max_clicks: 0` disables every link from the start; omitting the
/// prefix leaves the links unbounded.
///
/// # Trailer
/// An optional trailer attaches CSS classes and [`RenderData`](ifengine::view::RenderData):
/// ```text
/// :: "renderdata"
/// :: [class1, class2, ...]
/// :: [class1, class2, ...], "renderdata"
/// ```
/// Classes are the bracketed, comma-separated expressions after `::`; the trailing string
/// literal is the render data. Both parts are optional.
///
/// # Syntax
/// ```text
/// paragraph!([(maybe_key),] [max_clicks:] expr1, expr2, ..., exprN [trailer])
/// ```
///
/// # Example
/// ```rust,ignore
/// match paragraph!("Go to the [forest] or the [inn].").as_str() {
///     "forest" => NEXT!(p_forest),
///     "inn" => NEXT!(p_inn),
///     _ => {}
/// }
///
/// // Non-interactive (no brackets) — just displays text
/// paragraph!("A dark hallway stretches before you.");
///
/// // With classes and render data
/// paragraph!("A dark hallway stretches before you." :: ["in-500"], "popup");
///
/// // Dialogue variant
/// paragraph!("Very well" :: ":Adam");
/// ```
#[proc_macro]
pub fn paragraph(input: TokenStream) -> TokenStream {
    view::paragraph(input)
}

/// Push multiple separate paragraph blocks ([`Object::Paragraph`](ifengine::view::Object::Paragraph)) to the view.
///
/// Each argument is its own block with standard vertical spacing.
///
/// > [!NOTE]
/// > Unlike [`paragraph!`], `paragraphs!` does not parse `[target]` link actions.
///
/// Supports the trailing annotations as [`paragraph!`]:
/// ```text
/// :: "renderdata"
/// :: [class1, class2, ...]
/// :: [class1, class2, ...], "renderdata"
/// ```
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

/// Push vertical empty whitespace (`Object::Empty(n)`).
///
/// Accepts an optional count `n` (defaults to `1` if omitted).
///
/// # Example
/// ```rust,ignore
/// br!();  // 1 empty line
/// br!(2); // 2 empty lines
/// ```
#[proc_macro]
pub fn br(input: TokenStream) -> TokenStream {
    view::br(input)
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
/// Can optionally accept a sequence of state keys: `YIELD!(key, ...)`. If present, checks if the values stored under
/// those keys are `.is_some()`, and yields only if none of them are `is_some` (i.e. if all are `None`).
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
/// The context argument is optional inside `#[ifview]` functions; when omitted,
/// the generated local `__ifengine_ctx!()` accessor is used.
#[proc_macro]
#[allow(non_snake_case)]
pub fn EMBED(input: TokenStream) -> TokenStream {
    view::embed(input)
}

// =========================================================================
// Elements
// =========================================================================

/// Create a [`Span`](ifengine::view::Span), or an interactive replacement span when `=>` is present.
///
/// # Examples
/// ```rust,ignore
/// s!("Gold: {player.gold}")
///
/// // Replaced in-place on click:
/// s!("sealed" => "open")
/// s!((my_key), "sealed" => "open")
/// ```
#[proc_macro]
pub fn s(input: TokenStream) -> TokenStream {
    elements::s(input)
}

/// Create a [`Line`](ifengine::view::Line) from `{var}` interpolation, spans, lines, or string literals,
/// or an interactive replacement line with clickable `[bracketed]` targets when `=>` is present.
///
/// # Examples
/// ```rust,ignore
/// l!("Player {name} (Level {level})")
///
/// // Replaced in-place when a bracketed link is clicked:
/// l!("The doorway is [open]." => "You step inside.")
/// l!((my_key), "The doorway is [open]." => "You step inside.")
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
/// Supports three cycling behaviors (defaults to `Stop`), given as a prefix:
/// - `Stop:`: Advances on click until reaching the last alternative, then stops advancing.
/// - `Cycle:`: Advances on click and wraps around to the beginning indefinitely.
/// - `Shuffle:`: Picks a random alternative on click (excluding the current one).
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
/// alts!(Cycle: ["Red", "Green", "Blue"], |idx| {
///     state.color_idx = idx;
/// })
///
/// // Passing a variable or collection expression:
/// let weathers = ["sunny", "cloudy", "rainy"];
/// alts!(Shuffle: weathers, |idx| state.weather = weathers[idx].to_string())
/// ```
#[proc_macro]
pub fn alts(input: TokenStream) -> TokenStream {
    elements::alts(input)
}

/// Run code on click.
///
/// An optional `(maybe_key)` and `max_clicks:` prefix can be provided. `max_clicks` is the
/// number of clicks the span accepts; `0` disables it immediately. When `max_clicks` is omitted,
/// a span with a block or closure stays unbounded, while a body-less span becomes non-interactive
/// after its first click.
///
/// The displayed expression may be a closure `|clicks| ...` that receives the number of
/// times the span has been clicked.
///
/// # Syntax
/// ```rust,ignore
/// p!(click!(span))
/// p!(click!(span, block))
/// p!(click!(max_clicks: span, block))
/// p!(click!((maybe_key), max_clicks: span, block))
/// p!(click!(|clicks| format!("Clicked {clicks} times"), block))
/// ```
///
/// # Note
/// The block only runs on the render after a click, meaning
/// that if the block is used to toggle the visibility of the click element,
/// its definition must execute unconditionally
/// (i.e. be lifted out from where it is pushed).
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
/// Display a choice menu which transforms into the selected choice's paragraph.
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
/// - If `=> RHS` is omitted for an arm, selecting that choice keeps the LHS line as the paragraph.
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

/// Interactive paragraph that disappears or is replaced with new content when clicked.
///
/// Words enclosed in `[brackets]` become clickable targets.
/// If no brackets are present, the entire paragraph is clickable.
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
/// replace!("The old chest is [locked].");
///
/// // Replaces paragraph on click
/// replace!("The gate is [closed]." => "The gate swings open.");
///
/// // Extends original text on click using a closure:
/// replace!("The chest is [shut tight]." => |l| {
///     l + " You forced it open with a crowbar!"
/// });
///
/// // Conditionally show following content after replacement
/// if replace!("Click [here] to reveal options" => "Options revealed:") {
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

// =========================================================================
// State
// =========================================================================

/// Executes a block only once (or up to `n` times with an `n:` prefix).
///
/// The run count is stored under an automatic key and the block runs while the count is
/// below the limit. If `0:` is prefixed, the block runs only on a fresh page render.
///
/// # Syntax
/// ```rust,ignore
/// once! { /* code */ }        // limit 1
/// once!(3: { /* code */ })    // limit 3
/// once!(0: { /* code */ })    // only on a fresh page render
/// ```
#[proc_macro]
pub fn once(input: TokenStream) -> TokenStream {
    state::once(input)
}

/// Read or evaluate an expression based on persistent page state.
///
/// - `get!(key)`: Reads the `Option<u64>` stored for the key.
/// - `get!(key, when_some)`: Evaluates to `when_some` when the key is present, and to an
///   empty [`Line`](ifengine::view::Line) otherwise.
/// - `get!(key, when_none => when_some)`: `when_none` when absent, `when_some` when present.
/// - `get!(key, when_some <= when_none)`: reversed arrow, the leading side is the present case.
///
/// Each side is a single expression implementing `Into<`[`Line`](ifengine::view::Line)`>`.
/// In the conditional forms the key may combine several
/// number/string keys with `|` (or) and `&` (and), with `()` for grouping.
///
/// # Examples
/// ```rust,ignore
/// let val = get!("my_key");
/// p!(get!("opened_chest" , "A locked chest sits here." => "The chest stands open."));
/// p!(get!("has_gold", "You have gold!"));
/// ```
#[proc_macro]
pub fn get(input: TokenStream) -> TokenStream {
    state::get(input)
}

/// Reports whether a page-state key is set or a page has been visited.
///
/// A string or integer operand is a page-state key, equivalent to
/// `get!(key).is_some()`. A path operand names a page and reports whether that page
/// has been rendered. Operands may be combined with `|` (or) and `&` (and).
///
/// # Examples
/// ```rust,ignore
/// if v!("door_open") {
///     p!("The door already stands open.");
/// }
///
/// if v!(chap1::p1 | "intro_seen") {
///     p!("You have been here before.");
/// }
/// ```
#[proc_macro]
pub fn v(input: TokenStream) -> TokenStream {
    state::v(input)
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
/// A tag is attached to the current [`View`](ifengine::view::View) and, when
/// sticky, persisted in the global game tags across pages. Tags are sticky by
/// default. Prefix with `Once:` to keep the tag on the current render only, or
/// `Game:` to record it in the persisted tags without adding it to the current view.
///
/// # Examples
/// ```rust,ignore
/// tag!(my_value);          // sticky tag (view and persisted)
/// tag!(Sticky: my_value);  // sticky tag, explicit
/// tag!(Once: my_value);    // current render only
/// tag!(Game: my_value);    // persisted tags only
/// ```
#[proc_macro]
pub fn tag(input: TokenStream) -> TokenStream {
    state::tag(input)
}

/// Removes a tag from the current [`View`](ifengine::view::View) and/or the
/// persisted [`GameTags`](ifengine::core::GameTags).
///
/// Tags are sticky by default: with no prefix (or `Sticky:`) the tag is removed
/// from both the current view and the persisted tags. `Once:` removes it from the
/// current render only; `Game:` removes it from the persisted tags only.
///
/// # Examples
/// ```rust,ignore
/// untag!(my_value);          // current view and persisted tags
/// untag!(Sticky: my_value);  // current view and persisted tags
/// untag!(Once: my_value);    // current view only
/// untag!(Game: my_value);    // persisted tags only
/// ```
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
