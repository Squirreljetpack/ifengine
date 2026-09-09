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

pub mod elements {
    //! Interactive text elements, choices, navigation, and state macros.
    //!
    //! This module exports all authoring macros and element constructors used to compose interactive fiction pages.
    //!
    //! ## Note: `{var}` Interpolation
    //!
    //! String literal arguments in element macros support inline variable interpolation.
    //! Write `{expr}` inside any &'static str to expand it:
    //!
    //! ```rust,ignore
    //! p!("Welcome, {player.name}! You have {gold} gold.");
    //! choice! { "Attack {enemy.name}" => { /* ... */ } }
    //! replace!("The [[{item.name}]] glows faintly.", "You take it.");
    //! ```
    //!
    //! - Values are borrowed (`&expr`).
    //! - `{{` and `}}` can be used to emit literal braces.
    //!
    //! # Quick Reference / Cheat Sheet
    //!
    //! | Macro / Element | Syntax Example | Description |
    //! | :--- | :--- | :--- |
    //! | [`s!`](s) | `s!("Badge: {name}")` | Creates a styled inline `Span` with `{var}` interpolation or from expressions. |
    //! | [`l!`](l) | `l!("Player {name} (Level {level})")` | Creates a `Line` from `{var}` interpolation or multiple `Span`s. |
    //! | [`link!`](link) | `link!("Visit {vendor}'s Shop", shop_room)` | Creates an inline link `Span` navigating to a destination page. |
    //! | [`tun!`](tun) | `tun!("Consult with {mentor}", mentor_tunnel)` | Creates an inline link `Span` calling a page as a tunnel subroutine. |
    //! | [`back!`](back) | `back!("Return to {room}")` or `back!("Rewind", 2)` | Creates an inline link `Span` stepping back in navigation history. |
    //! | [`h!`](h) | `h!("Chapter {chap}: The Journey", 1)` | Adds a heading with `{var}` interpolation. |
    //! | [`click!`](click) | `click!("Search desk", { state.found_key = true; })` | Creates an interactive `Span` that triggers a callback when clicked. |
    //! | [`count!`](count) | `count!(\|n\| format!("Clicked {n} times"))` | Creates a dynamic counter `Span` incremented on each click. |
    //! | [`alts!`](alts) | `alts!(Cycle, ["North", "South", "East", "West"])` | Creates cycling, stopping, or shuffled text alternative spans. |
    //! | [`p!`](p) | `p!("Hello {name}, welcome back!")` | Adds a single paragraph block with `{var}` interpolation. |
    //! | [`ps!`](ps) | `ps!("Welcome {name}.", "You have {gold} gold.")` | Adds multiple separate paragraph blocks with `{var}` interpolation. |
    //! | [`text!`](text) | `text!("Status: {hp}/{max_hp} HP")` | Appends an unspaced text line with `{var}` interpolation and optional `:: "metadata"`. |
    //! | [`ts!`](ts) | `ts!("Player: {name}", "Score: {score}")` | Appends multiple unspaced text lines with `{var}` interpolation. |
    //! | [`choice!`](choice) | `choice!(("Open door", p_door), ("Turn back", p_back))` | Displays a static list of clickable choices for page navigation or actions. |
    //! | [`dchoice!`](dchoice) | `dchoice!(items.into_iter().map(...))` | Displays a dynamic choice list generated at runtime from an iterator/collection. |
    //! | [`mchoice!`](mchoice) | `mchoice!((key), ...)` | Choice menu with an explicit state key override. |
    //! | [`dparagraph!`](dparagraph) | `dparagraph!("Go to [[forest]] or [[inn\|cozy inn]]")` | Interactive paragraph with wiki-style links; returns `Some(target)` on selection (`None` initially). |
    //! | [`mparagraph!`](mparagraph) | `mparagraph!("Take [[torch]] and [[sword]]")` | Interactive paragraph with wiki-style links; tracks multiple clicked tokens and returns `Vec<bool>`. |
    //! | [`replace!`](replace) | `replace!("Chest is [[locked]].", "Unlocked!")` | Clickable paragraph that transitions into a replacement line or collapses to 0 height. |
    //! | [`NEXT!`](NEXT) | `NEXT!(next_page)` | Flow control: transition to the specified page function. |
    //! | [`EMBED!`](EMBED) | `EMBED!(sub_page)` | Flow control: evaluate sub-page with transient Game and embed View; propagate transitions. |
    //! | [`BACK!`](BACK) | `BACK!()` or `BACK!(2)` | Flow control: navigate back 1 (or `n`) steps in history. |
    //! | [`TUN!`](TUN) | `TUN!(tunnel_page)` | Flow control: enter a tunnel subroutine page. |
    //! | [`END!`](END) | `END!()` | Flow control: terminate story execution. |
    //! | [`read_key!`](read_key) | `read_key!(KEY_ID)` | Reads the stored `u64` state value for a key. |
    //! | [`set_key!`](set_key) | `set_key!(KEY_ID, val)` | Stores or updates the `u64` state value for a key. |
    //! | [`read_key_mask!`](read_key_mask) | `read_key_mask!(KEY_ID)` | Unpacks a `u64` state value into a boolean bitmask array. |

    pub use ifengine_core::elements::*;
    pub use ifengine_core::{BACK, END, NEXT, TUN};

    #[cfg(feature = "macros")]
    pub use ifengine_macros::*;

    #[cfg(feature = "macros")]
    pub use ifengine_macros::{
        dparagraph as dp, mchoice as choices, mparagraph as mp, paragraph as p, paragraphs as ps,
        text, texts as ts,
    };
}
