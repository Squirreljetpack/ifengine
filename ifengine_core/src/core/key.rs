//! Key definitions and hashing utilities for element and state tracking.

/// The key used by [`PageState`](crate::core::PageState) to track state.
pub type PageKey = u64;

/// The mask for user keys and location payloads (lower 48 bits, top 16 bits must be 0).
pub const USER_KEY_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Top bit reserved for string-hashed lookup keys.
pub const HASH_KEY_BIT: u64 = 1u64 << 63;

/// Computes a 64-bit FNV-1a hash of a string with the top bit (bit 63) set to 1.
pub const fn hash_key(s: &str) -> PageKey {
    const_fnv1a_hash::fnv1a_hash_str_64(s) | HASH_KEY_BIT
}

/// A trait for types that can be converted into a [`PageKey`].
///
/// Implemented for string types (which are hashed with bit 63 set)
/// and integer types (which are cast to `u64`).
pub trait IntoPageKey {
    fn into_page_key(self) -> PageKey;
}

impl IntoPageKey for u64 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self
    }
}

impl IntoPageKey for &u64 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self
    }
}

impl IntoPageKey for usize {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &usize {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for u32 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &u32 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for u16 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &u16 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for u8 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &u8 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for i64 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &i64 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for i32 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &i32 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for i16 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &i16 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for i8 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &i8 {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for isize {
    #[inline]
    fn into_page_key(self) -> PageKey {
        self as u64
    }
}

impl IntoPageKey for &isize {
    #[inline]
    fn into_page_key(self) -> PageKey {
        *self as u64
    }
}

impl IntoPageKey for &str {
    #[inline]
    fn into_page_key(self) -> PageKey {
        hash_key(self)
    }
}

impl IntoPageKey for &&str {
    #[inline]
    fn into_page_key(self) -> PageKey {
        hash_key(*self)
    }
}

impl IntoPageKey for String {
    #[inline]
    fn into_page_key(self) -> PageKey {
        hash_key(&self)
    }
}

impl IntoPageKey for &String {
    #[inline]
    fn into_page_key(self) -> PageKey {
        hash_key(self.as_str())
    }
}

impl<'a> IntoPageKey for std::borrow::Cow<'a, str> {
    #[inline]
    fn into_page_key(self) -> PageKey {
        hash_key(self.as_ref())
    }
}

impl<'a> IntoPageKey for &'a std::borrow::Cow<'a, str> {
    #[inline]
    fn into_page_key(self) -> PageKey {
        hash_key(self.as_ref())
    }
}
