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
/// The `justify-content` / `align-content: space-evenly` keyword.
pub const JUSTIFY_SPACE_EVENLY: InternedSymbol = InternedSymbol::from_raw(0x0000_1206);

// ---- `align-items` keyword values ----

/// The `align-items: flex-start` keyword.
pub const ALIGN_START: InternedSymbol = InternedSymbol::from_raw(0x0000_1301);
/// The `align-items: center` keyword.
pub const ALIGN_CENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1302);
/// The `align-items: flex-end` keyword.
pub const ALIGN_END: InternedSymbol = InternedSymbol::from_raw(0x0000_1303);
/// The `align-items: stretch` keyword.
pub const ALIGN_STRETCH: InternedSymbol = InternedSymbol::from_raw(0x0000_1304);
/// The `justify-self` / `align-self: auto` keyword (inherit the container's
/// items default). Only the *-self properties accept it; `justify-items` /
/// `align-items` do not.
pub const ALIGN_AUTO: InternedSymbol = InternedSymbol::from_raw(0x0000_1305);

// ---- `direction` keyword values ----

/// The `direction: ltr` (left-to-right) keyword.
pub const DIRECTION_LTR: InternedSymbol = InternedSymbol::from_raw(0x0000_1401);
/// The `direction: rtl` (right-to-left) keyword.
pub const DIRECTION_RTL: InternedSymbol = InternedSymbol::from_raw(0x0000_1402);

// ---- `font-style` keyword values ----

/// The `font-style: normal` keyword.
pub const FONT_STYLE_NORMAL: InternedSymbol = InternedSymbol::from_raw(0x0000_1501);
/// The `font-style: italic` keyword.
pub const FONT_STYLE_ITALIC: InternedSymbol = InternedSymbol::from_raw(0x0000_1502);
/// The `font-style: oblique` keyword.
pub const FONT_STYLE_OBLIQUE: InternedSymbol = InternedSymbol::from_raw(0x0000_1503);

// ---- `text-align` keyword values ----

/// The `text-align: left` keyword.
pub const TEXT_ALIGN_LEFT: InternedSymbol = InternedSymbol::from_raw(0x0000_1601);
/// The `text-align: right` keyword.
pub const TEXT_ALIGN_RIGHT: InternedSymbol = InternedSymbol::from_raw(0x0000_1602);
/// The `text-align: center` keyword.
pub const TEXT_ALIGN_CENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1603);
/// The `text-align: justify` keyword.
pub const TEXT_ALIGN_JUSTIFY: InternedSymbol = InternedSymbol::from_raw(0x0000_1604);
/// The `text-align: start` keyword.
pub const TEXT_ALIGN_START: InternedSymbol = InternedSymbol::from_raw(0x0000_1605);
/// The `text-align: end` keyword.
pub const TEXT_ALIGN_END: InternedSymbol = InternedSymbol::from_raw(0x0000_1606);

// ---- `text-overflow` keyword values ----

/// The `text-overflow: clip` keyword.
pub const TEXT_OVERFLOW_CLIP: InternedSymbol = InternedSymbol::from_raw(0x0000_1701);
/// The `text-overflow: ellipsis` keyword.
pub const TEXT_OVERFLOW_ELLIPSIS: InternedSymbol = InternedSymbol::from_raw(0x0000_1702);

// ---- relative `font-weight` keyword values ----

/// The `font-weight: bolder` keyword. Unlike the absolute weights (which lower
/// to numbers), this is carried as a reserved handle so the computed-value step
/// can resolve it *relative* to the parent's computed weight (§4.14).
pub const FONT_WEIGHT_BOLDER: InternedSymbol = InternedSymbol::from_raw(0x0000_1801);
/// The `font-weight: lighter` keyword, resolved relative to the parent's
/// computed weight (§4.14).
pub const FONT_WEIGHT_LIGHTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1802);

// ---- `grid-auto-flow` keyword values ----
//
// `grid-auto-flow` is a multi-keyword property: an axis (`row` / `column`) and
// an optional `dense` packing keyword combine (`row dense`, `column dense`).
// Rather than carry two symbols, each authored combination folds to ONE reserved
// handle, so the runtime reads a single keyword identity.

/// The `grid-auto-flow: row` value (row-major, sparse). The default.
pub const GRID_AUTO_FLOW_ROW: InternedSymbol = InternedSymbol::from_raw(0x0000_1901);
/// The `grid-auto-flow: column` value (column-major, sparse).
pub const GRID_AUTO_FLOW_COLUMN: InternedSymbol = InternedSymbol::from_raw(0x0000_1902);
/// The `grid-auto-flow: row dense` (or bare `dense`) value (row-major, dense).
pub const GRID_AUTO_FLOW_ROW_DENSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1903);
/// The `grid-auto-flow: column dense` value (column-major, dense).
pub const GRID_AUTO_FLOW_COLUMN_DENSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1904);

// ---- `position` keyword values ----

/// The `position: static` keyword (the default; laid out in normal flow).
pub const POSITION_STATIC: InternedSymbol = InternedSymbol::from_raw(0x0000_1A01);
/// The `position: relative` keyword (in flow, then offset by its insets).
pub const POSITION_RELATIVE: InternedSymbol = InternedSymbol::from_raw(0x0000_1A02);
/// The `position: absolute` keyword. Out-of-flow; treated as `static` for layout
/// until out-of-flow positioning lands.
pub const POSITION_ABSOLUTE: InternedSymbol = InternedSymbol::from_raw(0x0000_1A03);
/// The `position: fixed` keyword. Out-of-flow; treated as `static` for layout
/// until out-of-flow positioning lands.
pub const POSITION_FIXED: InternedSymbol = InternedSymbol::from_raw(0x0000_1A04);
/// The `position: sticky` keyword (in flow like `relative`, but its used position
/// is scroll-clamped to the scrollport edge within its containing block).
pub const POSITION_STICKY: InternedSymbol = InternedSymbol::from_raw(0x0000_1A05);

// ---- `z-index` keyword values ----

/// The `z-index: auto` keyword (the default; the element does not establish its
/// own stacking level and stacks with its parent). The `<integer>` form of
/// `z-index` lowers to a [`PropertyValue::Number`](crate::property::PropertyValue::Number)
/// instead, so this reserved handle carries only the `auto` case across crates.
pub const Z_INDEX_AUTO: InternedSymbol = InternedSymbol::from_raw(0x0000_1B01);

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

/// Maps a `position` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Returns [`None`] for any
/// keyword outside the closed `static` / `relative` / `absolute` / `fixed` /
/// `sticky` set.
#[must_use]
pub fn position_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "static" => Some(POSITION_STATIC),
        "relative" => Some(POSITION_RELATIVE),
        "absolute" => Some(POSITION_ABSOLUTE),
        "fixed" => Some(POSITION_FIXED),
        "sticky" => Some(POSITION_STICKY),
        _ => None,
    }
}

/// Maps a `z-index` keyword string to its reserved [`InternedSymbol`].
///
/// The only keyword `z-index` accepts is `auto` (mapped to [`Z_INDEX_AUTO`]); the
/// `<integer>` form lowers to a number, not a keyword. The input is
/// ASCII-lowercased before matching. Returns [`None`] for any other keyword.
#[must_use]
pub fn z_index_from_name(name: &str) -> Option<InternedSymbol> {
    if name.eq_ignore_ascii_case("auto") {
        Some(Z_INDEX_AUTO)
    } else {
        None
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
        "space-evenly" => Some(JUSTIFY_SPACE_EVENLY),
        _ => None,
    }
}

/// Maps an `align-content` keyword string to its reserved [`InternedSymbol`].
///
/// `align-content` (the grid container's block-axis track distribution) shares
/// the exact `justify-content` value grammar (`start` / `end` / `center` /
/// `space-between` / `space-around` / `space-evenly`, plus the flexbox
/// `flex-start` / `flex-end` spellings), so this simply delegates to
/// [`justify_content_from_name`]. Returns [`None`] for any keyword outside that
/// closed set.
#[must_use]
pub fn align_content_from_name(name: &str) -> Option<InternedSymbol> {
    justify_content_from_name(name)
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

/// Maps a `justify-self` / `align-self` keyword string to its reserved
/// [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. This accepts everything
/// [`align_items_from_name`] does (`start` / `end` / `center` / `stretch`, plus
/// the flexbox `flex-start` / `flex-end` spellings) and additionally `auto`
/// (mapped to [`ALIGN_AUTO`]), the per-item value that inherits the container's
/// items default. Returns [`None`] for any keyword outside that set.
#[must_use]
pub fn align_self_from_name(name: &str) -> Option<InternedSymbol> {
    if name.eq_ignore_ascii_case("auto") {
        return Some(ALIGN_AUTO);
    }
    align_items_from_name(name)
}

/// Maps a `direction` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Returns [`None`] for any
/// keyword outside the closed `ltr` / `rtl` set.
#[must_use]
pub fn direction_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "ltr" => Some(DIRECTION_LTR),
        "rtl" => Some(DIRECTION_RTL),
        _ => None,
    }
}

/// Maps a `font-style` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Returns [`None`] for any
/// keyword outside the closed `normal` / `italic` / `oblique` set.
#[must_use]
pub fn font_style_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "normal" => Some(FONT_STYLE_NORMAL),
        "italic" => Some(FONT_STYLE_ITALIC),
        "oblique" => Some(FONT_STYLE_OBLIQUE),
        _ => None,
    }
}

/// Maps a `text-align` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Returns [`None`] for any
/// keyword outside the closed `left` / `right` / `center` / `justify` / `start` /
/// `end` set.
#[must_use]
pub fn text_align_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "left" => Some(TEXT_ALIGN_LEFT),
        "right" => Some(TEXT_ALIGN_RIGHT),
        "center" => Some(TEXT_ALIGN_CENTER),
        "justify" => Some(TEXT_ALIGN_JUSTIFY),
        "start" => Some(TEXT_ALIGN_START),
        "end" => Some(TEXT_ALIGN_END),
        _ => None,
    }
}

/// Maps a `text-overflow` keyword string to its reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Returns [`None`] for any
/// keyword outside the closed `clip` / `ellipsis` set.
#[must_use]
pub fn text_overflow_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "clip" => Some(TEXT_OVERFLOW_CLIP),
        "ellipsis" => Some(TEXT_OVERFLOW_ELLIPSIS),
        _ => None,
    }
}

/// Maps a relative `font-weight` keyword string (`bolder` / `lighter`) to its
/// reserved [`InternedSymbol`].
///
/// The input is ASCII-lowercased before matching. Returns [`None`] for any other
/// keyword — including the absolute `normal` / `bold` spellings, which lower to
/// numeric weights rather than reserved handles.
#[must_use]
pub fn font_weight_relative_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "bolder" => Some(FONT_WEIGHT_BOLDER),
        "lighter" => Some(FONT_WEIGHT_LIGHTER),
        _ => None,
    }
}

/// Maps a `grid-auto-flow` value to its reserved [`InternedSymbol`].
///
/// This is a *multi-keyword* property: the axis (`row` / `column`) and the
/// optional `dense` packing keyword may appear together in either order. The
/// input is ASCII-lowercased and split on whitespace, then each accepted
/// combination folds to a single reserved handle:
///
/// * `row` → [`GRID_AUTO_FLOW_ROW`] (the default)
/// * `column` → [`GRID_AUTO_FLOW_COLUMN`]
/// * `dense` (bare) → [`GRID_AUTO_FLOW_ROW_DENSE`] (`row` is the implied axis)
/// * `row dense` / `dense row` → [`GRID_AUTO_FLOW_ROW_DENSE`]
/// * `column dense` / `dense column` → [`GRID_AUTO_FLOW_COLUMN_DENSE`]
///
/// Returns [`None`] for any other combination.
#[must_use]
pub fn grid_auto_flow_from_name(name: &str) -> Option<InternedSymbol> {
    let lowered = name.to_ascii_lowercase();
    let tokens: Vec<&str> = lowered.split_whitespace().collect();
    match tokens.as_slice() {
        ["row"] => Some(GRID_AUTO_FLOW_ROW),
        ["column"] => Some(GRID_AUTO_FLOW_COLUMN),
        // Bare `dense` implies the `row` axis, same as `row dense`.
        ["dense"] | ["row", "dense"] | ["dense", "row"] => Some(GRID_AUTO_FLOW_ROW_DENSE),
        ["column", "dense"] | ["dense", "column"] => Some(GRID_AUTO_FLOW_COLUMN_DENSE),
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
        POSITION_STATIC => Some("static"),
        POSITION_RELATIVE => Some("relative"),
        POSITION_ABSOLUTE => Some("absolute"),
        POSITION_FIXED => Some("fixed"),
        POSITION_STICKY => Some("sticky"),
        Z_INDEX_AUTO => Some("auto"),
        FLEX_ROW => Some("row"),
        FLEX_ROW_REVERSE => Some("row-reverse"),
        FLEX_COLUMN => Some("column"),
        FLEX_COLUMN_REVERSE => Some("column-reverse"),
        JUSTIFY_START => Some("flex-start"),
        JUSTIFY_CENTER => Some("center"),
        JUSTIFY_END => Some("flex-end"),
        JUSTIFY_SPACE_BETWEEN => Some("space-between"),
        JUSTIFY_SPACE_AROUND => Some("space-around"),
        JUSTIFY_SPACE_EVENLY => Some("space-evenly"),
        ALIGN_START => Some("flex-start"),
        ALIGN_CENTER => Some("center"),
        ALIGN_END => Some("flex-end"),
        ALIGN_STRETCH => Some("stretch"),
        ALIGN_AUTO => Some("auto"),
        DIRECTION_LTR => Some("ltr"),
        DIRECTION_RTL => Some("rtl"),
        FONT_STYLE_NORMAL => Some("normal"),
        FONT_STYLE_ITALIC => Some("italic"),
        FONT_STYLE_OBLIQUE => Some("oblique"),
        TEXT_ALIGN_LEFT => Some("left"),
        TEXT_ALIGN_RIGHT => Some("right"),
        TEXT_ALIGN_CENTER => Some("center"),
        TEXT_ALIGN_JUSTIFY => Some("justify"),
        TEXT_ALIGN_START => Some("start"),
        TEXT_ALIGN_END => Some("end"),
        TEXT_OVERFLOW_CLIP => Some("clip"),
        TEXT_OVERFLOW_ELLIPSIS => Some("ellipsis"),
        FONT_WEIGHT_BOLDER => Some("bolder"),
        FONT_WEIGHT_LIGHTER => Some("lighter"),
        GRID_AUTO_FLOW_ROW => Some("row"),
        GRID_AUTO_FLOW_COLUMN => Some("column"),
        GRID_AUTO_FLOW_ROW_DENSE => Some("row dense"),
        GRID_AUTO_FLOW_COLUMN_DENSE => Some("column dense"),
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
        POSITION_STATIC,
        POSITION_RELATIVE,
        POSITION_ABSOLUTE,
        POSITION_FIXED,
        POSITION_STICKY,
        Z_INDEX_AUTO,
        FLEX_ROW,
        FLEX_ROW_REVERSE,
        FLEX_COLUMN,
        FLEX_COLUMN_REVERSE,
        JUSTIFY_START,
        JUSTIFY_CENTER,
        JUSTIFY_END,
        JUSTIFY_SPACE_BETWEEN,
        JUSTIFY_SPACE_AROUND,
        JUSTIFY_SPACE_EVENLY,
        ALIGN_START,
        ALIGN_CENTER,
        ALIGN_END,
        ALIGN_STRETCH,
        ALIGN_AUTO,
        DIRECTION_LTR,
        DIRECTION_RTL,
        FONT_STYLE_NORMAL,
        FONT_STYLE_ITALIC,
        FONT_STYLE_OBLIQUE,
        TEXT_ALIGN_LEFT,
        TEXT_ALIGN_RIGHT,
        TEXT_ALIGN_CENTER,
        TEXT_ALIGN_JUSTIFY,
        TEXT_ALIGN_START,
        TEXT_ALIGN_END,
        TEXT_OVERFLOW_CLIP,
        TEXT_OVERFLOW_ELLIPSIS,
        FONT_WEIGHT_BOLDER,
        FONT_WEIGHT_LIGHTER,
        GRID_AUTO_FLOW_ROW,
        GRID_AUTO_FLOW_COLUMN,
        GRID_AUTO_FLOW_ROW_DENSE,
        GRID_AUTO_FLOW_COLUMN_DENSE,
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
    fn position_maps_known_keywords() {
        assert_eq!(position_from_name("static"), Some(POSITION_STATIC));
        assert_eq!(position_from_name("relative"), Some(POSITION_RELATIVE));
        assert_eq!(position_from_name("absolute"), Some(POSITION_ABSOLUTE));
        assert_eq!(position_from_name("Fixed"), Some(POSITION_FIXED));
        assert_eq!(position_from_name("sticky"), Some(POSITION_STICKY));
        assert_eq!(position_from_name("Sticky"), Some(POSITION_STICKY));
        assert_eq!(position_from_name("subgrid"), None);
        // Round-trips through `name_of`.
        assert_eq!(name_of(POSITION_RELATIVE), Some("relative"));
        assert_eq!(name_of(POSITION_STATIC), Some("static"));
        assert_eq!(name_of(POSITION_STICKY), Some("sticky"));
    }

    #[test]
    fn z_index_maps_auto_only() {
        assert_eq!(z_index_from_name("auto"), Some(Z_INDEX_AUTO));
        assert_eq!(z_index_from_name("AUTO"), Some(Z_INDEX_AUTO));
        // The `<integer>` form is not a keyword, and no other keyword is valid.
        assert_eq!(z_index_from_name("0"), None);
        assert_eq!(z_index_from_name("static"), None);
        // Round-trips through `name_of`.
        assert_eq!(name_of(Z_INDEX_AUTO), Some("auto"));
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
        assert_eq!(
            justify_content_from_name("space-evenly"),
            Some(JUSTIFY_SPACE_EVENLY)
        );
    }

    #[test]
    fn align_content_shares_justify_content_grammar() {
        // `align-content` delegates to the `justify-content` value grammar,
        // including the modern `start` / `end` spellings and `space-evenly`.
        assert_eq!(align_content_from_name("start"), Some(JUSTIFY_START));
        assert_eq!(align_content_from_name("center"), Some(JUSTIFY_CENTER));
        assert_eq!(align_content_from_name("end"), Some(JUSTIFY_END));
        assert_eq!(
            align_content_from_name("space-between"),
            Some(JUSTIFY_SPACE_BETWEEN)
        );
        assert_eq!(
            align_content_from_name("space-evenly"),
            Some(JUSTIFY_SPACE_EVENLY)
        );
        assert_eq!(align_content_from_name("stretch"), None);
        // Round-trips through `name_of`.
        assert_eq!(name_of(JUSTIFY_SPACE_EVENLY), Some("space-evenly"));
    }

    #[test]
    fn align_items_maps_known_keywords() {
        assert_eq!(align_items_from_name("flex-start"), Some(ALIGN_START));
        assert_eq!(align_items_from_name("start"), Some(ALIGN_START));
        assert_eq!(align_items_from_name("center"), Some(ALIGN_CENTER));
        assert_eq!(align_items_from_name("flex-end"), Some(ALIGN_END));
        assert_eq!(align_items_from_name("stretch"), Some(ALIGN_STRETCH));
        assert_eq!(align_items_from_name("baseline"), None);
        // `auto` is only valid on the *-self properties, not `align-items`.
        assert_eq!(align_items_from_name("auto"), None);
    }

    #[test]
    fn align_self_maps_known_keywords_including_auto() {
        // Accepts everything `align-items` does…
        assert_eq!(align_self_from_name("flex-start"), Some(ALIGN_START));
        assert_eq!(align_self_from_name("start"), Some(ALIGN_START));
        assert_eq!(align_self_from_name("center"), Some(ALIGN_CENTER));
        assert_eq!(align_self_from_name("end"), Some(ALIGN_END));
        assert_eq!(align_self_from_name("stretch"), Some(ALIGN_STRETCH));
        // …plus `auto`, case-insensitively.
        assert_eq!(align_self_from_name("auto"), Some(ALIGN_AUTO));
        assert_eq!(align_self_from_name("AUTO"), Some(ALIGN_AUTO));
        assert_eq!(align_self_from_name("baseline"), None);
        // Round-trips through `name_of`.
        assert_eq!(name_of(ALIGN_AUTO), Some("auto"));
    }

    #[test]
    fn direction_maps_known_keywords() {
        assert_eq!(direction_from_name("ltr"), Some(DIRECTION_LTR));
        assert_eq!(direction_from_name("rtl"), Some(DIRECTION_RTL));
        assert_eq!(direction_from_name("RTL"), Some(DIRECTION_RTL));
        assert_eq!(direction_from_name("sideways"), None);
    }

    #[test]
    fn font_style_maps_known_keywords() {
        assert_eq!(font_style_from_name("normal"), Some(FONT_STYLE_NORMAL));
        assert_eq!(font_style_from_name("italic"), Some(FONT_STYLE_ITALIC));
        assert_eq!(font_style_from_name("Oblique"), Some(FONT_STYLE_OBLIQUE));
        assert_eq!(font_style_from_name("slanted"), None);
    }

    #[test]
    fn name_of_round_trips_direction_and_font_style() {
        assert_eq!(name_of(DIRECTION_RTL), Some("rtl"));
        assert_eq!(name_of(FONT_STYLE_ITALIC), Some("italic"));
        assert_eq!(name_of(FONT_STYLE_OBLIQUE), Some("oblique"));
    }

    #[test]
    fn text_align_maps_known_keywords() {
        assert_eq!(text_align_from_name("left"), Some(TEXT_ALIGN_LEFT));
        assert_eq!(text_align_from_name("right"), Some(TEXT_ALIGN_RIGHT));
        assert_eq!(text_align_from_name("center"), Some(TEXT_ALIGN_CENTER));
        assert_eq!(text_align_from_name("justify"), Some(TEXT_ALIGN_JUSTIFY));
        assert_eq!(text_align_from_name("start"), Some(TEXT_ALIGN_START));
        assert_eq!(text_align_from_name("END"), Some(TEXT_ALIGN_END));
        assert_eq!(text_align_from_name("middle"), None);
    }

    #[test]
    fn text_overflow_maps_known_keywords() {
        assert_eq!(text_overflow_from_name("clip"), Some(TEXT_OVERFLOW_CLIP));
        assert_eq!(
            text_overflow_from_name("ellipsis"),
            Some(TEXT_OVERFLOW_ELLIPSIS)
        );
        assert_eq!(text_overflow_from_name("ELLIPSIS"), Some(TEXT_OVERFLOW_ELLIPSIS));
        assert_eq!(text_overflow_from_name("fade"), None);
        // Round-trips through `name_of`.
        assert_eq!(name_of(TEXT_OVERFLOW_CLIP), Some("clip"));
        assert_eq!(name_of(TEXT_OVERFLOW_ELLIPSIS), Some("ellipsis"));
    }

    #[test]
    fn font_weight_relative_maps_known_keywords() {
        assert_eq!(
            font_weight_relative_from_name("bolder"),
            Some(FONT_WEIGHT_BOLDER)
        );
        assert_eq!(
            font_weight_relative_from_name("lighter"),
            Some(FONT_WEIGHT_LIGHTER)
        );
        // Case-insensitive.
        assert_eq!(
            font_weight_relative_from_name("BOLDER"),
            Some(FONT_WEIGHT_BOLDER)
        );
        // Absolute keywords and unknowns map to nothing (they lower to numbers).
        assert_eq!(font_weight_relative_from_name("bold"), None);
        assert_eq!(font_weight_relative_from_name("normal"), None);
        // Round-trips through `name_of`.
        assert_eq!(name_of(FONT_WEIGHT_BOLDER), Some("bolder"));
        assert_eq!(name_of(FONT_WEIGHT_LIGHTER), Some("lighter"));
    }

    #[test]
    fn name_of_round_trips_text_align() {
        assert_eq!(name_of(TEXT_ALIGN_LEFT), Some("left"));
        assert_eq!(name_of(TEXT_ALIGN_JUSTIFY), Some("justify"));
        assert_eq!(name_of(TEXT_ALIGN_START), Some("start"));
        assert_eq!(name_of(TEXT_ALIGN_END), Some("end"));
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
    fn grid_auto_flow_maps_single_and_multi_keyword_values() {
        assert_eq!(grid_auto_flow_from_name("row"), Some(GRID_AUTO_FLOW_ROW));
        assert_eq!(
            grid_auto_flow_from_name("column"),
            Some(GRID_AUTO_FLOW_COLUMN)
        );
        // Bare `dense` implies the `row` axis.
        assert_eq!(
            grid_auto_flow_from_name("dense"),
            Some(GRID_AUTO_FLOW_ROW_DENSE)
        );
        // Multi-keyword, either order, case-insensitive.
        assert_eq!(
            grid_auto_flow_from_name("row dense"),
            Some(GRID_AUTO_FLOW_ROW_DENSE)
        );
        assert_eq!(
            grid_auto_flow_from_name("Dense Row"),
            Some(GRID_AUTO_FLOW_ROW_DENSE)
        );
        assert_eq!(
            grid_auto_flow_from_name("column dense"),
            Some(GRID_AUTO_FLOW_COLUMN_DENSE)
        );
        assert_eq!(
            grid_auto_flow_from_name("dense column"),
            Some(GRID_AUTO_FLOW_COLUMN_DENSE)
        );
        // Unknown / malformed.
        assert_eq!(grid_auto_flow_from_name("dense dense"), None);
        assert_eq!(grid_auto_flow_from_name("diagonal"), None);
        // Round-trips through `name_of`.
        assert_eq!(name_of(GRID_AUTO_FLOW_COLUMN_DENSE), Some("column dense"));
        assert_eq!(name_of(GRID_AUTO_FLOW_ROW), Some("row"));
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
