//! Typed identifiers for known longhand properties.
//!
//! [`PropertyId`] is a *typed* enum, not a string: the cascade and computed-value
//! machinery dispatch on the variant, never on a property name. Names are kept
//! only as stable `&'static str`s for diagnostics and `DevTools`.
//!
//! The set of variants here is a deliberate Phase-1 starter set. The full CSS
//! longhand catalogue grows in this enum over time; adding a longhand means
//! adding a variant plus its rows in [`PropertyId::name`] and
//! [`PropertyId::default_inheritance`].

use crate::property::Inheritance;

/// A typed identifier for a known longhand CSS property.
///
/// Only longhands are represented; shorthands expand to longhands before the
/// cascade sees them. This is `Copy + Eq + Hash` so it can key maps in the
/// cascade and computed-style storage cheaply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyId {
    /// The `color` property (foreground / text color).
    Color,
    /// The `background-color` property.
    BackgroundColor,
    /// The `background-image` property (a `LinearGradient` value; NOT inherited).
    /// Only the `linear-gradient(...)` image form is modelled; when present it
    /// paints the element's background fill over the `background-color`.
    BackgroundImage,
    /// The `background-size` property (a `BackgroundSize` value; NOT inherited).
    /// Sizes one copy of the `background-image` (`cover` / `contain` / `auto` /
    /// explicit per-axis lengths); initial `auto`.
    BackgroundSize,
    /// The `background-position` property (a `BackgroundPosition` value; NOT
    /// inherited). Places the sized image tile within the box; initial `0% 0%`.
    BackgroundPosition,
    /// The `background-repeat` property (a `BackgroundRepeat` value; NOT
    /// inherited). Tiles the sized image tile to fill the box; initial `repeat`.
    BackgroundRepeat,
    /// The `object-fit` property (an `ObjectFit` value; NOT inherited). Sizes a
    /// replaced element's content (e.g. an `<img>`'s pixels) within its content
    /// box: `fill` (default) / `contain` / `cover` / `none` / `scale-down`.
    ObjectFit,
    /// The `opacity` property.
    Opacity,
    /// The `width` property.
    Width,
    /// The `height` property.
    Height,
    /// The `box-sizing` property (Keyword-valued: `content-box` (initial) /
    /// `border-box`; NOT inherited). Governs whether an explicit `width` /
    /// `height` sizes the content box or the border box. See the reserved
    /// `BOX_SIZING_*` handles in [`crate::property::layout_keyword`].
    BoxSizing,
    /// The `display` property.
    Display,
    /// The `visibility` property.
    Visibility,

    // ---- Positioning (not inherited) ----
    /// The `position` property (Keyword-valued: `static` / `relative` /
    /// `absolute` / `fixed`; NOT inherited). See the reserved `POSITION_*`
    /// handles in [`crate::property::layout_keyword`].
    Position,
    /// The `top` inset longhand (Length-valued, not inherited).
    Top,
    /// The `right` inset longhand (Length-valued, not inherited).
    Right,
    /// The `bottom` inset longhand (Length-valued, not inherited).
    Bottom,
    /// The `left` inset longhand (Length-valued, not inherited).
    Left,
    /// The `z-index` property (Number-valued `<integer>`, or the reserved
    /// [`Z_INDEX_AUTO`](crate::property::layout_keyword::Z_INDEX_AUTO) keyword for
    /// `auto`; NOT inherited). Only honored on positioned elements; drives the
    /// scene's stacking (paint) order.
    ZIndex,

    // ---- Box model: margins (Length-valued, not inherited) ----
    /// The `margin-top` longhand.
    MarginTop,
    /// The `margin-right` longhand.
    MarginRight,
    /// The `margin-bottom` longhand.
    MarginBottom,
    /// The `margin-left` longhand.
    MarginLeft,

    // ---- Box model: padding (Length-valued, not inherited) ----
    /// The `padding-top` longhand.
    PaddingTop,
    /// The `padding-right` longhand.
    PaddingRight,
    /// The `padding-bottom` longhand.
    PaddingBottom,
    /// The `padding-left` longhand.
    PaddingLeft,

    // ---- Logical box model (spec §4.13 "logical properties") ----
    //
    // These are NOT computed properties of their own: each resolves into its
    // physical counterpart at computed-value time, per `physical_under`. Keeping
    // them out of `COMPUTED_PROPERTIES` is deliberate — a node's computed style
    // holds `margin-left`, and every downstream phase reads that one place rather
    // than having to ask "or was it declared logically?".
    /// The `margin-inline-start` logical longhand.
    MarginInlineStart,
    /// The `margin-inline-end` logical longhand.
    MarginInlineEnd,
    /// The `margin-block-start` logical longhand.
    MarginBlockStart,
    /// The `margin-block-end` logical longhand.
    MarginBlockEnd,
    /// The `padding-inline-start` logical longhand.
    PaddingInlineStart,
    /// The `padding-inline-end` logical longhand.
    PaddingInlineEnd,
    /// The `padding-block-start` logical longhand.
    PaddingBlockStart,
    /// The `padding-block-end` logical longhand.
    PaddingBlockEnd,
    /// The `inset-inline-start` logical longhand.
    InsetInlineStart,
    /// The `inset-inline-end` logical longhand.
    InsetInlineEnd,
    /// The `inset-block-start` logical longhand.
    InsetBlockStart,
    /// The `inset-block-end` logical longhand.
    InsetBlockEnd,

    // ---- Box model: border width (Length-valued, not inherited) ----
    /// The `border-top-width` longhand. Feeds the box model's border insets and,
    /// paired with a non-`none` style, the painted border thickness. Initial `0`.
    BorderTopWidth,
    /// The `border-right-width` longhand.
    BorderRightWidth,
    /// The `border-bottom-width` longhand.
    BorderBottomWidth,
    /// The `border-left-width` longhand.
    BorderLeftWidth,

    // ---- Border style (Keyword-valued: `none` / `solid`; not inherited) ----
    /// The `border-top-style` longhand (Keyword-valued against the reserved
    /// `BORDER_STYLE_*` handles in [`crate::property::layout_keyword`]). Only
    /// `none` and `solid` are modelled; other line styles fold to `solid`.
    BorderTopStyle,
    /// The `border-right-style` longhand.
    BorderRightStyle,
    /// The `border-bottom-style` longhand.
    BorderBottomStyle,
    /// The `border-left-style` longhand.
    BorderLeftStyle,

    // ---- Border color (Color-valued, not inherited) ----
    /// The `border-top-color` longhand. When unset but a style + width are
    /// present, the painted border falls back to the computed `color`
    /// (CSS `currentColor`).
    BorderTopColor,
    /// The `border-right-color` longhand.
    BorderRightColor,
    /// The `border-bottom-color` longhand.
    BorderBottomColor,
    /// The `border-left-color` longhand.
    BorderLeftColor,

    // ---- Border radius (Length-valued, not inherited; initial `0`) ----
    /// The `border-top-left-radius` longhand. Rounds the top-left corner of the
    /// border box (and clips the content there); initial `0` (a square corner).
    BorderTopLeftRadius,
    /// The `border-top-right-radius` longhand.
    BorderTopRightRadius,
    /// The `border-bottom-right-radius` longhand.
    BorderBottomRightRadius,
    /// The `border-bottom-left-radius` longhand.
    BorderBottomLeftRadius,

    // ---- Outline (painted OUTSIDE the border box; NOT inherited; no layout) ----
    /// The `outline-width` longhand (Length-valued, not inherited; initial `0`).
    /// Paired with a non-`none` [`OutlineStyle`](Self::OutlineStyle) it is the
    /// thickness of the focus ring the scene coordinator strokes around — but
    /// outside — the border box; unlike a border it reserves no layout space.
    OutlineWidth,
    /// The `outline-style` longhand (Keyword-valued against the reserved
    /// `BORDER_STYLE_*` handles in [`crate::property::layout_keyword`]; not
    /// inherited). Only `none` / `solid` / `dashed` are modelled (`dotted` folds
    /// to dashed); `none` (the initial) paints no outline.
    OutlineStyle,
    /// The `outline-color` longhand (Color-valued, not inherited). When unset the
    /// painted outline falls back to the computed `color` (CSS `currentColor`).
    OutlineColor,
    /// The `outline-offset` longhand (Length-valued, not inherited; initial `0`).
    /// The gap between the border box and the inner edge of the outline; the
    /// outline is inflated outward by this amount before it is stroked.
    OutlineOffset,

    // ---- Box effects (not inherited) ----
    /// The `box-shadow` property (a `BoxShadow` list value; NOT inherited).
    /// `box-shadow: none` lowers to an empty list. The scene coordinator paints a
    /// drop shadow behind the box's background and an inset shadow on top of it.
    BoxShadow,
    /// The `filter` property (a `BoxShadow` list value carrying only the
    /// `drop-shadow(...)` functions; NOT inherited). Only the single
    /// GPU-expressible filter function, `drop-shadow(<ox> <oy> <blur>? <color>?)`,
    /// is modelled — it lowers to a drop [`BoxShadow`](crate::property::BoxShadow)
    /// (spread `0`, not inset) that the scene coordinator paints BEHIND the
    /// element like a `box-shadow` drop shadow. Non-drop-shadow functions
    /// (`blur()`, `brightness()`, …) are out of scope on vello 0.9 and are
    /// silently dropped; `filter: none` / empty lowers to an empty list.
    Filter,
    /// The `backdrop-filter` property (a `Filter` op list carrying only the
    /// `blur(<length>)` function; NOT inherited). Unlike [`Filter`](Self::Filter)
    /// — which blurs the element's OWN content — `backdrop-filter` blurs what is
    /// painted BEHIND the element (the backdrop), producing frosted-glass /
    /// Fluent acrylic. Only `blur(<length>)` is modelled; it lowers to a
    /// [`FilterOp::Blur`](crate::property::FilterOp) that the scene coordinator
    /// emits as a `BackdropFilter` command before the element's background, and the
    /// Vello renderer frosts the backdrop with a segmented gaussian pass.
    /// Color-matrix / other functions are out of scope (dropped); `none` /
    /// `blur(0)` lowers to an empty list (no effect).
    BackdropFilter,

    // ---- 2D transform (not inherited) ----
    /// The `transform` property (a resolved `Transform` affine; NOT inherited).
    /// `transform: none` lowers to an absent value. The scene coordinator
    /// brackets the element's whole subtree paint with the composed affine,
    /// resolved about [`TransformOrigin`](Self::TransformOrigin).
    Transform,
    /// The `transform-origin` property (a `TransformOrigin`; NOT inherited).
    /// Initial `50% 50%` (the box center).
    TransformOrigin,

    // ---- Sizing constraints (Length-valued, not inherited) ----
    /// The `min-width` property.
    MinWidth,
    /// The `min-height` property.
    MinHeight,
    /// The `max-width` property.
    MaxWidth,
    /// The `max-height` property.
    MaxHeight,

    // ---- Flex box (not inherited) ----
    /// The `gap` property (Length-valued).
    Gap,
    /// The `flex-basis` property (Length-valued).
    FlexBasis,
    /// The `flex-grow` property (Number-valued).
    FlexGrow,
    /// The `flex-shrink` property (Number-valued).
    FlexShrink,
    /// The `flex-direction` property (Keyword-valued; see
    /// [`crate::property::layout_keyword`]).
    FlexDirection,
    /// The `flex-wrap` property (Keyword-valued: `nowrap` (initial) / `wrap` /
    /// `wrap-reverse`; NOT inherited). Governs whether a flex container is
    /// single-line or wraps its items onto multiple cross-axis lines. See the
    /// reserved `FLEX_WRAP_*` handles in [`crate::property::layout_keyword`].
    FlexWrap,
    /// The `justify-content` property (Keyword-valued; see
    /// [`crate::property::layout_keyword`]). On a grid container this is the
    /// *inline-axis* (column) distribution of leftover track space.
    JustifyContent,
    /// The `align-content` property (Keyword-valued; see
    /// [`crate::property::layout_keyword`]). On a grid container this is the
    /// *block-axis* (row) distribution of leftover track space. NOT inherited.
    AlignContent,
    /// The `align-items` property (Keyword-valued; see
    /// [`crate::property::layout_keyword`]). On a grid container this is also the
    /// default *block-axis* alignment of each item within its cell.
    AlignItems,
    /// The `justify-items` property (Keyword-valued; the grid container's default
    /// *inline-axis* alignment of each item within its cell). See
    /// [`crate::property::layout_keyword`].
    JustifyItems,
    /// The `align-self` property (Keyword-valued; a grid item's *block-axis*
    /// alignment override, `auto` inheriting the container `align-items`). See
    /// [`crate::property::layout_keyword`].
    AlignSelf,
    /// The `justify-self` property (Keyword-valued; a grid item's *inline-axis*
    /// alignment override, `auto` inheriting the container `justify-items`). See
    /// [`crate::property::layout_keyword`].
    JustifySelf,

    // ---- Grid container (not inherited) ----
    /// The `grid-template-columns` property (a `GridTracks` track list).
    GridTemplateColumns,
    /// The `grid-template-rows` property (a `GridTracks` track list).
    GridTemplateRows,
    /// The `grid-template-areas` property (a `GridAreas` named-cell grid; NOT
    /// inherited). The runtime resolves a child's [`GridArea`](Self::GridArea)
    /// name against the container's parsed template.
    GridTemplateAreas,
    /// The `grid-area: <custom-ident>` named-area placement on a grid item (a
    /// `GridAreaName` marker; NOT inherited). Carried unresolved from CSS because
    /// the area rectangle lives on the item's *container* `grid-template-areas`,
    /// which the runtime — not the cascade — has the tree context to resolve. The
    /// numeric `grid-area: a / b / c / d` form does not use this id; it expands to
    /// the four `grid-*-start` / `-end` line longhands.
    GridArea,

    /// The `grid-auto-flow` property (Keyword-valued: `row` / `column` /
    /// `dense` combinations; NOT inherited). See the reserved `GRID_AUTO_FLOW_*`
    /// handles in [`crate::property::layout_keyword`].
    GridAutoFlow,
    /// The `grid-auto-columns` property (a `GridTracks` track list; NOT inherited).
    /// Sizes *implicit* column tracks — those created beyond the explicit
    /// `grid-template-columns` by placement / auto-flow. The list is cycled: the
    /// n-th implicit column uses index `n % len`. Initial is `auto`.
    GridAutoColumns,
    /// The `grid-auto-rows` property (a `GridTracks` track list; NOT inherited).
    /// Sizes *implicit* row tracks — those created beyond the explicit
    /// `grid-template-rows` by placement / auto-flow. The list is cycled: the n-th
    /// implicit row uses index `n % len`. Initial is `auto`.
    GridAutoRows,

    // ---- Grid item line placement (a `GridLine` edge; not inherited) ----
    /// The `grid-column-start` longhand (the `grid-column` start edge).
    GridColumnStart,
    /// The `grid-column-end` longhand (the `grid-column` end edge).
    GridColumnEnd,
    /// The `grid-row-start` longhand (the `grid-row` start edge).
    GridRowStart,
    /// The `grid-row-end` longhand (the `grid-row` end edge).
    GridRowEnd,

    // ---- Typography (inherited) ----
    /// The `font-size` property (Length-valued, inherited).
    FontSize,
    /// The `font-family` property (a `FontFamilies` list of family/generic
    /// names, inherited).
    FontFamily,
    /// The `direction` property (Keyword-valued: `ltr` / `rtl`; inherited). See
    /// the reserved `DIRECTION_*` handles in [`crate::property::layout_keyword`].
    Direction,
    /// The `font-weight` property (Number-valued on the `1..=1000` axis;
    /// inherited).
    FontWeight,
    /// The `font-style` property (Keyword-valued: `normal` / `italic` /
    /// `oblique`; inherited). See the reserved `FONT_STYLE_*` handles in
    /// [`crate::property::layout_keyword`].
    FontStyle,
    /// The `line-height` property (a unitless [`PropertyValue::Number`] multiple
    /// of `font-size`, or a `<length>`; inherited). The keyword initial `normal`
    /// is left unresolved so an absent value falls back to the font's natural
    /// line height. See [`crate::property::PropertyValue`].
    LineHeight,
    /// The `text-align` property (Keyword-valued: `left` / `right` / `center` /
    /// `justify` / `start` / `end`; inherited). See the reserved `TEXT_ALIGN_*`
    /// handles in [`crate::property::layout_keyword`].
    TextAlign,
    /// The `text-overflow` property (Keyword-valued: `clip` / `ellipsis`; NOT
    /// inherited). See the reserved `TEXT_OVERFLOW_*` handles in
    /// [`crate::property::layout_keyword`].
    TextOverflow,
    /// The `letter-spacing` property (a `<length>` [`PropertyValue::Length`], or
    /// the keyword `normal` ≡ `0`; inherited). Extra tracking added after each
    /// typographic character unit (CSS Text 3 §8.1). May be negative.
    LetterSpacing,
    /// The `word-spacing` property (a `<length>` [`PropertyValue::Length`], or the
    /// keyword `normal` ≡ `0`; inherited). Extra spacing added to each word
    /// separator (CSS Text 3 §8.2). May be negative.
    WordSpacing,
    /// The `text-transform` property (Keyword-valued: `none` / `uppercase` /
    /// `lowercase` / `capitalize`; inherited). Casing applied to the text before
    /// shaping (CSS Text 3 §2.1). See the reserved `TEXT_TRANSFORM_*` handles in
    /// [`crate::property::layout_keyword`].
    TextTransform,
    /// The `white-space` property (Keyword-valued: `normal` / `nowrap` / `pre` /
    /// `pre-wrap` / `pre-line`; inherited). Selects whitespace collapsing (applied
    /// before shaping) and the segment-break / soft-wrap behaviour of the line
    /// breaker (CSS Text 3 §3, §4.1.1). See the reserved `WHITE_SPACE_*` handles in
    /// [`crate::property::layout_keyword`].
    WhiteSpace,
    /// The `overflow-wrap` property (Keyword-valued: `normal` (initial) /
    /// `break-word` / `anywhere`; inherited). The legacy `word-wrap` name is an
    /// alias for it. Permits a last-resort grapheme-cluster break of an over-wide
    /// unbreakable word so it does not overflow its line (CSS Text 3 §5.2). See the
    /// reserved `OVERFLOW_WRAP_*` handles in [`crate::property::layout_keyword`].
    OverflowWrap,
    /// The `word-break` property (Keyword-valued: `normal` (initial) / `break-all`
    /// / `keep-all`; inherited). Selects the soft-wrap granularity: `break-all`
    /// breaks between any two typographic units, `keep-all` forbids breaking
    /// between CJK-context letters (CSS Text 3 §5.2). See the reserved
    /// `WORD_BREAK_*` handles in [`crate::property::layout_keyword`].
    WordBreak,
    /// The `tab-size` property (a unitless [`PropertyValue::Number`] multiple of a
    /// space advance, or a `<length>`; inherited; initial `8`). Sets the width of a
    /// preserved tab stop (CSS Text 3 §4.3).
    TabSize,

    // ---- Overflow (Keyword-valued; NOT inherited; pure paint/clip) ----
    /// The `overflow` property (Keyword-valued: `visible` (initial) / `hidden` /
    /// `clip` / `scroll` / `auto`; NOT inherited). See the reserved `OVERFLOW_*`
    /// handles in [`crate::property::layout_keyword`]. A value other than
    /// `visible` clips the element's subtree to its box; `scroll` / `auto` clip
    /// like `hidden` (no scrollbars are synthesised yet — a documented
    /// simplification). It changes no layout, only the scene coordinator's clip.
    Overflow,
}

impl PropertyId {
    /// The physical property this logical one resolves into, or [`None`] when
    /// `self` is not a logical property (spec §4.13).
    ///
    /// `rtl` is the element's computed `direction`. Only `direction` is consulted
    /// because only `direction` is modelled: there is no `writing-mode`, so the
    /// block axis is always vertical and `block-start`/`block-end` map to
    /// `top`/`bottom` regardless. The moment a `writing-mode` exists this is the
    /// one place that has to learn about it.
    ///
    /// The inline axis is the one that flips: under `rtl`, `inline-start` is the
    /// RIGHT edge.
    #[must_use]
    pub const fn physical_under(self, rtl: bool) -> Option<Self> {
        Some(match self {
            // Inline axis: flipped by `direction`.
            Self::MarginInlineStart => {
                if rtl {
                    Self::MarginRight
                } else {
                    Self::MarginLeft
                }
            }
            Self::MarginInlineEnd => {
                if rtl {
                    Self::MarginLeft
                } else {
                    Self::MarginRight
                }
            }
            Self::PaddingInlineStart => {
                if rtl {
                    Self::PaddingRight
                } else {
                    Self::PaddingLeft
                }
            }
            Self::PaddingInlineEnd => {
                if rtl {
                    Self::PaddingLeft
                } else {
                    Self::PaddingRight
                }
            }
            Self::InsetInlineStart => {
                if rtl {
                    Self::Right
                } else {
                    Self::Left
                }
            }
            Self::InsetInlineEnd => {
                if rtl {
                    Self::Left
                } else {
                    Self::Right
                }
            }
            // Block axis: vertical, and `direction` does not touch it.
            Self::MarginBlockStart => Self::MarginTop,
            Self::MarginBlockEnd => Self::MarginBottom,
            Self::PaddingBlockStart => Self::PaddingTop,
            Self::PaddingBlockEnd => Self::PaddingBottom,
            Self::InsetBlockStart => Self::Top,
            Self::InsetBlockEnd => Self::Bottom,
            _ => return None,
        })
    }

    /// Whether this is a logical property — one that resolves into a physical
    /// counterpart rather than being computed itself.
    #[must_use]
    pub const fn is_logical(self) -> bool {
        // `ltr` is arbitrary here: a logical property maps under either direction.
        self.physical_under(false).is_some()
    }

    /// Every logical property, in a stable order.
    ///
    /// Used to find which logical declarations could feed a given physical
    /// property; there are twelve, so a scan is cheaper than a reverse map.
    pub const LOGICAL: [Self; 12] = [
        Self::MarginInlineStart,
        Self::MarginInlineEnd,
        Self::MarginBlockStart,
        Self::MarginBlockEnd,
        Self::PaddingInlineStart,
        Self::PaddingInlineEnd,
        Self::PaddingBlockStart,
        Self::PaddingBlockEnd,
        Self::InsetInlineStart,
        Self::InsetInlineEnd,
        Self::InsetBlockStart,
        Self::InsetBlockEnd,
    ];

    /// Returns the property's canonical CSS name as a stable string.
    ///
    /// This is intended for diagnostics and `DevTools` only; it is never used for
    /// dispatch (that is what the enum variant is for).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::BackgroundColor => "background-color",
            Self::BackgroundImage => "background-image",
            Self::BackgroundSize => "background-size",
            Self::BackgroundPosition => "background-position",
            Self::BackgroundRepeat => "background-repeat",
            Self::ObjectFit => "object-fit",
            Self::Opacity => "opacity",
            Self::Width => "width",
            Self::Height => "height",
            Self::BoxSizing => "box-sizing",
            Self::Display => "display",
            Self::Visibility => "visibility",
            Self::Position => "position",
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::ZIndex => "z-index",
            Self::MarginTop => "margin-top",
            Self::MarginRight => "margin-right",
            Self::MarginBottom => "margin-bottom",
            Self::MarginLeft => "margin-left",
            Self::PaddingTop => "padding-top",
            Self::PaddingRight => "padding-right",
            Self::PaddingBottom => "padding-bottom",
            Self::PaddingLeft => "padding-left",
            Self::MarginInlineStart => "margin-inline-start",
            Self::MarginInlineEnd => "margin-inline-end",
            Self::MarginBlockStart => "margin-block-start",
            Self::MarginBlockEnd => "margin-block-end",
            Self::PaddingInlineStart => "padding-inline-start",
            Self::PaddingInlineEnd => "padding-inline-end",
            Self::PaddingBlockStart => "padding-block-start",
            Self::PaddingBlockEnd => "padding-block-end",
            Self::InsetInlineStart => "inset-inline-start",
            Self::InsetInlineEnd => "inset-inline-end",
            Self::InsetBlockStart => "inset-block-start",
            Self::InsetBlockEnd => "inset-block-end",
            Self::BorderTopWidth => "border-top-width",
            Self::BorderRightWidth => "border-right-width",
            Self::BorderBottomWidth => "border-bottom-width",
            Self::BorderLeftWidth => "border-left-width",
            Self::BorderTopStyle => "border-top-style",
            Self::BorderRightStyle => "border-right-style",
            Self::BorderBottomStyle => "border-bottom-style",
            Self::BorderLeftStyle => "border-left-style",
            Self::BorderTopColor => "border-top-color",
            Self::BorderRightColor => "border-right-color",
            Self::BorderBottomColor => "border-bottom-color",
            Self::BorderLeftColor => "border-left-color",
            Self::BorderTopLeftRadius => "border-top-left-radius",
            Self::BorderTopRightRadius => "border-top-right-radius",
            Self::BorderBottomRightRadius => "border-bottom-right-radius",
            Self::BorderBottomLeftRadius => "border-bottom-left-radius",
            Self::OutlineWidth => "outline-width",
            Self::OutlineStyle => "outline-style",
            Self::OutlineColor => "outline-color",
            Self::OutlineOffset => "outline-offset",
            Self::BoxShadow => "box-shadow",
            Self::Filter => "filter",
            Self::BackdropFilter => "backdrop-filter",
            Self::Transform => "transform",
            Self::TransformOrigin => "transform-origin",
            Self::MinWidth => "min-width",
            Self::MinHeight => "min-height",
            Self::MaxWidth => "max-width",
            Self::MaxHeight => "max-height",
            Self::Gap => "gap",
            Self::FlexBasis => "flex-basis",
            Self::FlexGrow => "flex-grow",
            Self::FlexShrink => "flex-shrink",
            Self::FlexDirection => "flex-direction",
            Self::FlexWrap => "flex-wrap",
            Self::JustifyContent => "justify-content",
            Self::AlignContent => "align-content",
            Self::AlignItems => "align-items",
            Self::JustifyItems => "justify-items",
            Self::AlignSelf => "align-self",
            Self::JustifySelf => "justify-self",
            Self::GridTemplateColumns => "grid-template-columns",
            Self::GridTemplateRows => "grid-template-rows",
            Self::GridTemplateAreas => "grid-template-areas",
            Self::GridArea => "grid-area",
            Self::GridAutoFlow => "grid-auto-flow",
            Self::GridAutoColumns => "grid-auto-columns",
            Self::GridAutoRows => "grid-auto-rows",
            Self::GridColumnStart => "grid-column-start",
            Self::GridColumnEnd => "grid-column-end",
            Self::GridRowStart => "grid-row-start",
            Self::GridRowEnd => "grid-row-end",
            Self::FontSize => "font-size",
            Self::FontFamily => "font-family",
            Self::Direction => "direction",
            Self::FontWeight => "font-weight",
            Self::FontStyle => "font-style",
            Self::LineHeight => "line-height",
            Self::TextAlign => "text-align",
            Self::TextOverflow => "text-overflow",
            Self::LetterSpacing => "letter-spacing",
            Self::WordSpacing => "word-spacing",
            Self::TextTransform => "text-transform",
            Self::WhiteSpace => "white-space",
            Self::OverflowWrap => "overflow-wrap",
            Self::WordBreak => "word-break",
            Self::TabSize => "tab-size",
            Self::Overflow => "overflow",
        }
    }

    /// Returns whether this property inherits from its parent by default (§4.14).
    ///
    /// This is the property's *default* behaviour; the CSS-wide keywords on
    /// individual declarations may override it per element during the cascade.
    #[must_use]
    pub const fn default_inheritance(self) -> Inheritance {
        match self {
            // Text-affecting / typography properties inherit by default.
            Self::Color
            | Self::Visibility
            | Self::FontSize
            | Self::FontFamily
            | Self::Direction
            | Self::FontWeight
            | Self::FontStyle
            | Self::LineHeight
            | Self::TextAlign
            // `letter-spacing`, `word-spacing`, and `text-transform` all inherit
            // (CSS Text 3 §8.1 / §8.2 / §2.1).
            | Self::LetterSpacing
            | Self::WordSpacing
            | Self::TextTransform
            // `white-space` and `tab-size` inherit (CSS Text 3 §3 / §4.3).
            | Self::WhiteSpace
            // `overflow-wrap` and `word-break` inherit (CSS Text 3 §5.2).
            | Self::OverflowWrap
            | Self::WordBreak
            | Self::TabSize => Inheritance::Inherited,
            // Box / paint / layout properties reset on each element.
            Self::BackgroundColor
            | Self::BackgroundImage
            | Self::BackgroundSize
            | Self::BackgroundPosition
            | Self::BackgroundRepeat
            | Self::ObjectFit
            | Self::Opacity
            | Self::Width
            | Self::Height
            | Self::BoxSizing
            | Self::Display
            | Self::Position
            | Self::Top
            | Self::Right
            | Self::Bottom
            | Self::Left
            | Self::ZIndex
            | Self::MarginTop
            | Self::MarginRight
            | Self::MarginBottom
            | Self::MarginLeft
            | Self::PaddingTop
            | Self::PaddingRight
            | Self::PaddingBottom
            | Self::PaddingLeft
            | Self::BorderTopWidth
            | Self::BorderRightWidth
            | Self::BorderBottomWidth
            | Self::BorderLeftWidth
            | Self::BorderTopStyle
            | Self::BorderRightStyle
            | Self::BorderBottomStyle
            | Self::BorderLeftStyle
            | Self::BorderTopColor
            | Self::BorderRightColor
            | Self::BorderBottomColor
            | Self::BorderLeftColor
            | Self::BorderTopLeftRadius
            | Self::BorderTopRightRadius
            | Self::BorderBottomRightRadius
            | Self::BorderBottomLeftRadius
            | Self::OutlineWidth
            | Self::OutlineStyle
            | Self::OutlineColor
            | Self::OutlineOffset
            | Self::BoxShadow
            | Self::Filter
            | Self::BackdropFilter
            | Self::Transform
            | Self::TransformOrigin
            | Self::MinWidth
            | Self::MinHeight
            | Self::MaxWidth
            | Self::MaxHeight
            // Logical box-model properties inherit exactly as their physical
            // counterparts do: not at all.
            | Self::MarginInlineStart
            | Self::MarginInlineEnd
            | Self::MarginBlockStart
            | Self::MarginBlockEnd
            | Self::PaddingInlineStart
            | Self::PaddingInlineEnd
            | Self::PaddingBlockStart
            | Self::PaddingBlockEnd
            | Self::InsetInlineStart
            | Self::InsetInlineEnd
            | Self::InsetBlockStart
            | Self::InsetBlockEnd
            | Self::Gap
            | Self::FlexBasis
            | Self::FlexGrow
            | Self::FlexShrink
            | Self::FlexDirection
            | Self::FlexWrap
            | Self::JustifyContent
            | Self::AlignContent
            | Self::AlignItems
            | Self::JustifyItems
            | Self::AlignSelf
            | Self::JustifySelf
            | Self::GridTemplateColumns
            | Self::GridTemplateRows
            | Self::GridTemplateAreas
            | Self::GridArea
            | Self::GridAutoFlow
            | Self::GridAutoColumns
            | Self::GridAutoRows
            | Self::GridColumnStart
            | Self::GridColumnEnd
            | Self::GridRowStart
            | Self::GridRowEnd
            // `text-overflow` does not inherit (unlike the other typography
            // properties, which do).
            | Self::TextOverflow
            // `overflow` is a box / paint property and resets per element.
            | Self::Overflow => Inheritance::NotInherited,
        }
    }

    /// Whether this property is **paint-only**: changing it can never change any
    /// node's box, so a frame in which only paint-only properties changed can
    /// reuse the retained layout (spec §3.4 phase 5, §4.3 — invalidation is
    /// classified by what actually changed, not by the frame's reason).
    ///
    /// This is a deliberately **conservative allow-list**: a property is
    /// paint-only only when it provably cannot feed measure/arrange. Everything
    /// else — including any property added later — answers `false` via the
    /// catch-all and is treated as layout-affecting, so the failure mode is a
    /// redundant relayout (slow but correct), never a stale box.
    ///
    /// Deliberately **not** paint-only despite looking like it:
    /// - `border-*-style`: `none` forces the used border width to zero, so it
    ///   feeds the box.
    /// - `text-align` / `text-overflow`: both are threaded into the text
    ///   measure pass here, so a change is treated as layout-affecting.
    #[must_use]
    pub const fn is_paint_only(self) -> bool {
        matches!(
            self,
            Self::Color
                | Self::BackgroundColor
                | Self::BackgroundImage
                | Self::BackgroundSize
                | Self::BackgroundPosition
                | Self::BackgroundRepeat
                // How an image fills its box; the box itself is unchanged.
                | Self::ObjectFit
                | Self::Opacity
                // `visibility: hidden` still occupies its box (unlike `display: none`).
                | Self::Visibility
                // Paint order only.
                | Self::ZIndex
                | Self::BorderTopColor
                | Self::BorderRightColor
                | Self::BorderBottomColor
                | Self::BorderLeftColor
                // A radius rounds the corners of an unchanged box.
                | Self::BorderTopLeftRadius
                | Self::BorderTopRightRadius
                | Self::BorderBottomRightRadius
                | Self::BorderBottomLeftRadius
                // The outline reserves no space (CSS UI 3).
                | Self::OutlineWidth
                | Self::OutlineStyle
                | Self::OutlineColor
                | Self::OutlineOffset
                | Self::BoxShadow
                | Self::Filter
                | Self::BackdropFilter
                // CSS transforms are applied at paint time; layout is unaffected.
                | Self::Transform
                | Self::TransformOrigin
        )
    }

    /// Whether a change to this property can change a node's box — the inverse of
    /// [`is_paint_only`](Self::is_paint_only). Conservative: any property not on
    /// the paint-only allow-list — including newly added ones — answers `true`.
    #[must_use]
    pub const fn affects_layout(self) -> bool {
        !self.is_paint_only()
    }
}

#[cfg(test)]
mod paint_only_tests {
    use super::*;

    #[test]
    fn typical_hover_properties_are_paint_only() {
        // The properties a hover/focus rule usually touches must be paint-only,
        // so such a frame can reuse the retained boxes.
        for id in [
            PropertyId::Color,
            PropertyId::BackgroundColor,
            PropertyId::BackgroundImage,
            PropertyId::BorderTopColor,
            PropertyId::BorderTopLeftRadius,
            PropertyId::BoxShadow,
            PropertyId::Opacity,
            PropertyId::Transform,
            PropertyId::Filter,
            PropertyId::OutlineWidth,
            PropertyId::OutlineColor,
            PropertyId::ZIndex,
            PropertyId::Visibility,
        ] {
            assert!(id.is_paint_only(), "{id:?} should be paint-only");
            assert!(!id.affects_layout(), "{id:?} must not force a relayout");
        }
    }

    #[test]
    fn box_and_text_properties_affect_layout() {
        for id in [
            PropertyId::Display,
            PropertyId::Width,
            PropertyId::Height,
            PropertyId::MinHeight,
            PropertyId::MarginTop,
            PropertyId::PaddingLeft,
            PropertyId::BorderTopWidth,
            PropertyId::BoxSizing,
            PropertyId::Position,
            PropertyId::Top,
            PropertyId::Gap,
            PropertyId::FlexGrow,
            PropertyId::AlignSelf,
            PropertyId::GridTemplateColumns,
            PropertyId::FontSize,
            PropertyId::FontFamily,
            PropertyId::LineHeight,
            PropertyId::LetterSpacing,
            PropertyId::WhiteSpace,
            PropertyId::Overflow,
        ] {
            assert!(id.affects_layout(), "{id:?} feeds layout");
            assert!(!id.is_paint_only());
        }
    }

    #[test]
    fn deliberately_conservative_cases_are_layout_affecting() {
        // `border-style: none` forces the used border width to zero, so it feeds
        // the box; `text-align`/`text-overflow` are threaded into the text
        // measure pass here. All three are conservatively layout-affecting.
        assert!(PropertyId::BorderTopStyle.affects_layout());
        assert!(PropertyId::TextAlign.affects_layout());
        assert!(PropertyId::TextOverflow.affects_layout());
    }

    #[test]
    fn the_two_classifiers_are_exact_inverses() {
        for id in [
            PropertyId::BackgroundColor,
            PropertyId::Transform,
            PropertyId::Width,
            PropertyId::FontSize,
            PropertyId::BorderTopStyle,
        ] {
            assert_eq!(id.affects_layout(), !id.is_paint_only());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_stable() {
        assert_eq!(PropertyId::Color.name(), "color");
        assert_eq!(PropertyId::BackgroundColor.name(), "background-color");
        assert_eq!(PropertyId::BackgroundImage.name(), "background-image");
        assert_eq!(PropertyId::Opacity.name(), "opacity");
    }

    #[test]
    fn background_image_name_and_not_inherited() {
        assert_eq!(PropertyId::BackgroundImage.name(), "background-image");
        // Like `background-color`, `background-image` resets per element.
        assert!(
            !PropertyId::BackgroundImage
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn background_size_position_repeat_names_and_not_inherited() {
        assert_eq!(PropertyId::BackgroundSize.name(), "background-size");
        assert_eq!(PropertyId::BackgroundPosition.name(), "background-position");
        assert_eq!(PropertyId::BackgroundRepeat.name(), "background-repeat");
        // Like the other background / paint properties, none inherit.
        for id in [
            PropertyId::BackgroundSize,
            PropertyId::BackgroundPosition,
            PropertyId::BackgroundRepeat,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn object_fit_name_and_not_inherited() {
        assert_eq!(PropertyId::ObjectFit.name(), "object-fit");
        // Like the other paint / box properties, `object-fit` resets per element.
        assert!(!PropertyId::ObjectFit.default_inheritance().is_inherited());
    }

    #[test]
    fn default_inheritance_matches_css() {
        assert!(PropertyId::Color.default_inheritance().is_inherited());
        assert!(PropertyId::Visibility.default_inheritance().is_inherited());
        assert!(!PropertyId::Opacity.default_inheritance().is_inherited());
        assert!(
            !PropertyId::BackgroundColor
                .default_inheritance()
                .is_inherited()
        );
        assert!(!PropertyId::Display.default_inheritance().is_inherited());
    }

    #[test]
    fn box_model_and_flex_names_are_canonical() {
        assert_eq!(PropertyId::MarginTop.name(), "margin-top");
        assert_eq!(PropertyId::MarginRight.name(), "margin-right");
        assert_eq!(PropertyId::MarginBottom.name(), "margin-bottom");
        assert_eq!(PropertyId::MarginLeft.name(), "margin-left");
        assert_eq!(PropertyId::PaddingTop.name(), "padding-top");
        assert_eq!(PropertyId::PaddingRight.name(), "padding-right");
        assert_eq!(PropertyId::PaddingBottom.name(), "padding-bottom");
        assert_eq!(PropertyId::PaddingLeft.name(), "padding-left");
        assert_eq!(PropertyId::MinWidth.name(), "min-width");
        assert_eq!(PropertyId::MinHeight.name(), "min-height");
        assert_eq!(PropertyId::MaxWidth.name(), "max-width");
        assert_eq!(PropertyId::MaxHeight.name(), "max-height");
        assert_eq!(PropertyId::Gap.name(), "gap");
        assert_eq!(PropertyId::FlexBasis.name(), "flex-basis");
        assert_eq!(PropertyId::FlexGrow.name(), "flex-grow");
        assert_eq!(PropertyId::FlexShrink.name(), "flex-shrink");
        assert_eq!(PropertyId::FlexDirection.name(), "flex-direction");
        assert_eq!(PropertyId::FlexWrap.name(), "flex-wrap");
        assert_eq!(PropertyId::JustifyContent.name(), "justify-content");
        assert_eq!(PropertyId::AlignContent.name(), "align-content");
        assert_eq!(PropertyId::AlignItems.name(), "align-items");
        assert_eq!(PropertyId::FontSize.name(), "font-size");
    }

    #[test]
    fn align_content_name_and_not_inherited() {
        assert_eq!(PropertyId::AlignContent.name(), "align-content");
        // Like the other layout / box properties, `align-content` resets per
        // element rather than inheriting.
        assert!(
            !PropertyId::AlignContent
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn font_size_inherits_layout_does_not() {
        // Typography inherits, like `color`.
        assert!(PropertyId::FontSize.default_inheritance().is_inherited());
        // Box-model / flex properties reset on each element.
        for id in [
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
            PropertyId::MinWidth,
            PropertyId::MinHeight,
            PropertyId::MaxWidth,
            PropertyId::MaxHeight,
            PropertyId::Gap,
            PropertyId::FlexBasis,
            PropertyId::FlexGrow,
            PropertyId::FlexShrink,
            PropertyId::FlexDirection,
            PropertyId::FlexWrap,
            PropertyId::JustifyContent,
            PropertyId::AlignItems,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn font_family_name_and_inheritance() {
        assert_eq!(PropertyId::FontFamily.name(), "font-family");
        // Like `font-size` and `color`, `font-family` inherits by default.
        assert!(PropertyId::FontFamily.default_inheritance().is_inherited());
    }

    #[test]
    fn direction_weight_style_names_and_inheritance() {
        assert_eq!(PropertyId::Direction.name(), "direction");
        assert_eq!(PropertyId::FontWeight.name(), "font-weight");
        assert_eq!(PropertyId::FontStyle.name(), "font-style");
        // Like the other typography properties, all three inherit by default.
        assert!(PropertyId::Direction.default_inheritance().is_inherited());
        assert!(PropertyId::FontWeight.default_inheritance().is_inherited());
        assert!(PropertyId::FontStyle.default_inheritance().is_inherited());
    }

    #[test]
    fn text_align_name_and_inheritance() {
        assert_eq!(PropertyId::TextAlign.name(), "text-align");
        // Like the other typography properties, `text-align` inherits by default.
        assert!(PropertyId::TextAlign.default_inheritance().is_inherited());
    }

    #[test]
    fn text_overflow_name_and_not_inherited() {
        assert_eq!(PropertyId::TextOverflow.name(), "text-overflow");
        // Unlike the other typography properties, `text-overflow` does not inherit.
        assert!(
            !PropertyId::TextOverflow
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn line_height_name_and_inherited() {
        assert_eq!(PropertyId::LineHeight.name(), "line-height");
        // Like `font-size` and the other typography properties, `line-height`
        // inherits by default.
        assert!(PropertyId::LineHeight.default_inheritance().is_inherited());
    }

    #[test]
    fn overflow_name_and_not_inherited() {
        assert_eq!(PropertyId::Overflow.name(), "overflow");
        // Like the other box / paint properties, `overflow` resets per element.
        assert!(!PropertyId::Overflow.default_inheritance().is_inherited());
    }

    #[test]
    fn grid_alignment_names_and_not_inherited() {
        assert_eq!(PropertyId::JustifyItems.name(), "justify-items");
        assert_eq!(PropertyId::AlignSelf.name(), "align-self");
        assert_eq!(PropertyId::JustifySelf.name(), "justify-self");
        // Like the other layout / box properties, the alignment properties reset
        // per element rather than inheriting.
        for id in [
            PropertyId::JustifyItems,
            PropertyId::AlignSelf,
            PropertyId::JustifySelf,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn grid_template_names_and_not_inherited() {
        assert_eq!(
            PropertyId::GridTemplateColumns.name(),
            "grid-template-columns"
        );
        assert_eq!(PropertyId::GridTemplateRows.name(), "grid-template-rows");
        // Like the other layout / box properties, the grid templates reset per
        // element rather than inheriting.
        assert!(
            !PropertyId::GridTemplateColumns
                .default_inheritance()
                .is_inherited()
        );
        assert!(
            !PropertyId::GridTemplateRows
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn grid_areas_names_and_not_inherited() {
        assert_eq!(PropertyId::GridTemplateAreas.name(), "grid-template-areas");
        assert_eq!(PropertyId::GridArea.name(), "grid-area");
        assert!(
            !PropertyId::GridTemplateAreas
                .default_inheritance()
                .is_inherited()
        );
        assert!(!PropertyId::GridArea.default_inheritance().is_inherited());
    }

    #[test]
    fn grid_auto_flow_name_and_not_inherited() {
        assert_eq!(PropertyId::GridAutoFlow.name(), "grid-auto-flow");
        // Like the other grid / layout properties, `grid-auto-flow` resets per
        // element rather than inheriting.
        assert!(
            !PropertyId::GridAutoFlow
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn grid_auto_tracks_names_and_not_inherited() {
        assert_eq!(PropertyId::GridAutoColumns.name(), "grid-auto-columns");
        assert_eq!(PropertyId::GridAutoRows.name(), "grid-auto-rows");
        // Like the other grid container properties, the implicit-track sizing
        // properties reset per element rather than inheriting.
        for id in [PropertyId::GridAutoColumns, PropertyId::GridAutoRows] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn grid_line_names_and_not_inherited() {
        assert_eq!(PropertyId::GridColumnStart.name(), "grid-column-start");
        assert_eq!(PropertyId::GridColumnEnd.name(), "grid-column-end");
        assert_eq!(PropertyId::GridRowStart.name(), "grid-row-start");
        assert_eq!(PropertyId::GridRowEnd.name(), "grid-row-end");
        // Like the other layout / box properties, line placement resets per element.
        for id in [
            PropertyId::GridColumnStart,
            PropertyId::GridColumnEnd,
            PropertyId::GridRowStart,
            PropertyId::GridRowEnd,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn position_and_inset_names_and_not_inherited() {
        assert_eq!(PropertyId::Position.name(), "position");
        assert_eq!(PropertyId::Top.name(), "top");
        assert_eq!(PropertyId::Right.name(), "right");
        assert_eq!(PropertyId::Bottom.name(), "bottom");
        assert_eq!(PropertyId::Left.name(), "left");
        // Like the other layout / box properties, positioning resets per element.
        for id in [
            PropertyId::Position,
            PropertyId::Top,
            PropertyId::Right,
            PropertyId::Bottom,
            PropertyId::Left,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn z_index_name_and_not_inherited() {
        assert_eq!(PropertyId::ZIndex.name(), "z-index");
        // Like the other positioning / layout properties, `z-index` resets per
        // element rather than inheriting.
        assert!(!PropertyId::ZIndex.default_inheritance().is_inherited());
    }

    #[test]
    fn border_names_and_not_inherited() {
        assert_eq!(PropertyId::BorderTopWidth.name(), "border-top-width");
        assert_eq!(PropertyId::BorderRightWidth.name(), "border-right-width");
        assert_eq!(PropertyId::BorderBottomWidth.name(), "border-bottom-width");
        assert_eq!(PropertyId::BorderLeftWidth.name(), "border-left-width");
        assert_eq!(PropertyId::BorderTopStyle.name(), "border-top-style");
        assert_eq!(PropertyId::BorderRightStyle.name(), "border-right-style");
        assert_eq!(PropertyId::BorderBottomStyle.name(), "border-bottom-style");
        assert_eq!(PropertyId::BorderLeftStyle.name(), "border-left-style");
        assert_eq!(PropertyId::BorderTopColor.name(), "border-top-color");
        assert_eq!(PropertyId::BorderRightColor.name(), "border-right-color");
        assert_eq!(PropertyId::BorderBottomColor.name(), "border-bottom-color");
        assert_eq!(PropertyId::BorderLeftColor.name(), "border-left-color");
        // Like the other box / paint properties, borders reset per element.
        for id in [
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
            PropertyId::BorderTopColor,
            PropertyId::BorderRightColor,
            PropertyId::BorderBottomColor,
            PropertyId::BorderLeftColor,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn border_radius_names_and_not_inherited() {
        assert_eq!(
            PropertyId::BorderTopLeftRadius.name(),
            "border-top-left-radius"
        );
        assert_eq!(
            PropertyId::BorderTopRightRadius.name(),
            "border-top-right-radius"
        );
        assert_eq!(
            PropertyId::BorderBottomRightRadius.name(),
            "border-bottom-right-radius"
        );
        assert_eq!(
            PropertyId::BorderBottomLeftRadius.name(),
            "border-bottom-left-radius"
        );
        // Like the other box / paint properties, corner radii reset per element.
        for id in [
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn outline_names_and_not_inherited() {
        assert_eq!(PropertyId::OutlineWidth.name(), "outline-width");
        assert_eq!(PropertyId::OutlineStyle.name(), "outline-style");
        assert_eq!(PropertyId::OutlineColor.name(), "outline-color");
        assert_eq!(PropertyId::OutlineOffset.name(), "outline-offset");
        // Like borders, the outline properties reset per element (no layout).
        for id in [
            PropertyId::OutlineWidth,
            PropertyId::OutlineStyle,
            PropertyId::OutlineColor,
            PropertyId::OutlineOffset,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
    }

    #[test]
    fn box_shadow_name_and_not_inherited() {
        assert_eq!(PropertyId::BoxShadow.name(), "box-shadow");
        assert!(!PropertyId::BoxShadow.default_inheritance().is_inherited());
    }

    #[test]
    fn filter_name_and_not_inherited() {
        assert_eq!(PropertyId::Filter.name(), "filter");
        // Like `box-shadow` and the other paint effects, `filter` resets per
        // element rather than inheriting.
        assert!(!PropertyId::Filter.default_inheritance().is_inherited());
    }

    #[test]
    fn backdrop_filter_name_and_not_inherited() {
        assert_eq!(PropertyId::BackdropFilter.name(), "backdrop-filter");
        // Like `filter` and the other paint effects, `backdrop-filter` resets per
        // element rather than inheriting.
        assert!(
            !PropertyId::BackdropFilter
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn transform_names_and_not_inherited() {
        assert_eq!(PropertyId::Transform.name(), "transform");
        assert_eq!(PropertyId::TransformOrigin.name(), "transform-origin");
        // Like the other box / paint properties, transforms reset per element.
        assert!(!PropertyId::Transform.default_inheritance().is_inherited());
        assert!(
            !PropertyId::TransformOrigin
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn box_sizing_name_and_not_inherited() {
        assert_eq!(PropertyId::BoxSizing.name(), "box-sizing");
        // Like the other box-model properties, `box-sizing` resets per element.
        assert!(!PropertyId::BoxSizing.default_inheritance().is_inherited());
    }

    #[test]
    fn is_hashable_key() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PropertyId::Width);
        set.insert(PropertyId::Width);
        set.insert(PropertyId::Height);
        assert_eq!(set.len(), 2);
    }
}
