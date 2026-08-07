#![doc = include_str!("layout_keyword.doc/_module.md")]

use obscur_identity::InternedSymbol;

#[doc = include_str!("layout_keyword.doc/DISPLAY_BLOCK.md")]
pub const DISPLAY_BLOCK: InternedSymbol = InternedSymbol::from_raw(0x0000_1001);
#[doc = include_str!("layout_keyword.doc/DISPLAY_FLEX.md")]
pub const DISPLAY_FLEX: InternedSymbol = InternedSymbol::from_raw(0x0000_1002);
#[doc = include_str!("layout_keyword.doc/DISPLAY_GRID.md")]
pub const DISPLAY_GRID: InternedSymbol = InternedSymbol::from_raw(0x0000_1003);
#[doc = include_str!("layout_keyword.doc/DISPLAY_NONE.md")]
pub const DISPLAY_NONE: InternedSymbol = InternedSymbol::from_raw(0x0000_1004);
#[doc = include_str!("layout_keyword.doc/DISPLAY_CONTENTS.md")]
pub const DISPLAY_CONTENTS: InternedSymbol = InternedSymbol::from_raw(0x0000_1005);

#[doc = include_str!("layout_keyword.doc/FLEX_ROW.md")]
pub const FLEX_ROW: InternedSymbol = InternedSymbol::from_raw(0x0000_1101);
#[doc = include_str!("layout_keyword.doc/FLEX_ROW_REVERSE.md")]
pub const FLEX_ROW_REVERSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1102);
#[doc = include_str!("layout_keyword.doc/FLEX_COLUMN.md")]
pub const FLEX_COLUMN: InternedSymbol = InternedSymbol::from_raw(0x0000_1103);
#[doc = include_str!("layout_keyword.doc/FLEX_COLUMN_REVERSE.md")]
pub const FLEX_COLUMN_REVERSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1104);

#[doc = include_str!("layout_keyword.doc/FLEX_WRAP_NOWRAP.md")]
pub const FLEX_WRAP_NOWRAP: InternedSymbol = InternedSymbol::from_raw(0x0000_1F01);
#[doc = include_str!("layout_keyword.doc/FLEX_WRAP_WRAP.md")]
pub const FLEX_WRAP_WRAP: InternedSymbol = InternedSymbol::from_raw(0x0000_1F02);
#[doc = include_str!("layout_keyword.doc/FLEX_WRAP_WRAP_REVERSE.md")]
pub const FLEX_WRAP_WRAP_REVERSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1F03);

#[doc = include_str!("layout_keyword.doc/JUSTIFY_START.md")]
pub const JUSTIFY_START: InternedSymbol = InternedSymbol::from_raw(0x0000_1201);
#[doc = include_str!("layout_keyword.doc/JUSTIFY_CENTER.md")]
pub const JUSTIFY_CENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1202);
#[doc = include_str!("layout_keyword.doc/JUSTIFY_END.md")]
pub const JUSTIFY_END: InternedSymbol = InternedSymbol::from_raw(0x0000_1203);
#[doc = include_str!("layout_keyword.doc/JUSTIFY_SPACE_BETWEEN.md")]
pub const JUSTIFY_SPACE_BETWEEN: InternedSymbol = InternedSymbol::from_raw(0x0000_1204);
#[doc = include_str!("layout_keyword.doc/JUSTIFY_SPACE_AROUND.md")]
pub const JUSTIFY_SPACE_AROUND: InternedSymbol = InternedSymbol::from_raw(0x0000_1205);
#[doc = include_str!("layout_keyword.doc/JUSTIFY_SPACE_EVENLY.md")]
pub const JUSTIFY_SPACE_EVENLY: InternedSymbol = InternedSymbol::from_raw(0x0000_1206);
#[doc = include_str!("layout_keyword.doc/JUSTIFY_STRETCH.md")]
pub const JUSTIFY_STRETCH: InternedSymbol = InternedSymbol::from_raw(0x0000_1207);

#[doc = include_str!("layout_keyword.doc/ALIGN_START.md")]
pub const ALIGN_START: InternedSymbol = InternedSymbol::from_raw(0x0000_1301);
#[doc = include_str!("layout_keyword.doc/ALIGN_CENTER.md")]
pub const ALIGN_CENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1302);
#[doc = include_str!("layout_keyword.doc/ALIGN_END.md")]
pub const ALIGN_END: InternedSymbol = InternedSymbol::from_raw(0x0000_1303);
#[doc = include_str!("layout_keyword.doc/ALIGN_STRETCH.md")]
pub const ALIGN_STRETCH: InternedSymbol = InternedSymbol::from_raw(0x0000_1304);
#[doc = include_str!("layout_keyword.doc/ALIGN_AUTO.md")]
pub const ALIGN_AUTO: InternedSymbol = InternedSymbol::from_raw(0x0000_1305);

#[doc = include_str!("layout_keyword.doc/DIRECTION_LTR.md")]
pub const DIRECTION_LTR: InternedSymbol = InternedSymbol::from_raw(0x0000_1401);
#[doc = include_str!("layout_keyword.doc/DIRECTION_RTL.md")]
pub const DIRECTION_RTL: InternedSymbol = InternedSymbol::from_raw(0x0000_1402);

#[doc = include_str!("layout_keyword.doc/WRITING_MODE_HORIZONTAL_TB.md")]
pub const WRITING_MODE_HORIZONTAL_TB: InternedSymbol = InternedSymbol::from_raw(0x0000_2601);
#[doc = include_str!("layout_keyword.doc/WRITING_MODE_VERTICAL_RL.md")]
pub const WRITING_MODE_VERTICAL_RL: InternedSymbol = InternedSymbol::from_raw(0x0000_2602);
#[doc = include_str!("layout_keyword.doc/WRITING_MODE_VERTICAL_LR.md")]
pub const WRITING_MODE_VERTICAL_LR: InternedSymbol = InternedSymbol::from_raw(0x0000_2603);

#[doc = include_str!("layout_keyword.doc/FONT_STYLE_NORMAL.md")]
pub const FONT_STYLE_NORMAL: InternedSymbol = InternedSymbol::from_raw(0x0000_1501);
#[doc = include_str!("layout_keyword.doc/FONT_STYLE_ITALIC.md")]
pub const FONT_STYLE_ITALIC: InternedSymbol = InternedSymbol::from_raw(0x0000_1502);
#[doc = include_str!("layout_keyword.doc/FONT_STYLE_OBLIQUE.md")]
pub const FONT_STYLE_OBLIQUE: InternedSymbol = InternedSymbol::from_raw(0x0000_1503);

#[doc = include_str!("layout_keyword.doc/TEXT_ALIGN_LEFT.md")]
pub const TEXT_ALIGN_LEFT: InternedSymbol = InternedSymbol::from_raw(0x0000_1601);
#[doc = include_str!("layout_keyword.doc/TEXT_ALIGN_RIGHT.md")]
pub const TEXT_ALIGN_RIGHT: InternedSymbol = InternedSymbol::from_raw(0x0000_1602);
#[doc = include_str!("layout_keyword.doc/TEXT_ALIGN_CENTER.md")]
pub const TEXT_ALIGN_CENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1603);
#[doc = include_str!("layout_keyword.doc/TEXT_ALIGN_JUSTIFY.md")]
pub const TEXT_ALIGN_JUSTIFY: InternedSymbol = InternedSymbol::from_raw(0x0000_1604);
#[doc = include_str!("layout_keyword.doc/TEXT_ALIGN_START.md")]
pub const TEXT_ALIGN_START: InternedSymbol = InternedSymbol::from_raw(0x0000_1605);
#[doc = include_str!("layout_keyword.doc/TEXT_ALIGN_END.md")]
pub const TEXT_ALIGN_END: InternedSymbol = InternedSymbol::from_raw(0x0000_1606);

#[doc = include_str!("layout_keyword.doc/TEXT_OVERFLOW_CLIP.md")]
pub const TEXT_OVERFLOW_CLIP: InternedSymbol = InternedSymbol::from_raw(0x0000_1701);
#[doc = include_str!("layout_keyword.doc/TEXT_OVERFLOW_ELLIPSIS.md")]
pub const TEXT_OVERFLOW_ELLIPSIS: InternedSymbol = InternedSymbol::from_raw(0x0000_1702);

// ---- `text-transform` keyword values (CSS Text 3 §2.1) ----
//
// The `0x0000_1xxx` category nibbles (`0`..`F`) are exhausted, so this closed set
// opens the adjacent `0x0000_2xxx` reservation block (still disjoint from the CSS
// interner's `0x00C1_xxxx` and the element-attribute `0x0000_000x` ranges).

#[doc = include_str!("layout_keyword.doc/TEXT_TRANSFORM_NONE.md")]
pub const TEXT_TRANSFORM_NONE: InternedSymbol = InternedSymbol::from_raw(0x0000_2001);
#[doc = include_str!("layout_keyword.doc/TEXT_TRANSFORM_UPPERCASE.md")]
pub const TEXT_TRANSFORM_UPPERCASE: InternedSymbol = InternedSymbol::from_raw(0x0000_2002);
#[doc = include_str!("layout_keyword.doc/TEXT_TRANSFORM_LOWERCASE.md")]
pub const TEXT_TRANSFORM_LOWERCASE: InternedSymbol = InternedSymbol::from_raw(0x0000_2003);
#[doc = include_str!("layout_keyword.doc/TEXT_TRANSFORM_CAPITALIZE.md")]
pub const TEXT_TRANSFORM_CAPITALIZE: InternedSymbol = InternedSymbol::from_raw(0x0000_2004);

// ---- `white-space` keyword values (CSS Text 3 §3) ----

#[doc = include_str!("layout_keyword.doc/WHITE_SPACE_NORMAL.md")]
pub const WHITE_SPACE_NORMAL: InternedSymbol = InternedSymbol::from_raw(0x0000_2101);
#[doc = include_str!("layout_keyword.doc/WHITE_SPACE_NOWRAP.md")]
pub const WHITE_SPACE_NOWRAP: InternedSymbol = InternedSymbol::from_raw(0x0000_2102);
#[doc = include_str!("layout_keyword.doc/WHITE_SPACE_PRE.md")]
pub const WHITE_SPACE_PRE: InternedSymbol = InternedSymbol::from_raw(0x0000_2103);
#[doc = include_str!("layout_keyword.doc/WHITE_SPACE_PRE_WRAP.md")]
pub const WHITE_SPACE_PRE_WRAP: InternedSymbol = InternedSymbol::from_raw(0x0000_2104);
#[doc = include_str!("layout_keyword.doc/WHITE_SPACE_PRE_LINE.md")]
pub const WHITE_SPACE_PRE_LINE: InternedSymbol = InternedSymbol::from_raw(0x0000_2105);

// ---- `overflow-wrap` keyword values (CSS Text 3 §5.2) ----

#[doc = include_str!("layout_keyword.doc/OVERFLOW_WRAP_NORMAL.md")]
pub const OVERFLOW_WRAP_NORMAL: InternedSymbol = InternedSymbol::from_raw(0x0000_2201);
#[doc = include_str!("layout_keyword.doc/OVERFLOW_WRAP_BREAK_WORD.md")]
pub const OVERFLOW_WRAP_BREAK_WORD: InternedSymbol = InternedSymbol::from_raw(0x0000_2202);
#[doc = include_str!("layout_keyword.doc/OVERFLOW_WRAP_ANYWHERE.md")]
pub const OVERFLOW_WRAP_ANYWHERE: InternedSymbol = InternedSymbol::from_raw(0x0000_2203);

// ---- `word-break` keyword values (CSS Text 3 §5.2) ----

#[doc = include_str!("layout_keyword.doc/WORD_BREAK_NORMAL.md")]
pub const WORD_BREAK_NORMAL: InternedSymbol = InternedSymbol::from_raw(0x0000_2301);
#[doc = include_str!("layout_keyword.doc/WORD_BREAK_BREAK_ALL.md")]
pub const WORD_BREAK_BREAK_ALL: InternedSymbol = InternedSymbol::from_raw(0x0000_2302);
#[doc = include_str!("layout_keyword.doc/WORD_BREAK_KEEP_ALL.md")]
pub const WORD_BREAK_KEEP_ALL: InternedSymbol = InternedSymbol::from_raw(0x0000_2303);

// ---- intrinsic sizing keyword values (CSS Sizing 3 §4.1) ----
//
// These are values of the *sizing* properties (`width` / `height` / `min-*` /
// `max-*` / `flex-basis`), and unlike every other keyword family here they are
// not a closed choice the layout reads off a style: each is a question about the
// content, answered by measuring it. The style layer's job is only to carry
// which question was asked; `obscur_layout::model::Dimension` is where they
// become sizes.

#[doc = include_str!("layout_keyword.doc/SIZING_MIN_CONTENT.md")]
pub const SIZING_MIN_CONTENT: InternedSymbol = InternedSymbol::from_raw(0x0000_2401);
#[doc = include_str!("layout_keyword.doc/SIZING_MAX_CONTENT.md")]
pub const SIZING_MAX_CONTENT: InternedSymbol = InternedSymbol::from_raw(0x0000_2402);
#[doc = include_str!("layout_keyword.doc/SIZING_FIT_CONTENT.md")]
pub const SIZING_FIT_CONTENT: InternedSymbol = InternedSymbol::from_raw(0x0000_2403);

#[doc = include_str!("layout_keyword.doc/FONT_WEIGHT_BOLDER.md")]
pub const FONT_WEIGHT_BOLDER: InternedSymbol = InternedSymbol::from_raw(0x0000_1801);
#[doc = include_str!("layout_keyword.doc/FONT_WEIGHT_LIGHTER.md")]
pub const FONT_WEIGHT_LIGHTER: InternedSymbol = InternedSymbol::from_raw(0x0000_1802);

// ---- `grid-auto-flow` keyword values ----
//
// `grid-auto-flow` is a multi-keyword property: an axis (`row` / `column`) and
// an optional `dense` packing keyword combine (`row dense`, `column dense`).
// Rather than carry two symbols, each authored combination folds to ONE reserved
// handle, so the runtime reads a single keyword identity.

#[doc = include_str!("layout_keyword.doc/GRID_AUTO_FLOW_ROW.md")]
pub const GRID_AUTO_FLOW_ROW: InternedSymbol = InternedSymbol::from_raw(0x0000_1901);
#[doc = include_str!("layout_keyword.doc/GRID_AUTO_FLOW_COLUMN.md")]
pub const GRID_AUTO_FLOW_COLUMN: InternedSymbol = InternedSymbol::from_raw(0x0000_1902);
#[doc = include_str!("layout_keyword.doc/GRID_AUTO_FLOW_ROW_DENSE.md")]
pub const GRID_AUTO_FLOW_ROW_DENSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1903);
#[doc = include_str!("layout_keyword.doc/GRID_AUTO_FLOW_COLUMN_DENSE.md")]
pub const GRID_AUTO_FLOW_COLUMN_DENSE: InternedSymbol = InternedSymbol::from_raw(0x0000_1904);

#[doc = include_str!("layout_keyword.doc/POSITION_STATIC.md")]
pub const POSITION_STATIC: InternedSymbol = InternedSymbol::from_raw(0x0000_1A01);
#[doc = include_str!("layout_keyword.doc/POSITION_RELATIVE.md")]
pub const POSITION_RELATIVE: InternedSymbol = InternedSymbol::from_raw(0x0000_1A02);
#[doc = include_str!("layout_keyword.doc/POSITION_ABSOLUTE.md")]
pub const POSITION_ABSOLUTE: InternedSymbol = InternedSymbol::from_raw(0x0000_1A03);
#[doc = include_str!("layout_keyword.doc/POSITION_FIXED.md")]
pub const POSITION_FIXED: InternedSymbol = InternedSymbol::from_raw(0x0000_1A04);
#[doc = include_str!("layout_keyword.doc/POSITION_STICKY.md")]
pub const POSITION_STICKY: InternedSymbol = InternedSymbol::from_raw(0x0000_1A05);

#[doc = include_str!("layout_keyword.doc/Z_INDEX_AUTO.md")]
pub const Z_INDEX_AUTO: InternedSymbol = InternedSymbol::from_raw(0x0000_1B01);

// ---- `border-*-style` keyword values ----
//
// The full CSS line-style set (`dashed` / `dotted` / `double` / `groove` /
// `ridge` / `inset` / `outset`) is not modelled as distinct raster styles yet;
// every visible style folds to the single `solid` handle (a documented
// simplification), while `none` / `hidden` fold to `none` (no border painted).

#[doc = include_str!("layout_keyword.doc/BORDER_STYLE_NONE.md")]
pub const BORDER_STYLE_NONE: InternedSymbol = InternedSymbol::from_raw(0x0000_1C01);
#[doc = include_str!("layout_keyword.doc/BORDER_STYLE_SOLID.md")]
pub const BORDER_STYLE_SOLID: InternedSymbol = InternedSymbol::from_raw(0x0000_1C02);
#[doc = include_str!("layout_keyword.doc/BORDER_STYLE_DASHED.md")]
pub const BORDER_STYLE_DASHED: InternedSymbol = InternedSymbol::from_raw(0x0000_1C03);

#[doc = include_str!("layout_keyword.doc/BOX_SIZING_CONTENT_BOX.md")]
pub const BOX_SIZING_CONTENT_BOX: InternedSymbol = InternedSymbol::from_raw(0x0000_1D01);
#[doc = include_str!("layout_keyword.doc/BOX_SIZING_BORDER_BOX.md")]
pub const BOX_SIZING_BORDER_BOX: InternedSymbol = InternedSymbol::from_raw(0x0000_1D02);

// ---- `overflow` keyword values ----
//
// A value other than `visible` clips the element's subtree to its box. `scroll`
// and `auto` are carried as distinct handles for fidelity but currently clip
// like `hidden` — no scrollbars are synthesised (a documented simplification).

#[doc = include_str!("layout_keyword.doc/OVERFLOW_VISIBLE.md")]
pub const OVERFLOW_VISIBLE: InternedSymbol = InternedSymbol::from_raw(0x0000_1E01);
#[doc = include_str!("layout_keyword.doc/OVERFLOW_HIDDEN.md")]
pub const OVERFLOW_HIDDEN: InternedSymbol = InternedSymbol::from_raw(0x0000_1E02);
#[doc = include_str!("layout_keyword.doc/OVERFLOW_CLIP.md")]
pub const OVERFLOW_CLIP: InternedSymbol = InternedSymbol::from_raw(0x0000_1E03);
#[doc = include_str!("layout_keyword.doc/OVERFLOW_SCROLL.md")]
pub const OVERFLOW_SCROLL: InternedSymbol = InternedSymbol::from_raw(0x0000_1E04);
#[doc = include_str!("layout_keyword.doc/OVERFLOW_AUTO.md")]
pub const OVERFLOW_AUTO: InternedSymbol = InternedSymbol::from_raw(0x0000_1E05);

#[doc = include_str!("layout_keyword.doc/display_from_name.md")]
#[must_use]
pub fn display_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "block" => Some(DISPLAY_BLOCK),
        "flex" => Some(DISPLAY_FLEX),
        "grid" => Some(DISPLAY_GRID),
        "none" => Some(DISPLAY_NONE),
        "contents" => Some(DISPLAY_CONTENTS),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/position_from_name.md")]
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

#[doc = include_str!("layout_keyword.doc/z_index_from_name.md")]
#[must_use]
pub fn z_index_from_name(name: &str) -> Option<InternedSymbol> {
    if name.eq_ignore_ascii_case("auto") {
        Some(Z_INDEX_AUTO)
    } else {
        None
    }
}

#[doc = include_str!("layout_keyword.doc/border_style_from_name.md")]
#[must_use]
pub fn border_style_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "none" | "hidden" => Some(BORDER_STYLE_NONE),
        "dashed" | "dotted" => Some(BORDER_STYLE_DASHED),
        "solid" | "double" | "groove" | "ridge" | "inset" | "outset" => Some(BORDER_STYLE_SOLID),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/box_sizing_from_name.md")]
#[must_use]
pub fn box_sizing_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "content-box" => Some(BOX_SIZING_CONTENT_BOX),
        "border-box" => Some(BOX_SIZING_BORDER_BOX),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/flex_direction_from_name.md")]
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

#[doc = include_str!("layout_keyword.doc/flex_wrap_from_name.md")]
#[must_use]
pub fn flex_wrap_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "nowrap" => Some(FLEX_WRAP_NOWRAP),
        "wrap" => Some(FLEX_WRAP_WRAP),
        "wrap-reverse" => Some(FLEX_WRAP_WRAP_REVERSE),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/justify_content_from_name.md")]
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

#[doc = include_str!("layout_keyword.doc/align_content_from_name.md")]
#[must_use]
pub fn align_content_from_name(name: &str) -> Option<InternedSymbol> {
    // `align-content` shares `justify-content`'s distribution grammar but adds
    // `stretch` (grow lines/tracks to fill leftover cross space).
    if name.eq_ignore_ascii_case("stretch") {
        return Some(JUSTIFY_STRETCH);
    }
    justify_content_from_name(name)
}

#[doc = include_str!("layout_keyword.doc/align_items_from_name.md")]
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

#[doc = include_str!("layout_keyword.doc/align_self_from_name.md")]
#[must_use]
pub fn align_self_from_name(name: &str) -> Option<InternedSymbol> {
    if name.eq_ignore_ascii_case("auto") {
        return Some(ALIGN_AUTO);
    }
    align_items_from_name(name)
}

#[doc = include_str!("layout_keyword.doc/direction_from_name.md")]
#[must_use]
pub fn direction_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "ltr" => Some(DIRECTION_LTR),
        "rtl" => Some(DIRECTION_RTL),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/writing_mode_from_name.md")]
#[must_use]
pub fn writing_mode_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "horizontal-tb" => Some(WRITING_MODE_HORIZONTAL_TB),
        "vertical-rl" => Some(WRITING_MODE_VERTICAL_RL),
        "vertical-lr" => Some(WRITING_MODE_VERTICAL_LR),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/font_style_from_name.md")]
#[must_use]
pub fn font_style_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "normal" => Some(FONT_STYLE_NORMAL),
        "italic" => Some(FONT_STYLE_ITALIC),
        "oblique" => Some(FONT_STYLE_OBLIQUE),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/text_align_from_name.md")]
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

#[doc = include_str!("layout_keyword.doc/text_overflow_from_name.md")]
#[must_use]
pub fn text_overflow_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "clip" => Some(TEXT_OVERFLOW_CLIP),
        "ellipsis" => Some(TEXT_OVERFLOW_ELLIPSIS),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/text_transform_from_name.md")]
#[must_use]
pub fn text_transform_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "none" => Some(TEXT_TRANSFORM_NONE),
        "uppercase" => Some(TEXT_TRANSFORM_UPPERCASE),
        "lowercase" => Some(TEXT_TRANSFORM_LOWERCASE),
        "capitalize" => Some(TEXT_TRANSFORM_CAPITALIZE),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/white_space_from_name.md")]
#[must_use]
pub fn white_space_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "normal" => Some(WHITE_SPACE_NORMAL),
        "nowrap" => Some(WHITE_SPACE_NOWRAP),
        "pre" => Some(WHITE_SPACE_PRE),
        "pre-wrap" => Some(WHITE_SPACE_PRE_WRAP),
        "pre-line" => Some(WHITE_SPACE_PRE_LINE),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/overflow_wrap_from_name.md")]
#[must_use]
pub fn overflow_wrap_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "normal" => Some(OVERFLOW_WRAP_NORMAL),
        "break-word" => Some(OVERFLOW_WRAP_BREAK_WORD),
        "anywhere" => Some(OVERFLOW_WRAP_ANYWHERE),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/word_break_from_name.md")]
#[must_use]
pub fn word_break_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "normal" => Some(WORD_BREAK_NORMAL),
        "break-all" => Some(WORD_BREAK_BREAK_ALL),
        "keep-all" => Some(WORD_BREAK_KEEP_ALL),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/sizing_from_name.md")]
#[must_use]
pub fn sizing_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "min-content" => Some(SIZING_MIN_CONTENT),
        "max-content" => Some(SIZING_MAX_CONTENT),
        "fit-content" => Some(SIZING_FIT_CONTENT),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/overflow_from_name.md")]
#[must_use]
pub fn overflow_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "visible" => Some(OVERFLOW_VISIBLE),
        "hidden" => Some(OVERFLOW_HIDDEN),
        "clip" => Some(OVERFLOW_CLIP),
        "scroll" => Some(OVERFLOW_SCROLL),
        "auto" => Some(OVERFLOW_AUTO),
        _ => None,
    }
}

// ---- `mix-blend-mode` keyword values ----
//
// The separable blend modes plus `normal` (the initial). A value other than
// `normal` makes the element a compositing group blended onto its backdrop
// (compositing-1 §5). The non-separable modes (`hue`/`saturation`/`color`/
// `luminosity`) are not modelled yet.

#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_NORMAL.md")]
pub const MIX_BLEND_MODE_NORMAL: InternedSymbol = InternedSymbol::from_raw(0x0000_2501);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_MULTIPLY.md")]
pub const MIX_BLEND_MODE_MULTIPLY: InternedSymbol = InternedSymbol::from_raw(0x0000_2502);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_SCREEN.md")]
pub const MIX_BLEND_MODE_SCREEN: InternedSymbol = InternedSymbol::from_raw(0x0000_2503);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_OVERLAY.md")]
pub const MIX_BLEND_MODE_OVERLAY: InternedSymbol = InternedSymbol::from_raw(0x0000_2504);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_DARKEN.md")]
pub const MIX_BLEND_MODE_DARKEN: InternedSymbol = InternedSymbol::from_raw(0x0000_2505);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_LIGHTEN.md")]
pub const MIX_BLEND_MODE_LIGHTEN: InternedSymbol = InternedSymbol::from_raw(0x0000_2506);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_COLOR_DODGE.md")]
pub const MIX_BLEND_MODE_COLOR_DODGE: InternedSymbol = InternedSymbol::from_raw(0x0000_2507);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_COLOR_BURN.md")]
pub const MIX_BLEND_MODE_COLOR_BURN: InternedSymbol = InternedSymbol::from_raw(0x0000_2508);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_HARD_LIGHT.md")]
pub const MIX_BLEND_MODE_HARD_LIGHT: InternedSymbol = InternedSymbol::from_raw(0x0000_2509);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_SOFT_LIGHT.md")]
pub const MIX_BLEND_MODE_SOFT_LIGHT: InternedSymbol = InternedSymbol::from_raw(0x0000_250A);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_DIFFERENCE.md")]
pub const MIX_BLEND_MODE_DIFFERENCE: InternedSymbol = InternedSymbol::from_raw(0x0000_250B);
#[doc = include_str!("layout_keyword.doc/MIX_BLEND_MODE_EXCLUSION.md")]
pub const MIX_BLEND_MODE_EXCLUSION: InternedSymbol = InternedSymbol::from_raw(0x0000_250C);

#[doc = include_str!("layout_keyword.doc/mix_blend_mode_from_name.md")]
#[must_use]
pub fn mix_blend_mode_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "normal" => Some(MIX_BLEND_MODE_NORMAL),
        "multiply" => Some(MIX_BLEND_MODE_MULTIPLY),
        "screen" => Some(MIX_BLEND_MODE_SCREEN),
        "overlay" => Some(MIX_BLEND_MODE_OVERLAY),
        "darken" => Some(MIX_BLEND_MODE_DARKEN),
        "lighten" => Some(MIX_BLEND_MODE_LIGHTEN),
        "color-dodge" => Some(MIX_BLEND_MODE_COLOR_DODGE),
        "color-burn" => Some(MIX_BLEND_MODE_COLOR_BURN),
        "hard-light" => Some(MIX_BLEND_MODE_HARD_LIGHT),
        "soft-light" => Some(MIX_BLEND_MODE_SOFT_LIGHT),
        "difference" => Some(MIX_BLEND_MODE_DIFFERENCE),
        "exclusion" => Some(MIX_BLEND_MODE_EXCLUSION),
        _ => None,
    }
}

// ---- `pointer-events` keyword values ----
//
// Only the two values that mean something without SVG fill/stroke geometry.

#[doc = include_str!("layout_keyword.doc/POINTER_EVENTS_AUTO.md")]
pub const POINTER_EVENTS_AUTO: InternedSymbol = InternedSymbol::from_raw(0x0000_2701);
#[doc = include_str!("layout_keyword.doc/POINTER_EVENTS_NONE.md")]
pub const POINTER_EVENTS_NONE: InternedSymbol = InternedSymbol::from_raw(0x0000_2702);

#[doc = include_str!("layout_keyword.doc/pointer_events_from_name.md")]
#[must_use]
pub fn pointer_events_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "auto" => Some(POINTER_EVENTS_AUTO),
        "none" => Some(POINTER_EVENTS_NONE),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/font_weight_relative_from_name.md")]
#[must_use]
pub fn font_weight_relative_from_name(name: &str) -> Option<InternedSymbol> {
    match name.to_ascii_lowercase().as_str() {
        "bolder" => Some(FONT_WEIGHT_BOLDER),
        "lighter" => Some(FONT_WEIGHT_LIGHTER),
        _ => None,
    }
}

#[doc = include_str!("layout_keyword.doc/grid_auto_flow_from_name.md")]
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

#[doc = include_str!("layout_keyword.doc/name_of.md")]
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
        DISPLAY_CONTENTS => Some("contents"),
        POSITION_STATIC => Some("static"),
        POSITION_RELATIVE => Some("relative"),
        POSITION_ABSOLUTE => Some("absolute"),
        POSITION_FIXED => Some("fixed"),
        POSITION_STICKY => Some("sticky"),
        Z_INDEX_AUTO => Some("auto"),
        BORDER_STYLE_NONE => Some("none"),
        BORDER_STYLE_SOLID => Some("solid"),
        BORDER_STYLE_DASHED => Some("dashed"),
        BOX_SIZING_CONTENT_BOX => Some("content-box"),
        BOX_SIZING_BORDER_BOX => Some("border-box"),
        OVERFLOW_VISIBLE => Some("visible"),
        OVERFLOW_HIDDEN => Some("hidden"),
        OVERFLOW_CLIP => Some("clip"),
        OVERFLOW_SCROLL => Some("scroll"),
        OVERFLOW_AUTO => Some("auto"),
        FLEX_ROW => Some("row"),
        FLEX_ROW_REVERSE => Some("row-reverse"),
        FLEX_COLUMN => Some("column"),
        FLEX_COLUMN_REVERSE => Some("column-reverse"),
        FLEX_WRAP_NOWRAP => Some("nowrap"),
        FLEX_WRAP_WRAP => Some("wrap"),
        FLEX_WRAP_WRAP_REVERSE => Some("wrap-reverse"),
        JUSTIFY_START => Some("flex-start"),
        JUSTIFY_CENTER => Some("center"),
        JUSTIFY_END => Some("flex-end"),
        JUSTIFY_SPACE_BETWEEN => Some("space-between"),
        JUSTIFY_SPACE_AROUND => Some("space-around"),
        JUSTIFY_SPACE_EVENLY => Some("space-evenly"),
        JUSTIFY_STRETCH => Some("stretch"),
        ALIGN_START => Some("flex-start"),
        ALIGN_CENTER => Some("center"),
        ALIGN_END => Some("flex-end"),
        ALIGN_STRETCH => Some("stretch"),
        ALIGN_AUTO => Some("auto"),
        DIRECTION_LTR => Some("ltr"),
        DIRECTION_RTL => Some("rtl"),
        WRITING_MODE_HORIZONTAL_TB => Some("horizontal-tb"),
        WRITING_MODE_VERTICAL_RL => Some("vertical-rl"),
        WRITING_MODE_VERTICAL_LR => Some("vertical-lr"),
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
        TEXT_TRANSFORM_NONE => Some("none"),
        TEXT_TRANSFORM_UPPERCASE => Some("uppercase"),
        TEXT_TRANSFORM_LOWERCASE => Some("lowercase"),
        TEXT_TRANSFORM_CAPITALIZE => Some("capitalize"),
        FONT_WEIGHT_BOLDER => Some("bolder"),
        FONT_WEIGHT_LIGHTER => Some("lighter"),
        GRID_AUTO_FLOW_ROW => Some("row"),
        GRID_AUTO_FLOW_COLUMN => Some("column"),
        GRID_AUTO_FLOW_ROW_DENSE => Some("row dense"),
        GRID_AUTO_FLOW_COLUMN_DENSE => Some("column dense"),
        POINTER_EVENTS_AUTO => Some("auto"),
        POINTER_EVENTS_NONE => Some("none"),
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
        DISPLAY_CONTENTS,
        POSITION_STATIC,
        POSITION_RELATIVE,
        POSITION_ABSOLUTE,
        POSITION_FIXED,
        POSITION_STICKY,
        Z_INDEX_AUTO,
        BORDER_STYLE_NONE,
        BORDER_STYLE_SOLID,
        BORDER_STYLE_DASHED,
        BOX_SIZING_CONTENT_BOX,
        BOX_SIZING_BORDER_BOX,
        OVERFLOW_VISIBLE,
        OVERFLOW_HIDDEN,
        OVERFLOW_CLIP,
        OVERFLOW_SCROLL,
        OVERFLOW_AUTO,
        FLEX_ROW,
        FLEX_ROW_REVERSE,
        FLEX_COLUMN,
        FLEX_COLUMN_REVERSE,
        FLEX_WRAP_NOWRAP,
        FLEX_WRAP_WRAP,
        FLEX_WRAP_WRAP_REVERSE,
        JUSTIFY_START,
        JUSTIFY_CENTER,
        JUSTIFY_END,
        JUSTIFY_SPACE_BETWEEN,
        JUSTIFY_SPACE_AROUND,
        JUSTIFY_SPACE_EVENLY,
        JUSTIFY_STRETCH,
        ALIGN_START,
        ALIGN_CENTER,
        ALIGN_END,
        ALIGN_STRETCH,
        ALIGN_AUTO,
        DIRECTION_LTR,
        DIRECTION_RTL,
        WRITING_MODE_HORIZONTAL_TB,
        WRITING_MODE_VERTICAL_RL,
        WRITING_MODE_VERTICAL_LR,
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
        TEXT_TRANSFORM_NONE,
        TEXT_TRANSFORM_UPPERCASE,
        TEXT_TRANSFORM_LOWERCASE,
        TEXT_TRANSFORM_CAPITALIZE,
        FONT_WEIGHT_BOLDER,
        FONT_WEIGHT_LIGHTER,
        GRID_AUTO_FLOW_ROW,
        GRID_AUTO_FLOW_COLUMN,
        GRID_AUTO_FLOW_ROW_DENSE,
        GRID_AUTO_FLOW_COLUMN_DENSE,
        POINTER_EVENTS_AUTO,
        POINTER_EVENTS_NONE,
    ];

    #[test]
    fn pointer_events_maps_the_two_values_it_can_honour() {
        assert_eq!(pointer_events_from_name("auto"), Some(POINTER_EVENTS_AUTO));
        assert_eq!(pointer_events_from_name("none"), Some(POINTER_EVENTS_NONE));
        assert_eq!(pointer_events_from_name("NONE"), Some(POINTER_EVENTS_NONE));
        // The SVG-only values are not modelled, so they are not this keyword
        // set's business: they fall through as unknown.
        assert_eq!(pointer_events_from_name("visiblePainted"), None);
        assert_eq!(pointer_events_from_name("all"), None);
        assert_eq!(name_of(POINTER_EVENTS_NONE), Some("none"));
        assert_eq!(name_of(POINTER_EVENTS_AUTO), Some("auto"));
    }

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
    fn flex_wrap_maps_known_keywords() {
        assert_eq!(flex_wrap_from_name("nowrap"), Some(FLEX_WRAP_NOWRAP));
        assert_eq!(flex_wrap_from_name("wrap"), Some(FLEX_WRAP_WRAP));
        assert_eq!(
            flex_wrap_from_name("Wrap-Reverse"),
            Some(FLEX_WRAP_WRAP_REVERSE)
        );
        assert_eq!(flex_wrap_from_name("clip"), None);
        assert_eq!(name_of(FLEX_WRAP_NOWRAP), Some("nowrap"));
        assert_eq!(name_of(FLEX_WRAP_WRAP), Some("wrap"));
        assert_eq!(name_of(FLEX_WRAP_WRAP_REVERSE), Some("wrap-reverse"));
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
        // (An earlier `assert_eq!(align_content_from_name("stretch"), None)` was
        // removed 2026-07-20: `align-content: stretch` is now recognised and routes
        // to line-cross growth, per spec_ch4_flex::align_content_stretch_grows_lines.
        // It contradicted that oracle; the rest of this grammar check stands.)
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
        assert_eq!(
            text_overflow_from_name("ELLIPSIS"),
            Some(TEXT_OVERFLOW_ELLIPSIS)
        );
        assert_eq!(text_overflow_from_name("fade"), None);
        assert_eq!(name_of(TEXT_OVERFLOW_CLIP), Some("clip"));
        assert_eq!(name_of(TEXT_OVERFLOW_ELLIPSIS), Some("ellipsis"));
    }

    #[test]
    fn text_transform_maps_known_keywords() {
        assert_eq!(text_transform_from_name("none"), Some(TEXT_TRANSFORM_NONE));
        assert_eq!(
            text_transform_from_name("uppercase"),
            Some(TEXT_TRANSFORM_UPPERCASE)
        );
        assert_eq!(
            text_transform_from_name("LOWERCASE"),
            Some(TEXT_TRANSFORM_LOWERCASE)
        );
        assert_eq!(
            text_transform_from_name("capitalize"),
            Some(TEXT_TRANSFORM_CAPITALIZE)
        );
        assert_eq!(text_transform_from_name("full-width"), None);
        assert_eq!(name_of(TEXT_TRANSFORM_UPPERCASE), Some("uppercase"));
        assert_eq!(name_of(TEXT_TRANSFORM_CAPITALIZE), Some("capitalize"));
    }

    #[test]
    fn overflow_maps_known_keywords() {
        assert_eq!(overflow_from_name("visible"), Some(OVERFLOW_VISIBLE));
        assert_eq!(overflow_from_name("hidden"), Some(OVERFLOW_HIDDEN));
        assert_eq!(overflow_from_name("clip"), Some(OVERFLOW_CLIP));
        assert_eq!(overflow_from_name("scroll"), Some(OVERFLOW_SCROLL));
        assert_eq!(overflow_from_name("AUTO"), Some(OVERFLOW_AUTO));
        assert_eq!(overflow_from_name("overlay"), None);
        assert_eq!(name_of(OVERFLOW_VISIBLE), Some("visible"));
        assert_eq!(name_of(OVERFLOW_HIDDEN), Some("hidden"));
        assert_eq!(name_of(OVERFLOW_SCROLL), Some("scroll"));
        assert_eq!(name_of(OVERFLOW_AUTO), Some("auto"));
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
        assert_eq!(name_of(GRID_AUTO_FLOW_COLUMN_DENSE), Some("column dense"));
        assert_eq!(name_of(GRID_AUTO_FLOW_ROW), Some("row"));
    }

    #[test]
    fn box_sizing_maps_known_keywords() {
        assert_eq!(
            box_sizing_from_name("content-box"),
            Some(BOX_SIZING_CONTENT_BOX)
        );
        assert_eq!(
            box_sizing_from_name("border-box"),
            Some(BOX_SIZING_BORDER_BOX)
        );
        // Case-insensitive.
        assert_eq!(
            box_sizing_from_name("Border-Box"),
            Some(BOX_SIZING_BORDER_BOX)
        );
        assert_eq!(box_sizing_from_name("padding-box"), None);
        assert_eq!(name_of(BOX_SIZING_CONTENT_BOX), Some("content-box"));
        assert_eq!(name_of(BOX_SIZING_BORDER_BOX), Some("border-box"));
    }

    #[test]
    fn border_style_maps_known_keywords() {
        assert_eq!(border_style_from_name("none"), Some(BORDER_STYLE_NONE));
        assert_eq!(border_style_from_name("hidden"), Some(BORDER_STYLE_NONE));
        assert_eq!(border_style_from_name("solid"), Some(BORDER_STYLE_SOLID));
        // `dashed` / `dotted` map to the dashed handle (dotted folds to dashed).
        assert_eq!(border_style_from_name("dashed"), Some(BORDER_STYLE_DASHED));
        assert_eq!(border_style_from_name("DOTTED"), Some(BORDER_STYLE_DASHED));
        // The remaining continuous styles fold to `solid`.
        assert_eq!(border_style_from_name("double"), Some(BORDER_STYLE_SOLID));
        assert_eq!(border_style_from_name("wavy"), None);
        assert_eq!(name_of(BORDER_STYLE_NONE), Some("none"));
        assert_eq!(name_of(BORDER_STYLE_SOLID), Some("solid"));
        assert_eq!(name_of(BORDER_STYLE_DASHED), Some("dashed"));
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
