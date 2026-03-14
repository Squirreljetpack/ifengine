//! Utilities suitable for both internal and public use.

mod text;
pub use text::*;

mod mask;
pub use mask::*;

macro_rules! _dbg {
    ($($t:tt)*) => {
        #[cfg(debug_assertions)]
        {
            dbg!($($t)*);
        }
    };
}

pub(crate) use _dbg;
