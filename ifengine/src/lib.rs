//! IFEngine is a rust framework for writing interactive fiction.
//!
//! Other projects in this space include Twine and Inkle.
//!
//! The goal is to enjoy an effortless writing experience, together with all the benefits of the Rust ecosystem.
//!
//! To use this library, you write functions which produce [`Response`](core::Response), eventually resolving to a [`View`].
//! Such functions are called [`Page`](core::Page)s, and are decorated by the [`#[ifview]`](ifview) macro.
//! A starting page is used to initialize the [`Game`](struct@Game), which can then be called upon to yield its current view through [`Game::view`], and updated by interacting with the view.
//! A view consists of a sequence of [`Object`](view::Object)s which you can attach by calling the provided [elements and macros](elements) within the page.

extern crate self as ifengine;

pub use ifengine_core::*;
pub use inventory;

#[cfg(feature = "macros")]
pub use ifengine_macros::ifview;

#[cfg(feature = "macros")]
pub use ifengine_macros::v;

pub mod elements {
    //! Interactive text elements, choices, navigation, and state macros.
    //!
    //! This module exports all authoring macros and element constructors used to compose interactive fiction pages.
    //!
    //! ## Note: `{var}` Interpolation
    //!
    //! String literal arguments in element macros support inline variable interpolation.
    //! Write `{expr}` inside any str literal to expand it:
    //!
    //! ```rust,ignore
    //! p!("Welcome, {player.name}! You have {gold} gold.");
    //! choice! { "Attack {enemy.name}" => { /* ... */ } }
    //! replace!("The [{item.name}] glows faintly.", "You take it.");
    //! ```
    //!
    //! - Values are borrowed (`&expr`).
    //! - `{{` and `}}` can be used to emit literal braces.
    //!
    //! # Quick Reference / Cheat Sheet
    //!
    //! | Macro / Element | Syntax Example | Description |
    //! | :--- | :--- | :--- |
    //! | [`s!`](s) | `s!("Badge: {name}")` or `s!("sealed" => "open")` | Creates an inline `Span` with `{var}` interpolation, or an in-place clickable replacement `Span` when `=>` is present. |
    //! | [`l!`](l) | `l!("Player {name}")` or `l!("The door is [shut]." => "The door is open.")` | Creates a `Line` from `{var}` interpolation, spans, lines, or strings, or an in-place clickable replacement `Line` when `=>` is present. |
    //! | [`link!`](link) | `link!("Visit {vendor}'s Shop", shop_room)` | Creates an inline link `Span` navigating to a destination page. |
    //! | [`tun!`](tun) | `tun!("Consult with {mentor}", mentor_tunnel)` | Creates an inline link `Span` calling a page as a tunnel subroutine. |
    //! | [`back!`](back) | `back!("Return to {room}")` or `back!("Rewind", 2)` | Creates an inline link `Span` stepping back in navigation history. |
    //! | [`h!`](h) | `h!("Chapter {chap}: The Journey", 1)` | Adds a heading with `{var}` interpolation. |
    //! | [`click!`](click) | `click!("Search desk", { state.found_key = true; })` or `click!(\|n\| format!("Clicked {n} times"))` | Creates an interactive `Span` that triggers a callback on click; the displayed expression may be a closure receiving the click count. |
    //! | [`alts!`](alts) | `alts!(Cycle: ["North", "South", "East", "West"])` | Creates cycling, stopping, or shuffled text alternative spans. |
    //! | [`p!`](paragraph) | `p!("Go to [forest] or [inn]")` | Adds a paragraph block; words in `[x]` become clickable links, returns `PData` (clicked target via `.as_str()`, plus `bits`/`key`). Use `[[` for a literal `[`. |
    //! | [`ps!`](ps) | `ps!("Line 1", "Line 2" :: "stats")` | Adds multiple separate paragraph blocks with `{var}` interpolation and an optional trailing class/render-data trailer (does not parse `[x]` link actions; use `p!` or explicit `link!`). |
    //! | [`choice!`](choice) | `choice! { "Open door" => p_door, "Turn back" => p_back }` | Displays a static list of clickable choices for page navigation or actions. |
    //! | [`dchoice!`](dchoice) | `dchoice!(items.into_iter().map(...))` | Displays a dynamic choice list generated at runtime from an iterator/collection. |
    //! | [`mchoice!`](mchoice) | `mchoice!((key), ...)` | Choice menu with an explicit state key override. |
    //! | [`replace!`](replace) | `replace!("Chest is [locked].", "Unlocked!")` | Clickable paragraph that transitions into a replacement line or collapses to 0 height. |
    //! | [`br!`](br) | `br!()` or `br!(2)` | Inserts vertical empty line space (`Object::Empty(n)`). |
    //! | [`hr!`](hr) | `hr!()` | Inserts a horizontal divider break (`Object::Break`). |
    //! | [`NEXT!`](NEXT) | `NEXT!(next_page)` | Flow control: transition to the specified page function. |
    //! | [`EMBED!`](EMBED) | `EMBED!(sub_page)` | Flow control: evaluate sub-page with transient Game and embed View; automatically uses the current page context. |
    //! | [`BACK!`](BACK) | `BACK!()` or `BACK!(2)` | Flow control: navigate back 1 (or `n`) steps in history. |
    //! | [`TUN!`](TUN) | `TUN!(tunnel_page)` | Flow control: enter a tunnel subroutine page. |
    //! | [`END!`](END) | `END!()` | Flow control: terminate story execution. |
    //! | [`YIELD!`](YIELD) | `YIELD!()` or `YIELD!(keys...)` | Flow control: yield current view immediately, optionally conditionally if none of the keys are Some. |
    //! | [`x!`](extend) | `extend!("more text")` or `extend!(Choice, "new choice")` | Extends the previous object in the view (Paragraph, Choice, Embed). |
    //! | [`get!`](get) | `get!("key")` or `get!("key", "when_none" => "when_some")` | Reads state value, or evaluates a conditional `Line` based on key presence. |
    //! | [`v!`](v) | `v!("key")` or `v!(my_page)` | Reports key presence, or whether a page has been visited. |
    //! | [`once!`](once) | `once! { ... }` or `once!(3: { ... })` | Executes a block once (or up to `n` times); `0:` means only on a fresh page render. |
    //! | [`set!`](set) | `set!("key")` or `set!("key", val)` | Sets state key to 0 or a specified value. |
    //! | [`read_key_mask!`](read_key_mask) | `read_key_mask!(KEY_ID)` | Unpacks a `u64` state value into a boolean bitmask array. |
    //! | [`reset_key!`](reset_key) | `reset_key!(KEY_ID)` | Remove a entry from state. |

    pub use ifengine_core::elements::*;
    pub use ifengine_core::{BACK, END, NEXT, TUN};

    #[cfg(feature = "macros")]
    pub use ifengine_macros::*;

    #[cfg(feature = "macros")]
    pub use ifengine_macros::{
        EMBED as emb, YIELD as Y, extend as x, mchoice as choices, paragraph as p, paragraphs as ps,
    };
}
