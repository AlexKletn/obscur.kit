#![doc = include_str!("property_id.doc/_module.md")]

use crate::property::Inheritance;

#[doc = include_str!("property_id.doc/PropertyId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyId {
    #[doc = include_str!("property_id.doc/Color.md")]
    Color,
    #[doc = include_str!("property_id.doc/BackgroundColor.md")]
    BackgroundColor,
    #[doc = include_str!("property_id.doc/BackgroundImage.md")]
    BackgroundImage,
    #[doc = include_str!("property_id.doc/BackgroundSize.md")]
    BackgroundSize,
    #[doc = include_str!("property_id.doc/BackgroundPosition.md")]
    BackgroundPosition,
    #[doc = include_str!("property_id.doc/BackgroundRepeat.md")]
    BackgroundRepeat,
    #[doc = include_str!("property_id.doc/ObjectFit.md")]
    ObjectFit,
    #[doc = include_str!("property_id.doc/ObjectPosition.md")]
    ObjectPosition,
    #[doc = include_str!("property_id.doc/Opacity.md")]
    Opacity,
    #[doc = include_str!("property_id.doc/Width.md")]
    Width,
    #[doc = include_str!("property_id.doc/Height.md")]
    Height,
    #[doc = include_str!("property_id.doc/BoxSizing.md")]
    BoxSizing,
    #[doc = include_str!("property_id.doc/Display.md")]
    Display,
    #[doc = include_str!("property_id.doc/Visibility.md")]
    Visibility,

    // ---- Positioning (not inherited) ----
    #[doc = include_str!("property_id.doc/Position.md")]
    Position,
    #[doc = include_str!("property_id.doc/Top.md")]
    Top,
    #[doc = include_str!("property_id.doc/Right.md")]
    Right,
    #[doc = include_str!("property_id.doc/Bottom.md")]
    Bottom,
    #[doc = include_str!("property_id.doc/Left.md")]
    Left,
    #[doc = include_str!("property_id.doc/ZIndex.md")]
    ZIndex,

    // ---- Box model: margins (Length-valued, not inherited) ----
    #[doc = include_str!("property_id.doc/MarginTop.md")]
    MarginTop,
    #[doc = include_str!("property_id.doc/MarginRight.md")]
    MarginRight,
    #[doc = include_str!("property_id.doc/MarginBottom.md")]
    MarginBottom,
    #[doc = include_str!("property_id.doc/MarginLeft.md")]
    MarginLeft,

    // ---- Box model: padding (Length-valued, not inherited) ----
    #[doc = include_str!("property_id.doc/PaddingTop.md")]
    PaddingTop,
    #[doc = include_str!("property_id.doc/PaddingRight.md")]
    PaddingRight,
    #[doc = include_str!("property_id.doc/PaddingBottom.md")]
    PaddingBottom,
    #[doc = include_str!("property_id.doc/PaddingLeft.md")]
    PaddingLeft,

    // ---- Logical box model (spec §4.13 "logical properties") ----
    //
    // These are NOT computed properties of their own: each resolves into its
    // physical counterpart at computed-value time, per `physical_under`. Keeping
    // them out of `COMPUTED_PROPERTIES` is deliberate — a node's computed style
    // holds `margin-left`, and every downstream phase reads that one place rather
    // than having to ask "or was it declared logically?".
    #[doc = include_str!("property_id.doc/MarginInlineStart.md")]
    MarginInlineStart,
    #[doc = include_str!("property_id.doc/MarginInlineEnd.md")]
    MarginInlineEnd,
    #[doc = include_str!("property_id.doc/MarginBlockStart.md")]
    MarginBlockStart,
    #[doc = include_str!("property_id.doc/MarginBlockEnd.md")]
    MarginBlockEnd,
    #[doc = include_str!("property_id.doc/PaddingInlineStart.md")]
    PaddingInlineStart,
    #[doc = include_str!("property_id.doc/PaddingInlineEnd.md")]
    PaddingInlineEnd,
    #[doc = include_str!("property_id.doc/PaddingBlockStart.md")]
    PaddingBlockStart,
    #[doc = include_str!("property_id.doc/PaddingBlockEnd.md")]
    PaddingBlockEnd,
    #[doc = include_str!("property_id.doc/InsetInlineStart.md")]
    InsetInlineStart,
    #[doc = include_str!("property_id.doc/InsetInlineEnd.md")]
    InsetInlineEnd,
    #[doc = include_str!("property_id.doc/InsetBlockStart.md")]
    InsetBlockStart,
    #[doc = include_str!("property_id.doc/InsetBlockEnd.md")]
    InsetBlockEnd,
    #[doc = include_str!("property_id.doc/BorderInlineStartWidth.md")]
    BorderInlineStartWidth,
    #[doc = include_str!("property_id.doc/BorderInlineEndWidth.md")]
    BorderInlineEndWidth,
    #[doc = include_str!("property_id.doc/BorderBlockStartWidth.md")]
    BorderBlockStartWidth,
    #[doc = include_str!("property_id.doc/BorderBlockEndWidth.md")]
    BorderBlockEndWidth,
    #[doc = include_str!("property_id.doc/BorderInlineStartStyle.md")]
    BorderInlineStartStyle,
    #[doc = include_str!("property_id.doc/BorderInlineEndStyle.md")]
    BorderInlineEndStyle,
    #[doc = include_str!("property_id.doc/BorderBlockStartStyle.md")]
    BorderBlockStartStyle,
    #[doc = include_str!("property_id.doc/BorderBlockEndStyle.md")]
    BorderBlockEndStyle,

    // ---- Box model: border width (Length-valued, not inherited) ----
    #[doc = include_str!("property_id.doc/BorderTopWidth.md")]
    BorderTopWidth,
    #[doc = include_str!("property_id.doc/BorderRightWidth.md")]
    BorderRightWidth,
    #[doc = include_str!("property_id.doc/BorderBottomWidth.md")]
    BorderBottomWidth,
    #[doc = include_str!("property_id.doc/BorderLeftWidth.md")]
    BorderLeftWidth,

    // ---- Border style (Keyword-valued: `none` / `solid`; not inherited) ----
    #[doc = include_str!("property_id.doc/BorderTopStyle.md")]
    BorderTopStyle,
    #[doc = include_str!("property_id.doc/BorderRightStyle.md")]
    BorderRightStyle,
    #[doc = include_str!("property_id.doc/BorderBottomStyle.md")]
    BorderBottomStyle,
    #[doc = include_str!("property_id.doc/BorderLeftStyle.md")]
    BorderLeftStyle,

    // ---- Border color (Color-valued, not inherited) ----
    #[doc = include_str!("property_id.doc/BorderTopColor.md")]
    BorderTopColor,
    #[doc = include_str!("property_id.doc/BorderRightColor.md")]
    BorderRightColor,
    #[doc = include_str!("property_id.doc/BorderBottomColor.md")]
    BorderBottomColor,
    #[doc = include_str!("property_id.doc/BorderLeftColor.md")]
    BorderLeftColor,

    // ---- Border radius (Length-valued, not inherited; initial `0`) ----
    #[doc = include_str!("property_id.doc/BorderTopLeftRadius.md")]
    BorderTopLeftRadius,
    #[doc = include_str!("property_id.doc/BorderTopRightRadius.md")]
    BorderTopRightRadius,
    #[doc = include_str!("property_id.doc/BorderBottomRightRadius.md")]
    BorderBottomRightRadius,
    #[doc = include_str!("property_id.doc/BorderBottomLeftRadius.md")]
    BorderBottomLeftRadius,

    // ---- Outline (painted OUTSIDE the border box; NOT inherited; no layout) ----
    #[doc = include_str!("property_id.doc/OutlineWidth.md")]
    OutlineWidth,
    #[doc = include_str!("property_id.doc/OutlineStyle.md")]
    OutlineStyle,
    #[doc = include_str!("property_id.doc/OutlineColor.md")]
    OutlineColor,
    #[doc = include_str!("property_id.doc/OutlineOffset.md")]
    OutlineOffset,

    // ---- Box effects (not inherited) ----
    #[doc = include_str!("property_id.doc/BoxShadow.md")]
    BoxShadow,
    #[doc = include_str!("property_id.doc/Filter.md")]
    Filter,
    #[doc = include_str!("property_id.doc/BackdropFilter.md")]
    BackdropFilter,

    // ---- 2D transform (not inherited) ----
    #[doc = include_str!("property_id.doc/Transform.md")]
    Transform,
    #[doc = include_str!("property_id.doc/TransformOrigin.md")]
    TransformOrigin,

    #[doc = include_str!("property_id.doc/MixBlendMode.md")]
    MixBlendMode,

    // ---- Sizing constraints (Length-valued, not inherited) ----
    #[doc = include_str!("property_id.doc/MinWidth.md")]
    MinWidth,
    #[doc = include_str!("property_id.doc/MinHeight.md")]
    MinHeight,
    #[doc = include_str!("property_id.doc/MaxWidth.md")]
    MaxWidth,
    #[doc = include_str!("property_id.doc/MaxHeight.md")]
    MaxHeight,

    // ---- Flex box (not inherited) ----
    #[doc = include_str!("property_id.doc/Gap.md")]
    Gap,
    #[doc = include_str!("property_id.doc/FlexBasis.md")]
    FlexBasis,
    #[doc = include_str!("property_id.doc/FlexGrow.md")]
    FlexGrow,
    #[doc = include_str!("property_id.doc/FlexShrink.md")]
    FlexShrink,
    #[doc = include_str!("property_id.doc/FlexDirection.md")]
    FlexDirection,
    #[doc = include_str!("property_id.doc/FlexWrap.md")]
    FlexWrap,
    #[doc = include_str!("property_id.doc/JustifyContent.md")]
    JustifyContent,
    #[doc = include_str!("property_id.doc/AlignContent.md")]
    AlignContent,
    #[doc = include_str!("property_id.doc/AlignItems.md")]
    AlignItems,
    #[doc = include_str!("property_id.doc/JustifyItems.md")]
    JustifyItems,
    #[doc = include_str!("property_id.doc/AlignSelf.md")]
    AlignSelf,
    #[doc = include_str!("property_id.doc/JustifySelf.md")]
    JustifySelf,

    // ---- Grid container (not inherited) ----
    #[doc = include_str!("property_id.doc/GridTemplateColumns.md")]
    GridTemplateColumns,
    #[doc = include_str!("property_id.doc/GridTemplateRows.md")]
    GridTemplateRows,
    #[doc = include_str!("property_id.doc/GridTemplateAreas.md")]
    GridTemplateAreas,
    #[doc = include_str!("property_id.doc/GridArea.md")]
    GridArea,

    #[doc = include_str!("property_id.doc/GridAutoFlow.md")]
    GridAutoFlow,
    #[doc = include_str!("property_id.doc/GridAutoColumns.md")]
    GridAutoColumns,
    #[doc = include_str!("property_id.doc/GridAutoRows.md")]
    GridAutoRows,

    // ---- Grid item line placement (a `GridLine` edge; not inherited) ----
    #[doc = include_str!("property_id.doc/GridColumnStart.md")]
    GridColumnStart,
    #[doc = include_str!("property_id.doc/GridColumnEnd.md")]
    GridColumnEnd,
    #[doc = include_str!("property_id.doc/GridRowStart.md")]
    GridRowStart,
    #[doc = include_str!("property_id.doc/GridRowEnd.md")]
    GridRowEnd,

    // ---- Typography (inherited) ----
    #[doc = include_str!("property_id.doc/FontSize.md")]
    FontSize,
    #[doc = include_str!("property_id.doc/FontFamily.md")]
    FontFamily,
    #[doc = include_str!("property_id.doc/Direction.md")]
    Direction,
    #[doc = include_str!("property_id.doc/WritingMode.md")]
    WritingMode,
    #[doc = include_str!("property_id.doc/FontWeight.md")]
    FontWeight,
    #[doc = include_str!("property_id.doc/FontStyle.md")]
    FontStyle,
    #[doc = include_str!("property_id.doc/LineHeight.md")]
    LineHeight,
    #[doc = include_str!("property_id.doc/TextAlign.md")]
    TextAlign,
    #[doc = include_str!("property_id.doc/TextOverflow.md")]
    TextOverflow,
    #[doc = include_str!("property_id.doc/LetterSpacing.md")]
    LetterSpacing,
    #[doc = include_str!("property_id.doc/WordSpacing.md")]
    WordSpacing,
    #[doc = include_str!("property_id.doc/TextTransform.md")]
    TextTransform,
    #[doc = include_str!("property_id.doc/WhiteSpace.md")]
    WhiteSpace,
    #[doc = include_str!("property_id.doc/OverflowWrap.md")]
    OverflowWrap,
    #[doc = include_str!("property_id.doc/WordBreak.md")]
    WordBreak,
    #[doc = include_str!("property_id.doc/TabSize.md")]
    TabSize,

    // ---- Overflow (Keyword-valued; NOT inherited) ----
    //
    // The two axis longhands ARE the storage: `overflow` is a shorthand that the
    // CSS compiler expands into the pair, exactly as `place-content` expands. An
    // author who states one axis therefore reaches the same place as one who
    // states both, and the cascade orders the two declarations for free.
    #[doc = include_str!("property_id.doc/OverflowX.md")]
    OverflowX,
    #[doc = include_str!("property_id.doc/OverflowY.md")]
    OverflowY,

    #[doc = include_str!("property_id.doc/Content.md")]
    Content,

    // ---- Hit testing (Keyword-valued; INHERITED; neither layout nor paint) ----
    #[doc = include_str!("property_id.doc/PointerEvents.md")]
    PointerEvents,

    // ---- The kit's own vendor namespace, `-obscur-` (spec §4.13) ----
    #[doc = include_str!("property_id.doc/Painter.md")]
    Painter,
}

impl PropertyId {
    #[doc = include_str!("property_id.doc/name.md")]
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            // The one property whose name carries a vendor prefix, because it
            // names something no CSS specification has a word for. The prefix is
            // the whole point: a renderer that is not this one must ignore it.
            Self::Painter => "-obscur-painter",
            Self::Color => "color",
            Self::BackgroundColor => "background-color",
            Self::BackgroundImage => "background-image",
            Self::BackgroundSize => "background-size",
            Self::BackgroundPosition => "background-position",
            Self::BackgroundRepeat => "background-repeat",
            Self::ObjectFit => "object-fit",
            Self::ObjectPosition => "object-position",
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
            Self::BorderInlineStartWidth => "border-inline-start-width",
            Self::BorderInlineEndWidth => "border-inline-end-width",
            Self::BorderBlockStartWidth => "border-block-start-width",
            Self::BorderBlockEndWidth => "border-block-end-width",
            Self::BorderInlineStartStyle => "border-inline-start-style",
            Self::BorderInlineEndStyle => "border-inline-end-style",
            Self::BorderBlockStartStyle => "border-block-start-style",
            Self::BorderBlockEndStyle => "border-block-end-style",
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
            Self::MixBlendMode => "mix-blend-mode",
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
            Self::WritingMode => "writing-mode",
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
            Self::OverflowX => "overflow-x",
            Self::OverflowY => "overflow-y",
            Self::Content => "content",
            Self::PointerEvents => "pointer-events",
        }
    }

    #[doc = include_str!("property_id.doc/default_inheritance.md")]
    #[must_use]
    pub const fn default_inheritance(self) -> Inheritance {
        match self {
            // Text-affecting / typography properties inherit by default.
            Self::Color
            | Self::Visibility
            | Self::FontSize
            | Self::FontFamily
            | Self::Direction
            // `writing-mode` inherits (css-writing-modes-4 §3).
            | Self::WritingMode
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
            | Self::TabSize
            // `pointer-events` inherits (SVG 1.1 §16.6 / css-ui). That is not a
            // typography accident: inheritance IS the "and its descendants"
            // half of `pointer-events: none`, and it is what lets a descendant
            // opt back in with `auto` — a rule an ancestor walk at hit-test
            // time would have to special-case.
            | Self::PointerEvents => Inheritance::Inherited,
            // Box / paint / layout properties reset on each element.
            Self::BackgroundColor
            | Self::BackgroundImage
            | Self::BackgroundSize
            | Self::BackgroundPosition
            | Self::BackgroundRepeat
            | Self::ObjectFit
            | Self::ObjectPosition
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
            | Self::MixBlendMode
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
            | Self::BorderInlineStartWidth
            | Self::BorderInlineEndWidth
            | Self::BorderBlockStartWidth
            | Self::BorderBlockEndWidth
            | Self::BorderInlineStartStyle
            | Self::BorderInlineEndStyle
            | Self::BorderBlockStartStyle
            | Self::BorderBlockEndStyle
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
            // The `overflow` axis longhands are box / paint properties and reset
            // per element.
            | Self::OverflowX
            | Self::OverflowY
            // `content` selects generated text on a pseudo-element; it does not
            // inherit (CSS Content 3).
            | Self::Content
            // A painter draws ONE element's box. Inheriting it would repaint
            // every descendant with the same routine.
            | Self::Painter => Inheritance::NotInherited,
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
    fn pointer_events_is_named_and_inherits() {
        assert_eq!(PropertyId::PointerEvents.name(), "pointer-events");
        // Inherited: yes (SVG 1.1 §16.6). The subtree half of
        // `pointer-events: none` is exactly this inheritance.
        assert!(
            PropertyId::PointerEvents
                .default_inheritance()
                .is_inherited()
        );
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
    fn overflow_axis_names_and_not_inherited() {
        // `overflow` itself is a shorthand and has no id; the two axes are the
        // storage, and each names itself.
        assert_eq!(PropertyId::OverflowX.name(), "overflow-x");
        assert_eq!(PropertyId::OverflowY.name(), "overflow-y");
        // Like the other box / paint properties, they reset per element.
        assert!(!PropertyId::OverflowX.default_inheritance().is_inherited());
        assert!(!PropertyId::OverflowY.default_inheritance().is_inherited());
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
