//! Declarative macros and core types for interactive fiction elements.

use crate::view::Line;

// ----- Response Navigation Macros -----

/// Immediately return a transition-type [`Response`](crate::core::Response).
///
/// This returns `!`, exiting the current function.
///
/// - `$f:path`: Switch to the given page.
///
/// Note: Do not rely on these in closures!
#[macro_export]
macro_rules! NEXT {
    ($f:path) => {
        return $crate::core::Response::Switch($crate::core::PageHandle::new(
            stringify!($f).into(),
            $f,
        ));
    };
}

/// Immediately return a transition-type [`Response`](crate::core::Response).
///
/// This returns `!`, exiting the current function.
/// - `$n:expr`: Go back `$n` steps.
/// - No arguments: go back 1 step.
#[macro_export]
macro_rules! BACK {
    ($n:expr) => {
        return $crate::core::Response::Back($n);
    };
    () => {
        return $crate::core::Response::Back(1);
    };
}

/// Immediately return a tunnel-type [`Response`](crate::core::Response).
///
/// This returns `!`, exiting the current function.
///
/// - `$f:path`: Enter a tunnel to the specified page.
/// - No arguments: exit current tunnel.
#[macro_export]
macro_rules! TUN {
    ($f:path) => {
        return $crate::core::Response::Tunnel($crate::core::PageHandle::new(
            stringify!($f).into(),
            $f,
        ));
    };
    () => {
        return $crate::core::Response::Exit;
    };
}

/// Immediately return a [`Response::End`](crate::core::Response::End).
///
/// This returns `!`, exiting the current function.
#[macro_export]
macro_rules! END {
    () => {
        return $crate::core::Response::End;
    };
}

pub use crate::{BACK, END, NEXT, TUN};

/// The variants accepted by the `choices` / `mchoice` macro on the left-hand side.
///
/// This allows you to dynamically adapt the set of shown choices through your rust code.
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
