#![allow(unused_imports, ambiguous_glob_imports)]

// ONLY ONE FEATURE

#[cfg(feature = "saltwrack")]
mod saltwrack;
#[cfg(feature = "saltwrack")]
pub use crate::saltwrack::*;

#[cfg(not(feature = "saltwrack"))]
mod test_story;
#[cfg(not(feature = "saltwrack"))]
pub use crate::test_story::*;
