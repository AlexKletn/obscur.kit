#![doc = include_str!("computed_property_set.doc/_module.md")]

use crate::property::PropertyId;

#[doc = include_str!("computed_property_set.doc/COMPUTED_PROPERTIES.md")]
pub(super) const COMPUTED_PROPERTIES: &[PropertyId] = &[
    // Paint.
    PropertyId::Color,
    PropertyId::BackgroundColor,
    PropertyId::BackgroundImage,
    PropertyId::BackgroundSize,
    PropertyId::BackgroundPosition,
    PropertyId::BackgroundRepeat,
    PropertyId::ObjectFit,
    PropertyId::ObjectPosition,
    PropertyId::Opacity,
    PropertyId::Visibility,
    // Positioning.
    PropertyId::Position,
    PropertyId::Top,
    PropertyId::Right,
    PropertyId::Bottom,
    PropertyId::Left,
    PropertyId::ZIndex,
    // Box model: sizing.
    PropertyId::Width,
    PropertyId::Height,
    PropertyId::BoxSizing,
    PropertyId::MinWidth,
    PropertyId::MinHeight,
    PropertyId::MaxWidth,
    PropertyId::MaxHeight,
    // Box model: margins.
    PropertyId::MarginTop,
    PropertyId::MarginRight,
    PropertyId::MarginBottom,
    PropertyId::MarginLeft,
    // Box model: padding.
    PropertyId::PaddingTop,
    PropertyId::PaddingRight,
    PropertyId::PaddingBottom,
    PropertyId::PaddingLeft,
    // Box model: border (width feeds the box model; style + color paint).
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
    // Box model: border radius (rounds the background / border / content clip).
    PropertyId::BorderTopLeftRadius,
    PropertyId::BorderTopRightRadius,
    PropertyId::BorderBottomRightRadius,
    PropertyId::BorderBottomLeftRadius,
    // Outline (painted outside the border box; no layout).
    PropertyId::OutlineWidth,
    PropertyId::OutlineStyle,
    PropertyId::OutlineColor,
    PropertyId::OutlineOffset,
    // Box effects.
    PropertyId::BoxShadow,
    PropertyId::Filter,
    PropertyId::BackdropFilter,
    // 2D transform.
    PropertyId::Transform,
    PropertyId::TransformOrigin,
    // Compositing.
    PropertyId::MixBlendMode,
    // Flex container / item.
    PropertyId::Display,
    PropertyId::FlexDirection,
    PropertyId::FlexWrap,
    PropertyId::JustifyContent,
    PropertyId::AlignContent,
    PropertyId::AlignItems,
    PropertyId::JustifyItems,
    PropertyId::AlignSelf,
    PropertyId::JustifySelf,
    PropertyId::Gap,
    PropertyId::FlexBasis,
    PropertyId::FlexGrow,
    PropertyId::FlexShrink,
    // Grid container.
    PropertyId::GridTemplateColumns,
    PropertyId::GridTemplateRows,
    PropertyId::GridTemplateAreas,
    PropertyId::GridAutoFlow,
    PropertyId::GridAutoColumns,
    PropertyId::GridAutoRows,
    // Grid item line placement.
    PropertyId::GridColumnStart,
    PropertyId::GridColumnEnd,
    PropertyId::GridRowStart,
    PropertyId::GridRowEnd,
    PropertyId::GridArea,
    // Typography.
    PropertyId::FontSize,
    PropertyId::FontFamily,
    PropertyId::FontWeight,
    PropertyId::FontStyle,
    PropertyId::LineHeight,
    PropertyId::Direction,
    PropertyId::WritingMode,
    PropertyId::TextAlign,
    PropertyId::TextOverflow,
    // Inherited text spacing/casing (CSS Text 3 §8.1 / §8.2 / §2.1).
    PropertyId::LetterSpacing,
    PropertyId::WordSpacing,
    PropertyId::TextTransform,
    // Inherited white-space processing + tab-size (CSS Text 3 §3 / §4.3).
    PropertyId::WhiteSpace,
    // Inherited line-break controls (CSS Text 3 §5.2).
    PropertyId::OverflowWrap,
    PropertyId::WordBreak,
    PropertyId::TabSize,
    // Overflow (pure paint/clip; no layout effect).
    PropertyId::Overflow,
    // The kit's own namespace.
    PropertyId::Painter,
];

#[doc = include_str!("computed_property_set.doc/FACE_PROPERTIES.md")]
pub(super) const FACE_PROPERTIES: &[PropertyId] = &[
    PropertyId::FontFamily,
    PropertyId::FontWeight,
    PropertyId::FontStyle,
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every modelled [`PropertyId`] must be in the computed set, or an authored
    /// value for it would be dropped at the computed-value step (the bug this
    /// closes). The `match` is exhaustive, so a new upstream variant fails to
    /// compile here until it is classified; the assertion then fails until it is
    /// also added to `COMPUTED_PROPERTIES`.
    #[test]
    fn computed_properties_cover_every_property_id() {
        // Exhaustive by construction: adding a `PropertyId` variant upstream
        // breaks this `match` until the new variant is listed below.
        const fn assert_known(id: PropertyId) {
            match id {
                PropertyId::Color
                | PropertyId::BackgroundColor
                | PropertyId::BackgroundImage
                | PropertyId::BackgroundSize
                | PropertyId::BackgroundPosition
                | PropertyId::BackgroundRepeat
                | PropertyId::ObjectFit
                | PropertyId::ObjectPosition
                | PropertyId::Opacity
                | PropertyId::Width
                | PropertyId::Height
                | PropertyId::BoxSizing
                | PropertyId::Display
                | PropertyId::Visibility
                | PropertyId::Position
                | PropertyId::Top
                | PropertyId::Right
                | PropertyId::Bottom
                | PropertyId::Left
                | PropertyId::ZIndex
                | PropertyId::MarginTop
                | PropertyId::MarginRight
                | PropertyId::MarginBottom
                | PropertyId::MarginLeft
                | PropertyId::PaddingTop
                | PropertyId::PaddingRight
                | PropertyId::PaddingBottom
                | PropertyId::PaddingLeft
                | PropertyId::BorderTopWidth
                | PropertyId::BorderRightWidth
                | PropertyId::BorderBottomWidth
                | PropertyId::BorderLeftWidth
                | PropertyId::BorderTopStyle
                | PropertyId::BorderRightStyle
                | PropertyId::BorderBottomStyle
                | PropertyId::BorderLeftStyle
                | PropertyId::BorderTopColor
                | PropertyId::BorderRightColor
                | PropertyId::BorderBottomColor
                | PropertyId::BorderLeftColor
                | PropertyId::BorderTopLeftRadius
                | PropertyId::BorderTopRightRadius
                | PropertyId::BorderBottomRightRadius
                | PropertyId::BorderBottomLeftRadius
                | PropertyId::OutlineWidth
                | PropertyId::OutlineStyle
                | PropertyId::OutlineColor
                | PropertyId::OutlineOffset
                | PropertyId::BoxShadow
                | PropertyId::Filter
                | PropertyId::BackdropFilter
                | PropertyId::Transform
                | PropertyId::TransformOrigin
                | PropertyId::MixBlendMode
                | PropertyId::MinWidth
                | PropertyId::MinHeight
                | PropertyId::MaxWidth
                | PropertyId::MaxHeight
                | PropertyId::Gap
                | PropertyId::FlexBasis
                | PropertyId::FlexGrow
                | PropertyId::FlexShrink
                | PropertyId::FlexDirection
                | PropertyId::FlexWrap
                | PropertyId::JustifyContent
                | PropertyId::AlignContent
                | PropertyId::AlignItems
                | PropertyId::JustifyItems
                | PropertyId::AlignSelf
                | PropertyId::JustifySelf
                | PropertyId::GridTemplateColumns
                | PropertyId::GridTemplateRows
                | PropertyId::GridTemplateAreas
                | PropertyId::GridAutoFlow
                | PropertyId::GridAutoColumns
                | PropertyId::GridAutoRows
                | PropertyId::GridColumnStart
                | PropertyId::GridColumnEnd
                | PropertyId::GridRowStart
                | PropertyId::GridRowEnd
                | PropertyId::GridArea
                | PropertyId::FontSize
                | PropertyId::FontFamily
                | PropertyId::Direction
                | PropertyId::WritingMode
                | PropertyId::FontWeight
                | PropertyId::FontStyle
                | PropertyId::LineHeight
                | PropertyId::TextAlign
                | PropertyId::TextOverflow
                | PropertyId::LetterSpacing
                | PropertyId::WordSpacing
                | PropertyId::TextTransform
                | PropertyId::WhiteSpace
                | PropertyId::OverflowWrap
                | PropertyId::WordBreak
                | PropertyId::TabSize
                | PropertyId::Overflow
                // Logical properties are classified but deliberately NOT computed:
                // each resolves into its physical counterpart via
                // `cascaded_for`, so the computed style holds `margin-left` and
                // every downstream phase reads that one place. The assertion below
                // exempts them for exactly this reason.
                | PropertyId::MarginInlineStart
                | PropertyId::MarginInlineEnd
                | PropertyId::MarginBlockStart
                | PropertyId::MarginBlockEnd
                | PropertyId::PaddingInlineStart
                | PropertyId::PaddingInlineEnd
                | PropertyId::PaddingBlockStart
                | PropertyId::PaddingBlockEnd
                | PropertyId::InsetInlineStart
                | PropertyId::InsetInlineEnd
                | PropertyId::InsetBlockStart
                | PropertyId::InsetBlockEnd
                | PropertyId::BorderInlineStartWidth
                | PropertyId::BorderInlineEndWidth
                | PropertyId::BorderBlockStartWidth
                | PropertyId::BorderBlockEndWidth
                | PropertyId::BorderInlineStartStyle
                | PropertyId::BorderInlineEndStyle
                | PropertyId::BorderBlockStartStyle
                | PropertyId::BorderBlockEndStyle
                // `content` is generated-content-only: like the logical properties
                // it is deliberately NOT a computed property of a real box (it
                // decorates a `::before`/`::after` pseudo-element), so it is
                // classified here but excluded from `COMPUTED_PROPERTIES`.
                | PropertyId::Content
                | PropertyId::Painter => {}
            }
        }
        let all = [
            PropertyId::Color,
            PropertyId::BackgroundColor,
            PropertyId::BackgroundImage,
            PropertyId::BackgroundSize,
            PropertyId::BackgroundPosition,
            PropertyId::BackgroundRepeat,
            PropertyId::ObjectFit,
            PropertyId::ObjectPosition,
            PropertyId::Opacity,
            PropertyId::Width,
            PropertyId::Height,
            PropertyId::BoxSizing,
            PropertyId::Display,
            PropertyId::Visibility,
            PropertyId::Position,
            PropertyId::Top,
            PropertyId::Right,
            PropertyId::Bottom,
            PropertyId::Left,
            PropertyId::ZIndex,
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
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
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
            PropertyId::OutlineWidth,
            PropertyId::OutlineStyle,
            PropertyId::OutlineColor,
            PropertyId::OutlineOffset,
            PropertyId::BoxShadow,
            PropertyId::Filter,
            PropertyId::BackdropFilter,
            PropertyId::Transform,
            PropertyId::TransformOrigin,
            PropertyId::MixBlendMode,
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
            PropertyId::AlignContent,
            PropertyId::AlignItems,
            PropertyId::JustifyItems,
            PropertyId::AlignSelf,
            PropertyId::JustifySelf,
            PropertyId::GridTemplateColumns,
            PropertyId::GridTemplateRows,
            PropertyId::GridTemplateAreas,
            PropertyId::GridAutoFlow,
            PropertyId::GridAutoColumns,
            PropertyId::GridAutoRows,
            PropertyId::GridColumnStart,
            PropertyId::GridColumnEnd,
            PropertyId::GridRowStart,
            PropertyId::GridRowEnd,
            PropertyId::GridArea,
            PropertyId::FontSize,
            PropertyId::FontFamily,
            PropertyId::Direction,
            PropertyId::WritingMode,
            PropertyId::FontWeight,
            PropertyId::FontStyle,
            PropertyId::LineHeight,
            PropertyId::TextAlign,
            PropertyId::TextOverflow,
            PropertyId::LetterSpacing,
            PropertyId::WordSpacing,
            PropertyId::TextTransform,
            PropertyId::WhiteSpace,
            PropertyId::OverflowWrap,
            PropertyId::WordBreak,
            PropertyId::TabSize,
            PropertyId::Overflow,
            PropertyId::Painter,
        ];
        for id in all {
            assert_known(id);
            assert!(
                COMPUTED_PROPERTIES.contains(&id),
                "{} is modelled but not computed — authored values would be dropped",
                id.name()
            );
            assert!(
                !id.is_logical(),
                "{} is logical: it must resolve INTO a physical property, not be \
                 computed alongside one — two sources of truth for one box edge",
                id.name()
            );
        }
        assert_eq!(COMPUTED_PROPERTIES.len(), all.len());
    }
}
