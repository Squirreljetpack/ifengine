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
    ($f:path) => {{
        return $crate::core::Response::Switch($crate::core::PageHandle::new(
            stringify!($f).into(),
            $f,
        ));
    }};
}

/// Immediately return a transition-type [`Response`](crate::core::Response).
///
/// This returns `!`, exiting the current function.
/// - `$n:expr`: Go back `$n` steps.
/// - No arguments: go back 1 step.
#[macro_export]
macro_rules! BACK {
    ($n:expr) => {{
        return $crate::core::Response::Back($n);
    }};
    () => {{
        return $crate::core::Response::Back(1);
    }};
}

/// Immediately return a tunnel-type [`Response`](crate::core::Response).
///
/// This returns `!`, exiting the current function.
///
/// - `$f:path`: Enter a tunnel to the specified page.
/// - No arguments: exit current tunnel.
#[macro_export]
macro_rules! TUN {
    ($f:path) => {{
        return $crate::core::Response::Tunnel($crate::core::PageHandle::new(
            stringify!($f).into(),
            $f,
        ));
    }};
    () => {{
        return $crate::core::Response::Exit;
    }};
}

/// Immediately return a [`Response::End`](crate::core::Response::End).
///
/// This returns `!`, exiting the current function.
#[macro_export]
macro_rules! END {
    () => {{
        return $crate::core::Response::End;
    }};
}

pub use crate::core::key::IntoPageKey;
pub use crate::{BACK, END, NEXT, TUN};

/// The variants accepted by the `choices` / `mchoice` macro on the left-hand side.
///
/// This allows you to dynamically adapt the set of shown choices through your rust code.
#[derive(Debug)]
pub enum ChoiceVariant {
    /// Hide after being clicked
    Once(Line),
    /// Not shown
    Hidden(Line),
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
            ChoiceVariant::Hidden(Line::default())
        }
    }
}

impl ChoiceVariant {
    pub fn as_line(self, seen: bool) -> Option<Line> {
        match self {
            ChoiceVariant::Hidden(_) => None,
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

/// Interaction data returned by paragraph-style element macros such as
/// [`paragraph!`](crate::elements::paragraph).
///
/// `clicked` holds the target of the link clicked during the current render, `bits` is the
/// raw value stored under the element's key (a bitmask indexed by the position of each
/// interactive element), and `key` is the key the element was registered under.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PData {
    /// Target of the element clicked during this render, if any.
    pub clicked: Option<String>,
    /// Raw value stored under [`key`](Self::key), interpreted as a bitmask of tracked elements.
    pub bits: u64,
    /// Page-state key the element was registered under.
    pub key: u64,
}

impl PData {
    /// The clicked target, or `""` if nothing was clicked during this render.
    pub fn as_str(&self) -> &str {
        self.clicked.as_deref().unwrap_or("")
    }

    /// Page-state key the element was registered under.
    pub const fn key(&self) -> u64 {
        self.key
    }

    /// Consumes the data, returning the clicked target or `""` if nothing was clicked.
    pub fn unwrap_or_default(self) -> String {
        self.clicked.unwrap_or_default()
    }

    /// Expands [`bits`](Self::bits) into a boolean mask.
    ///
    /// The length is `max(highest set bit + 1, tracked)`, so it covers every tracked element
    /// without dropping any set bit. `tracked` is the number of elements the element macro
    /// tracked during this render.
    pub fn mask(&self, tracked: usize) -> Vec<bool> {
        let len = (64 - self.bits.leading_zeros() as usize).max(tracked);
        (0..len)
            .map(|i| i < 64 && (self.bits >> i) & 1 == 1)
            .collect()
    }
}

impl std::ops::Deref for PData {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.clicked
    }
}

impl std::ops::DerefMut for PData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.clicked
    }
}
