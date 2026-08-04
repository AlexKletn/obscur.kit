#![doc = include_str!("paint_only.doc/_module.md")]

use crate::property::PropertyId;

impl PropertyId {
    #[doc = include_str!("paint_only.doc/is_paint_only.md")]
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
                // Where the image sits inside its box; the box is unchanged.
                | Self::ObjectPosition
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
                // Compositing only: the blend mode changes how the group paints,
                // never its box.
                | Self::MixBlendMode
                // A painter fills a box it does not size. Which routine draws
                // into it can change every frame of an animation, and making
                // that relayout would put a full layout pass behind a repaint.
                | Self::Painter
        )
    }

    #[doc = include_str!("paint_only.doc/affects_layout.md")]
    #[must_use]
    pub const fn affects_layout(self) -> bool {
        !self.is_paint_only()
    }
}

#[cfg(test)]
mod tests {
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
            PropertyId::Painter,
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
