#[cfg(feature = "macros")]
mod macros_enabled {

    // --------------------------------------- Spans -----------------------------

    // todo: take styles, also custom variants
    #[macro_export]
    macro_rules! s {
        ($e:expr) => {
            $crate::view::Span::from($e)
        };
    }

    // Note: This is (counter-intuitively) exported at crate root
    // todo: lowpri: we should probably rewrite more proc macros into declarative macros if we can, only using proc macro to omit passing in the local state
    // i.e. this doesn't work
    // #[macro_export]
    // macro_rules! link_p {
    //     ($e:expr $(, $f:path)?) => {
    //         $crate::elements::p!($crate::elements::link($e $(, $f)?))
    //     };
    // }

    /// todo: docs
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

    #[macro_export]
    macro_rules! enter {
        ($f:path) => {
            return $crate::core::Response::EnterTunnel(
                //
                $crate::core::PageHandle::new(stringify!($f).into(), $f),
            );
        };
        () => {
            return $crate::core::Response::Exit
        };
    }

    #[macro_export]
    macro_rules! end {
        () => {
            return $crate::core::Response::End;
        };
    }

    pub use ifengine_macros::dparagraph as dp;
    pub use ifengine_macros::mchoice as choices;
    pub use ifengine_macros::mparagraph as mp;
    pub use ifengine_macros::paragraph as p;
    pub use ifengine_macros::paragraphs as ps;
    pub use ifengine_macros::*;
}

#[cfg(feature = "macros")]
pub use macros_enabled::*;

use crate::view::Line;

/// Recommended to import ChoiceVariant::* for ergonomics;
pub enum ChoiceVariant {
    Once(Line),
    Hidden,
    Always(Line),
}

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
