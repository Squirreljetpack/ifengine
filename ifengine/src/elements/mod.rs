/// Module containing macros that simplify creating interactive text elements and responses.
///
/// Enabled only if the `"macros"` feature is active. Provides declarative macros to create
/// spans, links, navigation actions, and tunnel/response handling conveniently.
#[cfg(feature = "macros")]
mod macros_enabled {
    #[allow(unused)] // doc imports
    use crate::view::Span;
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

    /// Creates a [`Span`] from the given expression.
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

    /// Creates a clickable link [`Span`].
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

    /// Creates a link [`Span`] that navigates backward.
    ///
    /// - `$e`: Display text.
    /// - `$n`: Optional number of steps to go back (defaults to 1).
    #[macro_export]
    macro_rules! back {
        ($e:expr, $n:expr) => {
            $crate::view::Span::from($e)
            .as_link()
            .with_action($crate::Action::Back($n))
        };
        ($e:expr) => {
            $crate::view::Span::from($e)
            .as_link()
            .with_action($crate::Action::Back(1))
        };
    }

    /// Creates a tunnel or exit link [`Span`].
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

    /// Immediately return a transition-type `Response`.
    /// This returns !, exiting the current function.
    ///
    /// - `$f:path`: Switch to the given page.
    /// - `$n:expr`: Go back `$n` steps.
    /// - No arguments: go back 1 step.
    #[macro_export]
    macro_rules! switch {
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
    /// Immediately return a transition-type `Response`.
    /// This returns !, exiting the current function.
    ///
    /// - `$f:path`: Enter a tunnel to the specified page.
    /// - No arguments: exit current tunnel.
    #[macro_export]
    macro_rules! enter {
        ($f:path) => {
            return $crate::core::Response::EnterTunnel(
                $crate::core::PageHandle::new(stringify!($f).into(), $f),
            );
        };
        () => {
            return $crate::core::Response::Exit
        };
    }

    /// Returns an `End` response.
    #[macro_export]
    macro_rules! end {
        () => {
            return $crate::core::Response::End;
        };
    }

    // ----- Re-exports for ergonomic access -----
    pub use ifengine_macros::dparagraph as dp;
    pub use ifengine_macros::mchoice as choices;
    pub use ifengine_macros::mparagraph as mp;
    pub use ifengine_macros::paragraph as p;
    pub use ifengine_macros::paragraphs as ps;
    pub use ifengine_macros::*;
}

/// Re-export macros when the feature is enabled.
#[cfg(feature = "macros")]
pub use macros_enabled::*;
use crate::view::Line;

/// The variants accepted by the [`choices`] macro on the LHS.
/// This allows you to dynamically adapt the set of shown choices through your rust code.
/// You may find it convenient to import ChoiceVariant::*;
///
/// - `Once(Line)`: Hide after being clicked.
/// - `Hidden`: Not shown.
/// - `Always(Line)`: Always shown.

#[derive(Debug)]
pub enum ChoiceVariant {
    Once(Line),
    Hidden,
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