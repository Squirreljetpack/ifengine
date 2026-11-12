//! Key definitions and hashing utilities for element and state tracking.

/// The key used by [`PageState`](crate::core::PageState) to track state.
pub type PageKey = u64;

/// The mask for user keys and location payloads (lower 48 bits, top 16 bits must be 0).
pub const USER_KEY_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Bit position of the dirty marker within a state value.
pub const DIRTY_BIT_POS: u8 = 63;

/// Dirty marker (bit 63) set in a state value to signal a pending interaction.
pub const DIRTY_BIT: u64 = 1u64 << DIRTY_BIT_POS;

/// Number of low bits in a link value that hold the click count.
pub const LINK_COUNT_BITS: u32 = 16;

/// Mask covering the click-count field (bits 0..=15) of a link value.
pub const LINK_COUNT_MASK: u64 = (1 << LINK_COUNT_BITS) - 1;

/// Bit position where the location payload starts inside a link value.
pub const LINK_LOC_SHIFT: u32 = LINK_COUNT_BITS;

/// Mask covering the location field (bits 16..=62) of a link value.
///
/// The 48-bit location payload window loses its top bit to the dirty marker.
pub const LINK_LOC_MASK: u64 = (USER_KEY_MASK >> 1) << LINK_LOC_SHIFT;

/// Location bits stored in a link value owned by the paragraph `key`.
pub const fn link_loc_bits(key: PageKey) -> u64 {
    (key << LINK_LOC_SHIFT) & LINK_LOC_MASK
}

/// Owner bits (dirty marker plus location payload) stored in a link value owned by `key`.
pub const fn link_owner(key: PageKey) -> u64 {
    DIRTY_BIT | link_loc_bits(key)
}

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
        hash_key(self)
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
