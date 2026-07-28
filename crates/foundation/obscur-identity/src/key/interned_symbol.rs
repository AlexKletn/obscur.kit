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

    /// A stable symbol derived from a **name** rather than a hand-assigned index:
    /// the FNV-1a 32-bit hash of `name`, lifted into the high namespace (top bit
    /// set) so it never collides with a low-range reserved symbol.
    ///
    /// This is what lets an author name a command or a list instead of inventing a
    /// unique integer for it: `App::on_command(InternedSymbol::named("increment"),
    /// …)` in place of `InternedSymbol::from_raw(0x1002)`. The scheme is deliberately
    /// **identical** to the frontend's `fallback_symbol`, so a handler registered
    /// under `named("save")` is reached by an `on:click="save"` written in `.okui`
    /// markup — the two meet on the name without either side knowing about the other.
    /// A drift guard in `obscur-ui-ir` pins the two implementations equal.
    ///
    /// `const`, so it can name a `const` symbol handle at the definition site.
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
