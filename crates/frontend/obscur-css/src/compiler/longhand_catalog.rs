#![doc = include_str!("../../docs/api/compiler/longhand_catalog.doc/_module.md")]

use super::color_lowering::named_color;
use super::value_lowering::css_wide_keyword;
use crate::parser::ParsedValue;
use obscur_identity::InternedSymbol;
use obscur_style::property::{PropertyId, layout_keyword};

#[doc = include_str!("../../docs/api/compiler/longhand_catalog.doc/property_id_from_name.md")]
#[must_use]
pub(crate) fn property_id_from_name(name: &str) -> Option<PropertyId> {
    const ALL: [PropertyId; 139] = [
        // Logical box-model longhands (spec §4.13). They parse exactly like their
        // physical counterparts — a length — and resolve onto them at
        // computed-value time via `PropertyId::physical_under`.
        PropertyId::MarginInlineStart,
        PropertyId::MarginInlineEnd,
        PropertyId::MarginBlockStart,
        PropertyId::MarginBlockEnd,
        PropertyId::PaddingInlineStart,
        PropertyId::PaddingInlineEnd,
        PropertyId::PaddingBlockStart,
        PropertyId::PaddingBlockEnd,
        PropertyId::InsetInlineStart,
        PropertyId::InsetInlineEnd,
        PropertyId::InsetBlockStart,
        PropertyId::InsetBlockEnd,
        PropertyId::BorderInlineStartWidth,
        PropertyId::BorderInlineEndWidth,
        PropertyId::BorderBlockStartWidth,
        PropertyId::BorderBlockEndWidth,
        PropertyId::BorderInlineStartStyle,
        PropertyId::BorderInlineEndStyle,
        PropertyId::BorderBlockStartStyle,
        PropertyId::BorderBlockEndStyle,
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
        PropertyId::BackgroundBlendMode,
        PropertyId::BackgroundClip,
        PropertyId::ClipPath,
        PropertyId::BackgroundOrigin,
        PropertyId::BackgroundAttachment,
        PropertyId::Isolation,
        PropertyId::MinWidth,
        PropertyId::MinHeight,
        PropertyId::MaxWidth,
        PropertyId::MaxHeight,
        PropertyId::Gap,
        PropertyId::RowGap,
        PropertyId::ColumnGap,
        PropertyId::FlexBasis,
        PropertyId::FlexGrow,
        PropertyId::Order,
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
        PropertyId::OverflowX,
        PropertyId::OverflowY,
        PropertyId::ContainIntrinsicWidth,
        PropertyId::ContainIntrinsicHeight,
        PropertyId::ContainIntrinsicInlineSize,
        PropertyId::ContainIntrinsicBlockSize,
        PropertyId::ContainerType,
        PropertyId::ContainerName,
        PropertyId::Content,
        PropertyId::PointerEvents,
        PropertyId::PixelSnap,
        PropertyId::OverscrollBehaviorX,
        PropertyId::OverscrollBehaviorY,
        // The kit's own namespace. Its `name()` carries the `-obscur-` prefix,
        // so it is found by the same lookup as everything else — a vendor
        // property is a property, not a separate kind of thing.
        PropertyId::Painter,
    ];
    // `word-wrap` is the legacy alias for `overflow-wrap` (CSS Text 3 §5.2).
    if name.eq_ignore_ascii_case("word-wrap") {
        return Some(PropertyId::OverflowWrap);
    }
    ALL.into_iter()
        .find(|id| name.eq_ignore_ascii_case(id.name()))
}

/// Whether `property` takes a `<length>`/`<length-percentage>` value, for which a
/// unitless `0` is accepted as `0px`.
pub(super) fn is_length_longhand(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::Top
            | PropertyId::Right
            | PropertyId::Bottom
            | PropertyId::Left
            | PropertyId::Width
            | PropertyId::Height
            | PropertyId::MinWidth
            | PropertyId::MinHeight
            | PropertyId::MaxWidth
            | PropertyId::MaxHeight
            | PropertyId::MarginTop
            | PropertyId::MarginRight
            | PropertyId::MarginBottom
            | PropertyId::MarginLeft
            | PropertyId::PaddingTop
            | PropertyId::PaddingRight
            | PropertyId::PaddingBottom
            | PropertyId::PaddingLeft
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
            | PropertyId::BorderTopWidth
            | PropertyId::BorderRightWidth
            | PropertyId::BorderBottomWidth
            | PropertyId::BorderLeftWidth
            | PropertyId::BorderInlineStartWidth
            | PropertyId::BorderInlineEndWidth
            | PropertyId::BorderBlockStartWidth
            | PropertyId::BorderBlockEndWidth
            | PropertyId::BorderTopLeftRadius
            | PropertyId::BorderTopRightRadius
            | PropertyId::BorderBottomRightRadius
            | PropertyId::BorderBottomLeftRadius
            | PropertyId::OutlineWidth
            | PropertyId::OutlineOffset
            | PropertyId::Gap
            | PropertyId::RowGap
            | PropertyId::ColumnGap
            | PropertyId::FlexBasis
            | PropertyId::FontSize
            // `letter-spacing` / `word-spacing` take a `<length>` (CSS Text 3
            // §8.1 / §8.2), so a unitless `0` is `0px`.
            | PropertyId::LetterSpacing
            | PropertyId::WordSpacing
    )
}

/// Returns the keyword→reserved-symbol mapper for a closed-set layout property,
/// or [`None`] for any property whose keywords are interned normally.
pub(super) fn layout_keyword_mapper(
    property: PropertyId,
) -> Option<fn(&str) -> Option<InternedSymbol>> {
    match property {
        PropertyId::Display => Some(layout_keyword::display_from_name),
        PropertyId::BoxSizing => Some(layout_keyword::box_sizing_from_name),
        PropertyId::Position => Some(layout_keyword::position_from_name),
        PropertyId::FlexDirection => Some(layout_keyword::flex_direction_from_name),
        PropertyId::FlexWrap => Some(layout_keyword::flex_wrap_from_name),
        PropertyId::JustifyContent => Some(layout_keyword::justify_content_from_name),
        // `align-content` shares the `justify-content` distribution value grammar
        // (`start`/`end`/`center`/`space-*`).
        PropertyId::AlignContent => Some(layout_keyword::align_content_from_name),
        // The item alignment properties share the single-keyword grammar.
        PropertyId::AlignItems => Some(layout_keyword::align_items_from_name),
        PropertyId::JustifyItems => Some(layout_keyword::justify_items_from_name),
        // The per-item `*-self` properties additionally accept `auto`.
        PropertyId::AlignSelf | PropertyId::JustifySelf => {
            Some(layout_keyword::align_self_from_name)
        }
        // The four `border-*-style` longhands share the closed `none` / `solid`
        // (and folded visible-style) keyword set.
        PropertyId::BorderTopStyle
        | PropertyId::BorderRightStyle
        | PropertyId::BorderBottomStyle
        | PropertyId::BorderLeftStyle
        // The logical `border-*-style` longhands share the same closed keyword set;
        // they resolve onto a physical `border-*-style` at computed-value time.
        | PropertyId::BorderInlineStartStyle
        | PropertyId::BorderInlineEndStyle
        | PropertyId::BorderBlockStartStyle
        | PropertyId::BorderBlockEndStyle
        // `outline-style` shares the same closed line-style keyword set.
        | PropertyId::OutlineStyle => Some(layout_keyword::border_style_from_name),
        PropertyId::Direction => Some(layout_keyword::direction_from_name),
        PropertyId::WritingMode => Some(layout_keyword::writing_mode_from_name),
        PropertyId::FontStyle => Some(layout_keyword::font_style_from_name),
        PropertyId::TextAlign => Some(layout_keyword::text_align_from_name),
        PropertyId::TextOverflow => Some(layout_keyword::text_overflow_from_name),
        PropertyId::TextTransform => Some(layout_keyword::text_transform_from_name),
        PropertyId::WhiteSpace => Some(layout_keyword::white_space_from_name),
        PropertyId::OverflowWrap => Some(layout_keyword::overflow_wrap_from_name),
        PropertyId::WordBreak => Some(layout_keyword::word_break_from_name),
        PropertyId::OverflowX | PropertyId::OverflowY => {
            Some(layout_keyword::overflow_from_name)
        }
        PropertyId::PointerEvents => Some(layout_keyword::pointer_events_from_name),
        PropertyId::Visibility => Some(layout_keyword::visibility_from_name),
        PropertyId::PixelSnap => Some(pixel_snap_keyword),
        PropertyId::OverscrollBehaviorX | PropertyId::OverscrollBehaviorY => {
            Some(overscroll_behavior_keyword)
        }
        PropertyId::MixBlendMode => Some(layout_keyword::mix_blend_mode_from_name),
        PropertyId::Isolation => Some(layout_keyword::isolation_from_name),
        _ => None,
    }
}

/// Recognizes the closed set of pixel-snap keywords.
fn pixel_snap_keyword(name: &str) -> Option<InternedSymbol> {
    ["auto", "none", "edges"]
        .into_iter()
        .find(|keyword| name.eq_ignore_ascii_case(keyword))
        .map(InternedSymbol::named)
}

/// Lowers an overscroll behavior keyword to its canonical symbol.
fn overscroll_behavior_keyword(name: &str) -> Option<InternedSymbol> {
    ["auto", "contain", "none"]
        .into_iter()
        .find(|keyword| name.eq_ignore_ascii_case(keyword))
        .map(InternedSymbol::named)
}

#[doc = include_str!("../../docs/api/compiler/longhand_catalog.doc/keyword_value_is_foreign.md")]
#[must_use]
pub(crate) fn keyword_value_is_foreign(property: PropertyId, values: &[ParsedValue]) -> bool {
    let [ParsedValue::Keyword(keyword)] = values else {
        return false;
    };
    if css_wide_keyword(keyword).is_some() {
        return false;
    }
    if is_color_longhand(property) {
        return named_color(keyword).is_none();
    }
    if matches!(property, PropertyId::MaxWidth | PropertyId::MaxHeight)
        && keyword.eq_ignore_ascii_case("none")
    {
        return false;
    }
    if matches!(
        property,
        PropertyId::Gap
            | PropertyId::RowGap
            | PropertyId::ColumnGap
            | PropertyId::LetterSpacing
            | PropertyId::WordSpacing
    ) && keyword.eq_ignore_ascii_case("normal")
    {
        return false;
    }
    if is_length_longhand(property) {
        return !keyword.eq_ignore_ascii_case("auto")
            && layout_keyword::sizing_from_name(keyword).is_none();
    }
    false
}

/// The colour-valued longhands, for which a bare keyword (other than a CSS-wide
/// one) is never a valid value.
#[must_use]
const fn is_color_longhand(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::Color
            | PropertyId::BackgroundColor
            | PropertyId::BorderTopColor
            | PropertyId::BorderRightColor
            | PropertyId::BorderBottomColor
            | PropertyId::BorderLeftColor
            | PropertyId::OutlineColor
    )
}

#[cfg(test)]
mod tests {
    use super::super::CompiledDeclaration;
    use super::super::property_compiler::tests::{compile, standard, value_of};
    use super::*;

    use crate::error::UnsupportedFeatureKind;
    use obscur_style::property::{Length, PropertyId, PropertyValue, layout_keyword};

    #[test]
    fn unitless_zero_is_a_zero_length_for_length_longhands() {
        let zero = PropertyValue::Length(Length::px(0.0));
        // Positioning insets, box model, sizing, gap, flex-basis, font-size.
        assert_eq!(value_of("top: 0"), zero);
        assert_eq!(value_of("right: 0"), zero);
        assert_eq!(value_of("width: 0"), zero);
        assert_eq!(value_of("margin-top: 0"), zero);
        assert_eq!(value_of("padding-left: 0"), zero);
        assert_eq!(value_of("gap: 0"), zero);
        // An explicit `0px` is unchanged, and a non-length property keeps a
        // unitless number as a `Number` (so `opacity: 0` is not a length).
        assert_eq!(value_of("top: 0px"), zero);
        assert_eq!(value_of("opacity: 0"), PropertyValue::Number(0.0));
    }

    #[test]
    fn resolves_known_property_names() {
        assert_eq!(property_id_from_name("color"), Some(PropertyId::Color));
        assert_eq!(
            property_id_from_name("background-color"),
            Some(PropertyId::BackgroundColor)
        );
        assert_eq!(property_id_from_name("WIDTH"), Some(PropertyId::Width));
        assert_eq!(property_id_from_name("margin"), None);
    }

    #[test]
    fn resolves_new_longhand_names() {
        assert_eq!(
            property_id_from_name("margin-top"),
            Some(PropertyId::MarginTop)
        );
        assert_eq!(
            property_id_from_name("padding-left"),
            Some(PropertyId::PaddingLeft)
        );
        // Logical longhands parse by name like any other (spec §4.13).
        assert_eq!(
            property_id_from_name("margin-inline-start"),
            Some(PropertyId::MarginInlineStart)
        );
        assert_eq!(
            property_id_from_name("inset-block-end"),
            Some(PropertyId::InsetBlockEnd)
        );
        assert_eq!(
            property_id_from_name("padding-inline-end"),
            Some(PropertyId::PaddingInlineEnd)
        );
        assert_eq!(
            property_id_from_name("min-width"),
            Some(PropertyId::MinWidth)
        );
        assert_eq!(property_id_from_name("gap"), Some(PropertyId::Gap));
        assert_eq!(
            property_id_from_name("flex-basis"),
            Some(PropertyId::FlexBasis)
        );
        assert_eq!(
            property_id_from_name("font-size"),
            Some(PropertyId::FontSize)
        );
        // Case-insensitive, matching the existing seven.
        assert_eq!(
            property_id_from_name("FLEX-DIRECTION"),
            Some(PropertyId::FlexDirection)
        );
        // Shorthands are not longhand ids.
        assert_eq!(property_id_from_name("margin"), None);
        assert_eq!(property_id_from_name("padding"), None);
    }

    #[test]
    fn resolves_grid_template_names() {
        assert_eq!(
            property_id_from_name("grid-template-columns"),
            Some(PropertyId::GridTemplateColumns)
        );
        assert_eq!(
            property_id_from_name("GRID-TEMPLATE-ROWS"),
            Some(PropertyId::GridTemplateRows)
        );
    }

    #[test]
    fn resolves_grid_auto_track_names() {
        assert_eq!(
            property_id_from_name("grid-auto-columns"),
            Some(PropertyId::GridAutoColumns)
        );
        assert_eq!(
            property_id_from_name("GRID-AUTO-ROWS"),
            Some(PropertyId::GridAutoRows)
        );
    }

    #[test]
    fn resolves_grid_line_longhand_names() {
        assert_eq!(
            property_id_from_name("grid-column-start"),
            Some(PropertyId::GridColumnStart)
        );
        assert_eq!(
            property_id_from_name("GRID-COLUMN-END"),
            Some(PropertyId::GridColumnEnd)
        );
        assert_eq!(
            property_id_from_name("grid-row-start"),
            Some(PropertyId::GridRowStart)
        );
        assert_eq!(
            property_id_from_name("grid-row-end"),
            Some(PropertyId::GridRowEnd)
        );
        // The shorthands are not longhand ids.
        assert_eq!(property_id_from_name("grid-column"), None);
        assert_eq!(property_id_from_name("grid-row"), None);
    }

    #[test]
    fn resolves_font_family_name() {
        assert_eq!(
            property_id_from_name("font-family"),
            Some(PropertyId::FontFamily)
        );
        assert_eq!(
            property_id_from_name("FONT-FAMILY"),
            Some(PropertyId::FontFamily)
        );
    }

    #[test]
    fn resolves_direction_weight_style_names() {
        assert_eq!(
            property_id_from_name("direction"),
            Some(PropertyId::Direction)
        );
        assert_eq!(
            property_id_from_name("FONT-WEIGHT"),
            Some(PropertyId::FontWeight)
        );
        assert_eq!(
            property_id_from_name("font-style"),
            Some(PropertyId::FontStyle)
        );
    }

    #[test]
    fn resolves_text_align_name() {
        assert_eq!(
            property_id_from_name("text-align"),
            Some(PropertyId::TextAlign)
        );
        assert_eq!(
            property_id_from_name("TEXT-ALIGN"),
            Some(PropertyId::TextAlign)
        );
    }

    #[test]
    fn resolves_text_overflow_name() {
        assert_eq!(
            property_id_from_name("text-overflow"),
            Some(PropertyId::TextOverflow)
        );
        assert_eq!(
            property_id_from_name("TEXT-OVERFLOW"),
            Some(PropertyId::TextOverflow)
        );
    }

    #[test]
    fn resolves_overflow_axis_names() {
        assert_eq!(
            property_id_from_name("overflow-x"),
            Some(PropertyId::OverflowX)
        );
        assert_eq!(
            property_id_from_name("OVERFLOW-Y"),
            Some(PropertyId::OverflowY)
        );
        // `overflow` is the shorthand over the pair, so it is not a longhand id.
        assert_eq!(property_id_from_name("overflow"), None);
    }

    #[test]
    fn grid_alignment_property_names_resolve() {
        assert_eq!(
            property_id_from_name("justify-items"),
            Some(PropertyId::JustifyItems)
        );
        assert_eq!(
            property_id_from_name("justify-self"),
            Some(PropertyId::JustifySelf)
        );
        assert_eq!(
            property_id_from_name("align-self"),
            Some(PropertyId::AlignSelf)
        );
    }

    #[test]
    fn grid_alignment_keywords_map_to_reserved_symbols() {
        // `justify-items` shares the `align-items` closed set (no `auto`).
        assert_eq!(
            value_of("justify-items: center"),
            PropertyValue::Keyword(layout_keyword::ALIGN_CENTER)
        );
        assert_eq!(
            value_of("justify-items: start"),
            PropertyValue::Keyword(layout_keyword::ALIGN_START)
        );
        // The `*-self` properties additionally accept `auto`.
        assert_eq!(
            value_of("justify-self: center"),
            PropertyValue::Keyword(layout_keyword::ALIGN_CENTER)
        );
        assert_eq!(
            value_of("align-self: end"),
            PropertyValue::Keyword(layout_keyword::ALIGN_END)
        );
        assert_eq!(
            value_of("align-self: auto"),
            PropertyValue::Keyword(layout_keyword::ALIGN_AUTO)
        );
    }

    #[test]
    fn align_content_maps_to_reserved_symbols() {
        // `align-content` shares the `justify-content` distribution grammar.
        assert_eq!(
            value_of("align-content: space-between"),
            PropertyValue::Keyword(layout_keyword::JUSTIFY_SPACE_BETWEEN)
        );
        assert_eq!(
            value_of("align-content: space-evenly"),
            PropertyValue::Keyword(layout_keyword::JUSTIFY_SPACE_EVENLY)
        );
        assert_eq!(
            value_of("align-content: center"),
            PropertyValue::Keyword(layout_keyword::JUSTIFY_CENTER)
        );
    }

    #[test]
    fn display_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("display: flex"),
            PropertyValue::Keyword(layout_keyword::DISPLAY_FLEX)
        );
        assert_eq!(
            value_of("display: grid"),
            PropertyValue::Keyword(layout_keyword::DISPLAY_GRID)
        );
        assert_eq!(
            value_of("display: block"),
            PropertyValue::Keyword(layout_keyword::DISPLAY_BLOCK)
        );
        assert_eq!(
            value_of("display: none"),
            PropertyValue::Keyword(layout_keyword::DISPLAY_NONE)
        );
    }

    #[test]
    fn flex_layout_keywords_map_to_reserved_symbols() {
        assert_eq!(
            value_of("flex-direction: column"),
            PropertyValue::Keyword(layout_keyword::FLEX_COLUMN)
        );
        assert_eq!(
            value_of("flex-wrap: wrap"),
            PropertyValue::Keyword(layout_keyword::FLEX_WRAP_WRAP)
        );
        assert_eq!(
            value_of("flex-wrap: wrap-reverse"),
            PropertyValue::Keyword(layout_keyword::FLEX_WRAP_WRAP_REVERSE)
        );
        assert_eq!(
            value_of("justify-content: center"),
            PropertyValue::Keyword(layout_keyword::JUSTIFY_CENTER)
        );
        assert_eq!(
            value_of("align-items: stretch"),
            PropertyValue::Keyword(layout_keyword::ALIGN_STRETCH)
        );
    }

    #[test]
    fn a_word_outside_the_closed_set_is_reported_rather_than_interned() {
        // The mapper is the whole grammar of these properties, so a word it
        // does not know is a declaration that matches no grammar. It used to be
        // interned instead, which was not inert: `OverflowAxes` reads every
        // symbol that is not `visible` or `clip` as clipping, so the typo below
        // clipped the box. Both cases now go the same way — the value is
        // dropped and the compiler SAYS so, which the substitution path cannot
        // do because by then the compiler is gone.
        for text in [
            // A word `overflow-x` simply does not have.
            "overflow-x: sideways",
            // And a word CSS has that this renderer does not model. Same
            // answer, for the honest reason that it cannot carry the value
            // either — the difference is a note in the diagnostic list, not a
            // computed value that means nothing.
            "display: inline",
        ] {
            let (compiled, _) = compile(text);
            let CompiledDeclaration::Unsupported(report) = compiled else {
                panic!("expected `{text}` to be reported, got {compiled:?}");
            };
            assert_eq!(report.kind(), UnsupportedFeatureKind::Property);
        }
    }

    #[test]
    fn layout_keyword_prop_honours_css_wide_keywords() {
        assert_eq!(value_of("display: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("flex-direction: initial"), PropertyValue::Initial);
    }

    #[test]
    fn direction_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("direction: rtl"),
            PropertyValue::Keyword(layout_keyword::DIRECTION_RTL)
        );
        assert_eq!(
            value_of("direction: ltr"),
            PropertyValue::Keyword(layout_keyword::DIRECTION_LTR)
        );
        assert_eq!(
            standard("direction: rtl").0.property(),
            PropertyId::Direction
        );
    }

    #[test]
    fn font_style_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("font-style: italic"),
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_ITALIC)
        );
        assert_eq!(
            value_of("font-style: normal"),
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_NORMAL)
        );
        assert_eq!(
            value_of("font-style: oblique"),
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_OBLIQUE)
        );
    }

    #[test]
    fn text_align_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("text-align: justify"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_JUSTIFY)
        );
        assert_eq!(
            value_of("text-align: center"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_CENTER)
        );
        assert_eq!(
            value_of("text-align: right"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_RIGHT)
        );
        assert_eq!(
            value_of("text-align: left"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_LEFT)
        );
        assert_eq!(
            value_of("text-align: start"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_START)
        );
        assert_eq!(
            value_of("text-align: end"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_END)
        );
        assert_eq!(
            standard("text-align: justify").0.property(),
            PropertyId::TextAlign
        );
    }

    #[test]
    fn text_overflow_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("text-overflow: ellipsis"),
            PropertyValue::Keyword(layout_keyword::TEXT_OVERFLOW_ELLIPSIS)
        );
        assert_eq!(
            value_of("text-overflow: clip"),
            PropertyValue::Keyword(layout_keyword::TEXT_OVERFLOW_CLIP)
        );
        assert_eq!(
            standard("text-overflow: ellipsis").0.property(),
            PropertyId::TextOverflow
        );
        // CSS-wide keywords still take priority.
        assert_eq!(value_of("text-overflow: inherit"), PropertyValue::Inherit);
    }

    #[test]
    fn text_transform_maps_to_reserved_symbols() {
        assert_eq!(
            property_id_from_name("text-transform"),
            Some(PropertyId::TextTransform)
        );
        assert_eq!(
            value_of("text-transform: uppercase"),
            PropertyValue::Keyword(layout_keyword::TEXT_TRANSFORM_UPPERCASE)
        );
        assert_eq!(
            value_of("text-transform: lowercase"),
            PropertyValue::Keyword(layout_keyword::TEXT_TRANSFORM_LOWERCASE)
        );
        assert_eq!(
            value_of("text-transform: capitalize"),
            PropertyValue::Keyword(layout_keyword::TEXT_TRANSFORM_CAPITALIZE)
        );
        assert_eq!(
            value_of("text-transform: none"),
            PropertyValue::Keyword(layout_keyword::TEXT_TRANSFORM_NONE)
        );
        // CSS-wide keywords still take priority.
        assert_eq!(value_of("text-transform: inherit"), PropertyValue::Inherit);
    }

    #[test]
    fn overflow_wrap_maps_to_reserved_symbols_and_word_wrap_is_an_alias() {
        // CSS Text 3 §5.2: `overflow-wrap` and its legacy `word-wrap` alias both
        // resolve to `PropertyId::OverflowWrap`.
        assert_eq!(
            property_id_from_name("overflow-wrap"),
            Some(PropertyId::OverflowWrap)
        );
        assert_eq!(
            property_id_from_name("word-wrap"),
            Some(PropertyId::OverflowWrap)
        );
        assert_eq!(
            property_id_from_name("WORD-WRAP"),
            Some(PropertyId::OverflowWrap)
        );
        assert_eq!(
            value_of("overflow-wrap: break-word"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_WRAP_BREAK_WORD)
        );
        assert_eq!(
            value_of("overflow-wrap: anywhere"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_WRAP_ANYWHERE)
        );
        assert_eq!(
            value_of("overflow-wrap: normal"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_WRAP_NORMAL)
        );
        // The legacy alias lowers through the same closed keyword set.
        assert_eq!(
            value_of("word-wrap: break-word"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_WRAP_BREAK_WORD)
        );
        // Both inherit by default (CSS Text 3 §5.2).
        assert!(
            PropertyId::OverflowWrap
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn word_break_maps_to_reserved_symbols() {
        assert_eq!(
            property_id_from_name("word-break"),
            Some(PropertyId::WordBreak)
        );
        assert_eq!(
            value_of("word-break: break-all"),
            PropertyValue::Keyword(layout_keyword::WORD_BREAK_BREAK_ALL)
        );
        assert_eq!(
            value_of("word-break: keep-all"),
            PropertyValue::Keyword(layout_keyword::WORD_BREAK_KEEP_ALL)
        );
        assert_eq!(
            value_of("word-break: normal"),
            PropertyValue::Keyword(layout_keyword::WORD_BREAK_NORMAL)
        );
        assert_eq!(value_of("word-break: inherit"), PropertyValue::Inherit);
        assert!(PropertyId::WordBreak.default_inheritance().is_inherited());
    }

    #[test]
    fn overflow_axis_longhands_map_to_reserved_symbols() {
        assert_eq!(
            value_of("overflow-y: hidden"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_HIDDEN)
        );
        assert_eq!(
            value_of("overflow-y: visible"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_VISIBLE)
        );
        assert_eq!(
            value_of("overflow-x: clip"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_CLIP)
        );
        assert_eq!(
            value_of("overflow-x: scroll"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_SCROLL)
        );
        assert_eq!(
            value_of("overflow-y: auto"),
            PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO)
        );
        assert_eq!(
            standard("overflow-y: hidden").0.property(),
            PropertyId::OverflowY
        );
        assert_eq!(
            standard("overflow-x: hidden").0.property(),
            PropertyId::OverflowX
        );
        // CSS-wide keywords still take priority.
        assert_eq!(value_of("overflow-y: inherit"), PropertyValue::Inherit);
    }

    #[test]
    fn pointer_events_maps_to_reserved_symbols() {
        // Before this property was catalogued the declaration compiled to
        // `Unsupported` and was dropped, so §4.7's pointer-event policy had
        // nothing to consult.
        assert_eq!(
            property_id_from_name("pointer-events"),
            Some(PropertyId::PointerEvents)
        );
        assert_eq!(
            value_of("pointer-events: none"),
            PropertyValue::Keyword(layout_keyword::POINTER_EVENTS_NONE)
        );
        assert_eq!(
            value_of("pointer-events: auto"),
            PropertyValue::Keyword(layout_keyword::POINTER_EVENTS_AUTO)
        );
        assert_eq!(
            standard("pointer-events: none").0.property(),
            PropertyId::PointerEvents
        );
        // CSS-wide keywords still take priority.
        assert_eq!(value_of("pointer-events: inherit"), PropertyValue::Inherit);
        // Inherited: yes — the subtree half of `none` is the cascade's doing.
        assert!(
            PropertyId::PointerEvents
                .default_inheritance()
                .is_inherited()
        );
    }

    #[test]
    fn position_keyword_lowers_to_reserved_symbol() {
        assert_eq!(
            value_of("position: relative"),
            PropertyValue::Keyword(layout_keyword::POSITION_RELATIVE)
        );
        assert_eq!(
            value_of("position: static"),
            PropertyValue::Keyword(layout_keyword::POSITION_STATIC)
        );
        assert_eq!(
            value_of("position: absolute"),
            PropertyValue::Keyword(layout_keyword::POSITION_ABSOLUTE)
        );
        assert_eq!(
            value_of("position: fixed"),
            PropertyValue::Keyword(layout_keyword::POSITION_FIXED)
        );
        assert_eq!(
            value_of("position: sticky"),
            PropertyValue::Keyword(layout_keyword::POSITION_STICKY)
        );
    }

    #[test]
    fn lowers_length_longhands() {
        assert_eq!(
            value_of("margin-top: 8px"),
            PropertyValue::Length(Length::px(8.0))
        );
        assert_eq!(
            value_of("padding-left: 5%"),
            PropertyValue::Length(Length::Percent(5.0))
        );
        assert_eq!(
            value_of("min-width: 100px"),
            PropertyValue::Length(Length::px(100.0))
        );
        assert_eq!(
            value_of("gap: 12px"),
            PropertyValue::Length(Length::px(12.0))
        );
        assert_eq!(
            value_of("font-size: 14px"),
            PropertyValue::Length(Length::px(14.0))
        );
    }

    #[test]
    fn length_longhand_carries_its_property_id() {
        assert_eq!(
            standard("margin-top: 8px").0.property(),
            PropertyId::MarginTop
        );
        assert_eq!(
            standard("font-size: 14px").0.property(),
            PropertyId::FontSize
        );
    }

    #[test]
    fn flex_basis_auto_lowers_to_auto_length() {
        assert_eq!(
            value_of("flex-basis: auto"),
            PropertyValue::Length(Length::Auto)
        );
    }

    #[test]
    fn lowers_number_valued_flex_props() {
        assert_eq!(value_of("flex-grow: 1"), PropertyValue::Number(1.0));
        assert_eq!(value_of("flex-shrink: 0"), PropertyValue::Number(0.0));
    }

    #[test]
    fn inset_longhands_lower_to_lengths() {
        assert_eq!(
            value_of("top: 10px"),
            PropertyValue::Length(Length::px(10.0))
        );
        assert_eq!(
            value_of("left: 5px"),
            PropertyValue::Length(Length::px(5.0))
        );
        assert_eq!(
            value_of("right: 50%"),
            PropertyValue::Length(Length::Percent(50.0))
        );
        assert_eq!(
            value_of("bottom: auto"),
            PropertyValue::Length(Length::Auto)
        );
    }

    #[test]
    fn resolves_line_height_name_and_lowers_number_and_length() {
        assert_eq!(
            property_id_from_name("line-height"),
            Some(PropertyId::LineHeight)
        );
        assert_eq!(
            property_id_from_name("LINE-HEIGHT"),
            Some(PropertyId::LineHeight)
        );
        // A unitless number is kept as a `Number` multiple of the font size.
        assert_eq!(value_of("line-height: 1.5"), PropertyValue::Number(1.5));
        assert_eq!(
            standard("line-height: 1.5").0.property(),
            PropertyId::LineHeight
        );
        // An absolute length lowers to a px `Length`.
        assert_eq!(
            value_of("line-height: 24px"),
            PropertyValue::Length(Length::px(24.0))
        );
        // `line-height: 0` is a zero multiple (a unitless number), not `0px`.
        assert_eq!(value_of("line-height: 0"), PropertyValue::Number(0.0));
        // CSS-wide keywords still take priority.
        assert_eq!(value_of("line-height: inherit"), PropertyValue::Inherit);
    }

    #[test]
    fn resolves_and_lowers_letter_and_word_spacing() {
        // Names resolve (case-insensitive) to the new longhands.
        assert_eq!(
            property_id_from_name("letter-spacing"),
            Some(PropertyId::LetterSpacing)
        );
        assert_eq!(
            property_id_from_name("WORD-SPACING"),
            Some(PropertyId::WordSpacing)
        );
        // A `<length>` lowers to a pixel Length (CSS Text 3 §8.1 / §8.2), and may
        // be negative or a unitless `0`.
        assert_eq!(
            value_of("letter-spacing: 6px"),
            PropertyValue::Length(Length::px(6.0))
        );
        assert_eq!(
            value_of("word-spacing: 12px"),
            PropertyValue::Length(Length::px(12.0))
        );
        assert_eq!(
            value_of("letter-spacing: -2px"),
            PropertyValue::Length(Length::px(-2.0))
        );
        assert_eq!(
            value_of("letter-spacing: 0"),
            PropertyValue::Length(Length::px(0.0))
        );
        // CSS-wide keywords still take priority; `normal` is a plain keyword the
        // runtime reads as 0.
        assert_eq!(value_of("letter-spacing: inherit"), PropertyValue::Inherit);
    }

    #[test]
    fn direction_weight_style_honour_css_wide_keywords() {
        assert_eq!(value_of("direction: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("font-weight: initial"), PropertyValue::Initial);
        assert_eq!(value_of("font-style: unset"), PropertyValue::Unset);
    }

    #[test]
    fn flex_is_a_shorthand_not_a_longhand_id() {
        assert_eq!(property_id_from_name("flex"), None);
    }
}
