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
    /// The `opacity` property.
    Opacity,
    /// The `width` property.
    Width,
    /// The `height` property.
    Height,
    /// The `display` property.
    Display,
    /// The `visibility` property.
    Visibility,

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
    /// The `text-align` property (Keyword-valued: `left` / `right` / `center` /
    /// `justify` / `start` / `end`; inherited). See the reserved `TEXT_ALIGN_*`
    /// handles in [`crate::property::layout_keyword`].
    TextAlign,
    /// The `text-overflow` property (Keyword-valued: `clip` / `ellipsis`; NOT
    /// inherited). See the reserved `TEXT_OVERFLOW_*` handles in
    /// [`crate::property::layout_keyword`].
    TextOverflow,
}

impl PropertyId {
    /// Returns the property's canonical CSS name as a stable string.
    ///
    /// This is intended for diagnostics and `DevTools` only; it is never used for
    /// dispatch (that is what the enum variant is for).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::BackgroundColor => "background-color",
            Self::Opacity => "opacity",
            Self::Width => "width",
            Self::Height => "height",
            Self::Display => "display",
            Self::Visibility => "visibility",
            Self::MarginTop => "margin-top",
            Self::MarginRight => "margin-right",
            Self::MarginBottom => "margin-bottom",
            Self::MarginLeft => "margin-left",
            Self::PaddingTop => "padding-top",
            Self::PaddingRight => "padding-right",
            Self::PaddingBottom => "padding-bottom",
            Self::PaddingLeft => "padding-left",
            Self::MinWidth => "min-width",
            Self::MinHeight => "min-height",
            Self::MaxWidth => "max-width",
            Self::MaxHeight => "max-height",
            Self::Gap => "gap",
            Self::FlexBasis => "flex-basis",
            Self::FlexGrow => "flex-grow",
            Self::FlexShrink => "flex-shrink",
            Self::FlexDirection => "flex-direction",
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
            Self::TextAlign => "text-align",
            Self::TextOverflow => "text-overflow",
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
            | Self::TextAlign => Inheritance::Inherited,
            // Box / paint / layout properties reset on each element.
            Self::BackgroundColor
            | Self::Opacity
            | Self::Width
            | Self::Height
            | Self::Display
            | Self::MarginTop
            | Self::MarginRight
            | Self::MarginBottom
            | Self::MarginLeft
            | Self::PaddingTop
            | Self::PaddingRight
            | Self::PaddingBottom
            | Self::PaddingLeft
            | Self::MinWidth
            | Self::MinHeight
            | Self::MaxWidth
            | Self::MaxHeight
            | Self::Gap
            | Self::FlexBasis
            | Self::FlexGrow
            | Self::FlexShrink
            | Self::FlexDirection
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
            | Self::TextOverflow => Inheritance::NotInherited,
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
        assert_eq!(PropertyId::Opacity.name(), "opacity");
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
        assert_eq!(
            PropertyId::GridTemplateAreas.name(),
            "grid-template-areas"
        );
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
    fn is_hashable_key() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PropertyId::Width);
        set.insert(PropertyId::Width);
        set.insert(PropertyId::Height);
        assert_eq!(set.len(), 2);
    }
}
