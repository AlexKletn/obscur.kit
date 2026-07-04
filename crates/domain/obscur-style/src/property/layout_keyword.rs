//! Reserved interned-symbol handles for layout keyword values.
//!
//! A handful of layout properties (`display`, `flex-direction`,
//! `justify-content`, `align-items`) take their values from a small, *closed*
//! set of CSS keywords. Those keywords are carried through the style system as
//! [`PropertyValue::Keyword`](crate::property::PropertyValue::Keyword) values —
//! i.e. as [`InternedSymbol`] identities, never as strings.
//!
//! The problem is that the CSS compiler and the runtime layout bridge live in
//! different crates and would ordinarily intern the same keyword text into
//! *different* symbol tables, so their `InternedSymbol`s could never be compared
//! for identity. To make the keyword handles agree across crates without sharing
//! an interner, this module reserves a fixed block of symbol values — one per
//! keyword — that both sides use directly. The CSS compiler resolves a keyword
//! *string* to its reserved symbol via the `*_from_name` helpers here; the
//! layout bridge compares a computed `Keyword(sym)` against the `pub const`s
//! below. These constants are the canonical cross-crate handles for layout
//! keyword values.
//!
//! The reserved values occupy the `0x0000_1xxx` namespace. This is distinct
//! from `obscur-view`'s element-attribute reservations (`CLASS`/`ID`/`TYPE` in
//! `0x0000_000x`) and from the component/primitive prop namespaces (e.g.
//! `0x00C1_xxxx`), so a reserved layout keyword can never collide with either.

use obscur_identity::InternedSymbol;

// ---- `display` keyword values ----

/// The `display: block` keyword.
pub const DISPLAY_BLOCK: InternedSymbol = InternedSymbol::from_raw(0x0000_1001);
/// The `display: flex` keyword.
pub const DISPLAY_FLEX: InternedSymbol = InternedSymbol::from_raw(0x0000_1002);
/// The `display: grid` keyword.
pub const DISPLAY_GRID: InternedSymbol = InternedSymbol::from_raw(0x0000_1003);
/// The `display: none` keyword.
pub const DISPLAY_NONE: InternedSymbol = InternedSymbol::from_raw(0x0000_1004);

// ---- `flex-direction` keyword values ----

/// The `flex-direction: row` keyword.
pub const FLEX_ROW: InternedSymbol = InternedSymbol::from_raw(0x0000_1101);
/// The `flex-direction: row-reverse` keyword.
pub const FLEX_ROW_REVERSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1102);
/// The `flex-direction: column` keyword.
pub const FLEX_COLUMN: InternedSymbol = InternedSymbol::from_raw(0x0000_1103);
/// The `flex-direction: column-reverse` keyword.
pub const FLEX_COLUMN_REVERSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1104);

// ---- `justify-content` keyword values ----

/// The `justify-content: flex-start` keyword.
pub const JUSTIFY_START: InternedSymbol = InternedSymbol::from_raw(0x0000_1201);
/// The `justify-content: center` keyword.
pub const JUSTIFY_CENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1202);
/// The `justify-content: flex-end` keyword.
pub const JUSTIFY_END: InternedSymbol = InternedSymbol::from_raw(0x0000_1203);
/// The `justify-content: space-between` keyword.
pub const JUSTIFY_SPACE_BETWEEN: InternedSymbol = InternedSymbol::from_raw(0x0000_1204);
/// The `justify-content: space-around` keyword.
pub const JUSTIFY_SPACE_AROUND: InternedSymbol = InternedSymbol::from_raw(0x0000_1205);

// ---- `align-items` keyword values ----

/// The `align-items: flex-start` keyword.
pub const ALIGN_START: InternedSymbol = InternedSymbol::from_raw(0x0000_1301);
/// The `align-items: center` keyword.
pub const ALIGN_CENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1302);
/// The `align-items: flex-end` keyword.
pub const ALIGN_END: InternedSymbol = InternedSymbol::from_raw(0x0000_1303);
/// The `align-items: stretch` keyword.
pub const ALIGN_STRETCH: InternedSymbol = InternedSymbol::from_raw(0x0000_1304);

/// Maps a `display` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching, so callers may pass authored
/// casing directly. Returns [`None`] for any keyword outside the closed set
/// modelled here.
#[must_use]
pub fn display_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "block" => Some(DISPLAY_BLOCK),
        "flex" => Some(DISPLAY_FLEX),
        "grid" => Some(DISPLAY_GRID),
        "none" => Some(DISPLAY_NONE),
        _ => None,
    }
}

/// Maps a `flex-direction` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Returns [`None`] for any
/// keyword outside the closed set modelled here.
#[must_use]
pub fn flex_direction_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "row" => Some(FLEX_ROW),
        "row-reverse" => Some(FLEX_ROW_REVERSE),
        "column" => Some(FLEX_COLUMN),
        "column-reverse" => Some(FLEX_COLUMN_REVERSE),
        _ => None,
    }
}

/// Maps a `justify-content` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Both the modern (`start` /
/// `end`) and the flexbox (`flex-start` / `flex-end`) spellings resolve to the
/// same reserved handle. Returns [`None`] for any keyword outside the closed set
/// modelled here.
#[must_use]
pub fn justify_content_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "flex-start" | "start" => Some(JUSTIFY_START),
        "center" => Some(JUSTIFY_CENTER),
        "flex-end" | "end" => Some(JUSTIFY_END),
        "space-between" => Some(JUSTIFY_SPACE_BETWEEN),
        "space-around" => Some(JUSTIFY_SPACE_AROUND),
        _ => None,
    }
}

/// Maps an `align-items` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Both the modern (`start` /
/// `end`) and the flexbox (`flex-start` / `flex-end`) spellings resolve to the
/// same reserved handle. Returns [`None`] for any keyword outside the closed set
/// modelled here.
#[must_use]
pub fn align_items_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "flex-start" | "start" => Some(ALIGN_START),
        "center" => Some(ALIGN_CENTER),
        "flex-end" | "end" => Some(ALIGN_END),
        "stretch" => Some(ALIGN_STRETCH),
        _ => None,
    }
}

/// Returns the canonical CSS keyword string for a reserved layout symbol.
///
/// This is the reverse of the `*_from_name` helpers, intended for diagnostics
/// and `DevTools`. It returns the canonical spelling (the flexbox spelling for
/// the `justify-content` / `align-items` start/end values). Returns [`None`] for
/// any symbol that is not one of the reserved layout keyword handles.
// The `justify-content` and `align-items` start/center/end handles share their
// canonical spelling but are kept as distinct arms, grouped by property, so the
// reverse mapping reads one property at a time.
#[allow(clippy::match_same_arms)]
#[must_use]
pub fn name_of(symbol: InternedSymbol) -> Option<&'static str> {
    match symbol {
        DISPLAY_BLOCK => Some("block"),
        DISPLAY_FLEX => Some("flex"),
        DISPLAY_GRID => Some("grid"),
        DISPLAY_NONE => Some("none"),
        FLEX_ROW => Some("row"),
        FLEX_ROW_REVERSE => Some("row-reverse"),
        FLEX_COLUMN => Some("column"),
        FLEX_COLUMN_REVERSE => Some("column-reverse"),
        JUSTIFY_START => Some("flex-start"),
        JUSTIFY_CENTER => Some("center"),
        JUSTIFY_END => Some("flex-end"),
        JUSTIFY_SPACE_BETWEEN => Some("space-between"),
        JUSTIFY_SPACE_AROUND => Some("space-around"),
        ALIGN_START => Some("flex-start"),
        ALIGN_CENTER => Some("center"),
        ALIGN_END => Some("flex-end"),
        ALIGN_STRETCH => Some("stretch"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every reserved layout keyword handle, for collision checks.
    const ALL: &[InternedSymbol] = &[
        DISPLAY_BLOCK,
        DISPLAY_FLEX,
        DISPLAY_GRID,
        DISPLAY_NONE,
        FLEX_ROW,
        FLEX_ROW_REVERSE,
        FLEX_COLUMN,
        FLEX_COLUMN_REVERSE,
        JUSTIFY_START,
        JUSTIFY_CENTER,
        JUSTIFY_END,
        JUSTIFY_SPACE_BETWEEN,
        JUSTIFY_SPACE_AROUND,
        ALIGN_START,
        ALIGN_CENTER,
        ALIGN_END,
        ALIGN_STRETCH,
    ];

    #[test]
    fn display_maps_known_keywords() {
        assert_eq!(display_from_name("block"), Some(DISPLAY_BLOCK));
        assert_eq!(display_from_name("flex"), Some(DISPLAY_FLEX));
        assert_eq!(display_from_name("grid"), Some(DISPLAY_GRID));
        assert_eq!(display_from_name("none"), Some(DISPLAY_NONE));
        // Case-insensitive.
        assert_eq!(display_from_name("FLEX"), Some(DISPLAY_FLEX));
        // Unknown.
        assert_eq!(display_from_name("inline"), None);
        assert_eq!(display_from_name(""), None);
    }

    #[test]
    fn flex_direction_maps_known_keywords() {
        assert_eq!(flex_direction_from_name("row"), Some(FLEX_ROW));
        assert_eq!(
            flex_direction_from_name("row-reverse"),
            Some(FLEX_ROW_REVERSE)
        );
        assert_eq!(flex_direction_from_name("column"), Some(FLEX_COLUMN));
        assert_eq!(
            flex_direction_from_name("Column-Reverse"),
            Some(FLEX_COLUMN_REVERSE)
        );
        assert_eq!(flex_direction_from_name("diagonal"), None);
    }

    #[test]
    fn justify_content_maps_known_keywords() {
        assert_eq!(justify_content_from_name("flex-start"), Some(JUSTIFY_START));
        assert_eq!(justify_content_from_name("start"), Some(JUSTIFY_START));
        assert_eq!(justify_content_from_name("center"), Some(JUSTIFY_CENTER));
        assert_eq!(justify_content_from_name("flex-end"), Some(JUSTIFY_END));
        assert_eq!(justify_content_from_name("end"), Some(JUSTIFY_END));
        assert_eq!(
            justify_content_from_name("space-between"),
            Some(JUSTIFY_SPACE_BETWEEN)
        );
        assert_eq!(
            justify_content_from_name("space-around"),
            Some(JUSTIFY_SPACE_AROUND)
        );
        assert_eq!(justify_content_from_name("space-evenly"), None);
    }

    #[test]
    fn align_items_maps_known_keywords() {
        assert_eq!(align_items_from_name("flex-start"), Some(ALIGN_START));
        assert_eq!(align_items_from_name("start"), Some(ALIGN_START));
        assert_eq!(align_items_from_name("center"), Some(ALIGN_CENTER));
        assert_eq!(align_items_from_name("flex-end"), Some(ALIGN_END));
        assert_eq!(align_items_from_name("stretch"), Some(ALIGN_STRETCH));
        assert_eq!(align_items_from_name("baseline"), None);
    }

    #[test]
    fn name_of_round_trips_display_and_flex() {
        assert_eq!(name_of(DISPLAY_FLEX), Some("flex"));
        assert_eq!(name_of(FLEX_COLUMN_REVERSE), Some("column-reverse"));
        assert_eq!(name_of(ALIGN_STRETCH), Some("stretch"));
        // A symbol outside the reserved block resolves to nothing.
        assert_eq!(name_of(InternedSymbol::from_raw(0x0000_9999)), None);
    }

    #[test]
    fn reserved_handles_are_all_distinct() {
        let mut seen = std::collections::HashSet::new();
        for &sym in ALL {
            assert!(seen.insert(sym), "duplicate reserved symbol: {}", sym.raw());
        }
    }

    #[test]
    fn reserved_handles_live_above_the_view_attribute_range() {
        // `obscur-view` reserves CLASS/ID/TYPE in `0x0000_000x`; layout keyword
        // handles must sit above that range so the two namespaces never collide.
        for &sym in ALL {
            assert!(
                sym.raw() >= 0x0000_1000,
                "reserved symbol {:#x} collides with the view attribute range",
                sym.raw()
            );
        }
    }
}
