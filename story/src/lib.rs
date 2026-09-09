#![allow(unused_imports, ambiguous_glob_imports)]

// ONLY ONE FEATURE

#[cfg(any(feature = "test", not(feature = "saltwrack")))]
mod test_story;
#[cfg(any(feature = "test", not(feature = "saltwrack")))]
pub use crate::test_story::*;

#[cfg(all(feature = "saltwrack", not(feature = "test")))]
mod saltwrack;
#[cfg(all(feature = "saltwrack", not(feature = "test")))]
pub use crate::saltwrack::*;
