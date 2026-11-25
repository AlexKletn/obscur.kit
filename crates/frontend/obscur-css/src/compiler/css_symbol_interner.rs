use std::collections::HashMap;

use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/CssSymbolInterner.md")]
#[derive(Debug, Clone, Default)]
pub struct CssSymbolInterner {
    lookup: HashMap<Box<str>, InternedSymbol>,
    reverse: Vec<Box<str>>,
}

impl CssSymbolInterner {
    /// Creates an empty interner.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/intern.md")]
    pub fn intern(&mut self, text: &str) -> InternedSymbol {
        if let Some(&symbol) = self.lookup.get(text) {
            return symbol;
        }
        // `from_index`, never `from_raw`: this table numbers from ZERO, and a
        // raw zero-based index is squarely inside the range the reserved
        // symbols occupy — `layout_keyword::OVERFLOW_HIDDEN` is `0x0000_1E02`,
        // so the 7682nd word a sheet interned used to come out EQUAL to it,
        // and `overflow-x: sideways` was then read as `hidden` and clipped.
        // `from_index` lifts the index into the high namespace the way `named`
        // does, which makes the two ranges disjoint by construction.
        let index = u32::try_from(self.reverse.len()).unwrap_or(u32::MAX);
        let symbol = InternedSymbol::from_index(index);
        let boxed: Box<str> = text.into();
        self.reverse.push(boxed.clone());
        self.lookup.insert(boxed, symbol);
        symbol
    }

    /// Resolves a previously interned symbol back to its text, or [`None`] if the
    /// symbol was not minted by this interner.
    #[must_use]
    pub fn resolve(&self, symbol: InternedSymbol) -> Option<&str> {
        symbol
            .index()
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| self.reverse.get(index))
            .map(|boxed| &**boxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_style::property::layout_keyword;

    #[test]
    fn interner_dedups_and_round_trips() {
        let mut interner = CssSymbolInterner::new();
        let a = interner.intern("flex");
        let b = interner.intern("flex");
        let c = interner.intern("block");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(interner.resolve(a), Some("flex"));
        assert_eq!(interner.resolve(c), Some("block"));
        assert_eq!(interner.resolve(InternedSymbol::from_raw(99)), None);
    }

    #[test]
    fn no_sheet_local_symbol_can_alias_a_reserved_layout_keyword() {
        // The two producers of `InternedSymbol` values that meet in a
        // `PropertyValue::Keyword` are this interner and `layout_keyword`'s
        // hand-assigned constants. Reserved layout keywords run from
        // `DISPLAY_BLOCK` (0x1001) to `POINTER_EVENTS_NONE` (0x2702), so a
        // sweep past 0x2702 walks every id that could ever collide — and
        // `name_of` is the reserved table itself, so this fails if EITHER side
        // moves: if the interner goes back to raw indices, or if a reserved
        // keyword is renumbered into the high namespace.
        let mut interner = CssSymbolInterner::new();
        for index in 0..0x3000u32 {
            let symbol = interner.intern(&format!("kw{index}"));
            assert_eq!(
                layout_keyword::name_of(symbol),
                None,
                "sheet-local symbol {index} aliased a reserved layout keyword"
            );
            assert_ne!(
                symbol.raw() & 0x8000_0000,
                0,
                "sheet-local symbol {index} landed in the reserved (top-bit-clear) range"
            );
        }
    }
}
