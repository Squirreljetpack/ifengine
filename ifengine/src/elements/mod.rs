//! Module containing macros that simplify creating interactive text elements and [responses](crate::core::Response). Provides declarative macros to create spans, links, navigation actions and responses.
//! Provides procedural macros for choices, paragraphs, and other clickable elements.
//!
//! # Additional
//! Requires the `"macros"` feature (included with default).

#[cfg(feature = "macros")]
mod macros_enabled {
    #[allow(unused)] // doc imports, don't work on proc macros
    use crate::{
        elements,
        view::{Line, Span, Object, RenderData},
        Action,
        View,
        core::{GameContext, GameTags, PageState, Response},
        run::Simulation,
        elements::ChoiceVariant
    };


    // Note: these are (counter-intuitively) exported at crate root
    // todo: lowpri: we should probably rewrite more proc macros into declarative macros if we can, only using proc macro to omit passing in the local state
    // i.e. this doesn't work
    // #[macro_export]
    // macro_rules! link_p {
    //     ($e:expr $(, $f:path)?) => {
    //         $crate::elements::p!($crate::elements::link($e $(, $f)?))
    //     };
    // }

    // --------------------------------------- Spans -----------------------------

    /// Create a [`Span`] from the given expression.
    ///
    /// # Examples
    ///
    /// ```
    /// let my_span = s!("Hello world");
    /// ```
    #[macro_export]
    macro_rules! s {
        ($e:expr) => {
            $crate::view::Span::from($e)
        };
    }

    /// Create a clickable link [`Span`].
    ///
    /// - `$e`: The text to display.
    /// - `$f`: Optional path to a page or function for the link action.
    ///
    /// # Examples
    ///
    /// ```
    /// let link1 = link!("Click me", MyPage);
    /// let link2 = link!("Just a link");
    /// ```
    #[macro_export]
    macro_rules! link {
        ($e:expr, $f:path) => {
            $crate::view::Span::from($e)
            .as_link()
            .with_action($crate::Action::Next($crate::core::PageHandle::new(
                stringify!($f).into(),
                $f,
            )))
        };
        ($e:expr) => {
            $crate::view::Span::from($e).as_link()
        };
    }

    /// Create a tunnel or exit link [`Span`].
    ///
    /// - `$e`: Display text.
    /// - `$f`: Optional target page to tunnel to.
    /// - No parameters: produces an exit link.
    #[macro_export]
    macro_rules! tun {
        ($e:expr, $f:path) => {
            $crate::view::Span::from($e)
            .as_link()
            .with_action($crate::Action::Tunnel($crate::core::PageHandle::new(
                stringify!($f).into(),
                $f,
            )))
        };
        ($e:expr) => {
            $crate::view::Span::from($e)
            .as_link()
            .with_action($crate::Action::Exit)
        };
    }

    // ----- Response ---------

    /// Immediately return a transition-type [`Response`].
    ///
    /// This returns `!`, exiting the current function.
    ///
    /// - `$f:path`: Switch to the given page.
    /// - `$n:expr`: Go back `$n` steps.
    /// - No arguments: go back 1 step.
    #[macro_export]
    macro_rules! GO {
        ($f:path) => {
            return $crate::core::Response::Switch($crate::core::PageHandle::new(
                stringify!($f).into(),
                $f,
            ));
        };
        ($n:expr) => {
            return $crate::core::Response::Back($n);
        };
        () => {
            return $crate::core::Response::Back(1);
        };
    }
    /// Immediately return a tunnel-type [`Response`].
    ///
    /// This returns `!`, exiting the current function.
    ///
    /// - `$f:path`: Enter a tunnel to the specified page.
    /// - No arguments: exit current tunnel.
    #[macro_export]
    macro_rules! ENTER {
        ($f:path) => {
            return $crate::core::Response::EnterTunnel(
                $crate::core::PageHandle::new(stringify!($f).into(), $f),
            );
        };
        () => {
            return $crate::core::Response::Exit
        };
    }

    /// Immediately return a `End` [`Response`].
    ///
    /// This returns `!`, exiting the current function.
    #[macro_export]
    macro_rules! END {
        () => {
            return $crate::core::Response::End;
        };
    }

    // ----- Re-exports for ergonomic access -----
    /// Alias for [`dparagraph`].
    pub use ifengine_macros::dparagraph as dp;

    /// Alias for [`mchoice`].
    pub use ifengine_macros::mchoice as choices;

    /// Alias for [`mparagraph`].
    pub use ifengine_macros::mparagraph as mp;

    /// Alias for [`paragraph`].
    pub use ifengine_macros::paragraph as p;

    /// Alias for [`paragraphs`].
    pub use ifengine_macros::paragraphs as ps;

    /// Alias for [`text`].
    pub use ifengine_macros::text as l;

    /// Alias for [`texts`].
    pub use ifengine_macros::texts as ls;

    pub use ifengine_macros::*;
}

/// Re-export macros when the feature is enabled.
#[cfg(feature = "macros")]
pub use macros_enabled::*;
use crate::view::Line;

/// The variants accepted by the [`choices`] macro on the left-hand side.
///
/// This allows you to dynamically adapt the set of shown choices through your rust code.
///
/// # Additional
/// You may find it convenient to `import ChoiceVariant::*`;
///
#[derive(Debug)]
pub enum ChoiceVariant {
    /// Hide after being clicked
    Once(Line),
    /// Not shown
    Hidden,
    /// Always shown
    Always(Line),
}

// ----------- BOILERPLATE ------------------------------

impl<T: Into<Line>> From<T> for ChoiceVariant {
    fn from(value: T) -> Self {
        ChoiceVariant::Once(value.into())
    }
}

impl<T: Into<Line>> From<Option<T>> for ChoiceVariant {
    fn from(value: Option<T>) -> Self {
        if let Some(value) = value {
            ChoiceVariant::Always(value.into())
        } else {
            ChoiceVariant::Hidden
        }
    }
}

impl ChoiceVariant {
    pub fn as_line(self, seen: bool) -> Option<Line> {
        match self {
            ChoiceVariant::Hidden => None,
            ChoiceVariant::Once(l) => {
                if seen {
                    None
                } else {
                    Some(l)
                }
            }
            ChoiceVariant::Always(l) => Some(l),
        }
    }
}