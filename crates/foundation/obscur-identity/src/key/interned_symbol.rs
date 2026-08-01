#![doc = include_str!("interned_symbol.doc/_module.md")]

#[doc = include_str!("interned_symbol.doc/InternedSymbol.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedSymbol(u32);

impl InternedSymbol {
    #[doc = include_str!("interned_symbol.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[doc = include_str!("interned_symbol.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[doc = include_str!("interned_symbol.doc/named.md")]
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
    fn named_is_usable_in_const_context() {
        const SAVE: InternedSymbol = InternedSymbol::named("save");
        assert_eq!(SAVE, InternedSymbol::named("save"));
    }
}
