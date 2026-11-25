//! Interned symbol handle.

/// An opaque handle into a symbol interner. The backing table lives in the
/// subsystem that owns interning; this type is only the stable handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedSymbol(u32);

impl InternedSymbol {
    /// Wraps a raw interner index.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw interner index.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[doc = include_str!("../../docs/api/key/interned_symbol.doc/named.md")]
    #[must_use]
    pub const fn named(name: &str) -> Self {
        let bytes = name.as_bytes();
        let mut hash: u32 = 0x811c_9dc5;
        let mut i = 0;
        while i < bytes.len() {
            hash ^= bytes[i] as u32;
            hash = hash.wrapping_mul(0x0100_0193);
            i += 1;
        }
        Self(hash | 0x8000_0000)
    }

    #[doc = include_str!("../../docs/api/key/interned_symbol.doc/from_index.md")]
    #[must_use]
    pub const fn from_index(index: u32) -> Self {
        Self(index | 0x8000_0000)
    }

    #[doc = include_str!("../../docs/api/key/interned_symbol.doc/index.md")]
    #[must_use]
    pub const fn index(self) -> Option<u32> {
        if self.0 & 0x8000_0000 == 0 {
            None
        } else {
            Some(self.0 & 0x7FFF_FFFF)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InternedSymbol;

    #[test]
    fn named_is_deterministic() {
        assert_eq!(InternedSymbol::named("save"), InternedSymbol::named("save"));
    }

    #[test]
    fn distinct_names_give_distinct_symbols() {
        assert_ne!(InternedSymbol::named("save"), InternedSymbol::named("open"));
    }

    #[test]
    fn named_sets_the_high_bit_so_it_never_aliases_a_reserved_symbol() {
        // Reserved low-range symbols (`from_raw(0x0000_00xx)`) have the top bit
        // clear; a named symbol always has it set, so the two ranges are disjoint.
        for name in ["increment", "decrement", "goParent", "", "a"] {
            assert_ne!(InternedSymbol::named(name).raw() & 0x8000_0000, 0, "{name}");
        }
    }

    #[test]
    fn from_index_sets_the_high_bit_so_it_never_aliases_a_reserved_symbol() {
        // A table-backed interner numbers from ZERO, so its raw indices land
        // inside the reserved low range — `from_raw(0x0000_1E02)` is
        // `layout_keyword::OVERFLOW_HIDDEN`, and the 7682nd word a stylesheet
        // interned used to come out equal to it. Lifting the index into the
        // high namespace is what makes that unreachable.
        for index in [0, 1, 2, 0x1E02, 0x7FFF_FFFF] {
            assert_ne!(
                InternedSymbol::from_index(index).raw() & 0x8000_0000,
                0,
                "{index}"
            );
            assert_ne!(
                InternedSymbol::from_index(index),
                InternedSymbol::from_raw(index)
            );
        }
    }

    #[test]
    fn from_index_round_trips_through_index() {
        for index in [0, 1, 0x1E02, 0x7FFF_FFFF] {
            assert_eq!(InternedSymbol::from_index(index).index(), Some(index));
        }
    }

    #[test]
    fn a_reserved_symbol_carries_no_index() {
        for raw in [0, 1, 0x1E02, 0x7FFF_FFFF] {
            assert_eq!(InternedSymbol::from_raw(raw).index(), None, "{raw}");
        }
    }

    #[test]
    fn named_is_usable_in_const_context() {
        const SAVE: InternedSymbol = InternedSymbol::named("save");
        assert_eq!(SAVE, InternedSymbol::named("save"));
    }
}
