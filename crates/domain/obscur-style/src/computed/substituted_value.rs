#![doc = include_str!("../../docs/api/computed/substituted_value.doc/_module.md")]

use obscur_identity::{FastMap, InternedSymbol};

use crate::computed::{CustomValueParser, substitute};
use crate::property::{ContainerComponent, PropertyId, PropertyValue};

#[doc = include_str!("../../docs/api/computed/substituted_value.doc/substituted_value.md")]
pub(crate) fn substituted_value(
    property: PropertyId,
    value: &PropertyValue,
    custom_values: Option<&dyn CustomValueParser>,
) -> PropertyValue {
    // A custom property kept the text it was declared with, because until this
    // moment there was no grammar to read it against. This is that moment: the
    // property it landed on names the grammar, and the port owns it.
    //
    // Taken by reference so that text is read where it lies: it belongs to the
    // custom-property map, which outlives this call, and copying it in bought
    // one heap string per reference per node per restyle and nothing else.
    let value = match value {
        PropertyValue::RawTokens(tokens) => {
            match custom_values.and_then(|parser| parser.parse(property, tokens)) {
                Some(parsed) => parsed,
                // Either nothing this property accepts, or no grammar installed
                // to ask. Both are invalid at computed-value time.
                None => return PropertyValue::Unset,
            }
        }
        // The one copy still owed: what the reference resolved to becomes this
        // node's own value, and the map keeps its.
        other => other.clone(),
    };
    if fits(property, &value) {
        value
    } else {
        PropertyValue::Unset
    }
}

pub(super) fn substituted_container(
    property: PropertyId,
    name: InternedSymbol,
    fallback: Option<&str>,
    custom: &FastMap<InternedSymbol, PropertyValue>,
    parser: Option<&dyn CustomValueParser>,
) -> Option<PropertyValue> {
    let tokens = container_variable_tokens(name, fallback, custom)?;
    container_longhand(property, tokens, parser)
}

pub(super) fn substituted_container_template(
    property: PropertyId,
    components: &[ContainerComponent],
    custom: &FastMap<InternedSymbol, PropertyValue>,
    parser: Option<&dyn CustomValueParser>,
) -> Option<PropertyValue> {
    let tokens = shorthand_tokens(components, custom)?;
    container_longhand(property, &tokens, parser)
}

pub(super) fn substituted_flex_flow(
    property: PropertyId,
    components: &[ContainerComponent],
    custom: &FastMap<InternedSymbol, PropertyValue>,
    parser: Option<&dyn CustomValueParser>,
) -> Option<PropertyValue> {
    let tokens = shorthand_tokens(components, custom)?;
    let [direction, wrap] = parser?.parse_flex_flow(&tokens)?;
    let value = match property {
        PropertyId::FlexDirection => direction,
        PropertyId::FlexWrap => wrap,
        _ => return None,
    };
    fits(property, &value).then_some(value)
}

/// Resolves a deferred gap shorthand and returns the requested axis.
pub(super) fn substituted_gap(
    property: PropertyId,
    components: &[ContainerComponent],
    custom: &FastMap<InternedSymbol, PropertyValue>,
    parser: Option<&dyn CustomValueParser>,
) -> Option<PropertyValue> {
    if let [
        ContainerComponent::Variable { name, .. }
        | ContainerComponent::VariableWithTemplateFallback { name, .. },
    ] = components
        && let Some(value) = custom.get(name).and_then(|value| substitute(value, custom))
        && !matches!(value, PropertyValue::RawTokens(_) | PropertyValue::Initial)
    {
        return fits(property, value).then(|| value.clone());
    }
    let tokens = shorthand_tokens(components, custom)?;
    let [row, column] = parser?.parse_gap(&tokens)?;
    let value = match property {
        PropertyId::RowGap => row,
        PropertyId::ColumnGap => column,
        _ => return None,
    };
    fits(property, &value).then_some(value)
}

/// Substitutes a variable template and parses the resulting tokens as the target longhand.
pub(super) fn substituted_variable_template(
    property: PropertyId,
    components: &[ContainerComponent],
    custom: &FastMap<InternedSymbol, PropertyValue>,
    parser: Option<&dyn CustomValueParser>,
) -> Option<PropertyValue> {
    // A host can supply a typed custom value. A sole reference can use that
    // value directly, without asking a token parser to reconstruct it.
    match components {
        [
            ContainerComponent::Variable { name, .. }
            | ContainerComponent::VariableWithTemplateFallback { name, .. },
        ] => {
            if let Some(value) = custom.get(name).and_then(|value| substitute(value, custom))
                && !matches!(value, PropertyValue::RawTokens(_) | PropertyValue::Initial)
            {
                return fits(property, value).then(|| value.clone());
            }
            if let [ContainerComponent::VariableWithTemplateFallback { fallback, .. }] = components
                && shorthand_variable(*name, custom)?.is_none()
            {
                return substituted_variable_template(property, fallback, custom, parser);
            }
        }
        _ => {}
    }
    let tokens = shorthand_tokens(components, custom)?;
    let value = parser?.parse(property, &tokens)?;
    fits(property, &value).then_some(value)
}

fn shorthand_tokens(
    components: &[ContainerComponent],
    custom: &FastMap<InternedSymbol, PropertyValue>,
) -> Option<String> {
    let mut tokens = String::new();
    append_shorthand_tokens(&mut tokens, components, custom)?;
    Some(tokens)
}

fn append_shorthand_tokens(
    tokens: &mut String,
    components: &[ContainerComponent],
    custom: &FastMap<InternedSymbol, PropertyValue>,
) -> Option<()> {
    for component in components {
        let fragment = match component {
            ContainerComponent::Literal(text) => text,
            ContainerComponent::Variable { name, fallback } => {
                container_variable_tokens(*name, fallback.as_deref(), custom)?
            }
            ContainerComponent::Function { name, args } => {
                if !tokens.is_empty() {
                    tokens.push(' ');
                }
                tokens.push_str(name);
                tokens.push('(');
                append_shorthand_tokens(tokens, args, custom)?;
                tokens.push(')');
                continue;
            }
            ContainerComponent::VariableWithTemplateFallback { name, fallback } => {
                match shorthand_variable(*name, custom)? {
                    Some(tokens) => tokens,
                    None => {
                        append_shorthand_tokens(tokens, fallback, custom)?;
                        continue;
                    }
                }
            }
        };
        // Keep adjacent fragments from merging into a different CSS token.
        if !tokens.is_empty() {
            tokens.push(' ');
        }
        tokens.push_str(fragment);
    }
    Some(())
}

fn container_variable_tokens<'a>(
    name: InternedSymbol,
    fallback: Option<&'a str>,
    custom: &'a FastMap<InternedSymbol, PropertyValue>,
) -> Option<&'a str> {
    shorthand_variable(name, custom)?.or(fallback)
}

fn shorthand_variable(
    name: InternedSymbol,
    custom: &FastMap<InternedSymbol, PropertyValue>,
) -> Option<Option<&str>> {
    match custom
        .get(&name)
        .and_then(|value| substitute(value, custom))
    {
        Some(PropertyValue::RawTokens(tokens)) if !tokens.eq_ignore_ascii_case("initial") => {
            Some(Some(tokens))
        }
        Some(PropertyValue::RawTokens(_) | PropertyValue::Initial) | None => Some(None),
        _ => None,
    }
}

fn container_longhand(
    property: PropertyId,
    tokens: &str,
    parser: Option<&dyn CustomValueParser>,
) -> Option<PropertyValue> {
    let (name, kind) = parser?.parse_container(tokens)?;
    let value = match property {
        PropertyId::ContainerName => name,
        PropertyId::ContainerType => kind,
        _ => return None,
    };
    fits(property, &value).then_some(value)
}

#[doc = include_str!("../../docs/api/computed/substituted_value.doc/fits.md")]
//
// Two arms answer `false` for reasons that have nothing to do with each other —
// a reference that never resolved, and a string that is no element's computed
// value — so merging them would cost the reader both explanations.
#[allow(clippy::match_same_arms)]
fn fits(property: PropertyId, value: &PropertyValue) -> bool {
    match value {
        // A CSS-wide keyword is part of every property's grammar, and an
        // unresolvable reference has already been spelled `unset` upstream.
        PropertyValue::Inherit | PropertyValue::Initial | PropertyValue::Unset => true,
        // A reference that substituted to another reference never resolved:
        // `substitute` follows chains, so what is left is a name nothing
        // declared.
        PropertyValue::CustomRef(_)
        | PropertyValue::ContainerRef { .. }
        | PropertyValue::ContainerTemplate(_)
        | PropertyValue::FlexFlowTemplate(_)
        | PropertyValue::GapTemplate(_)
        | PropertyValue::VariableTemplate(_) => false,
        // Unreachable: the text was handed to the grammar above, and what comes
        // back is a parsed value or an `unset`. Spelled out rather than folded
        // into a catch-all so the family stays exhaustive.
        PropertyValue::RawTokens(_) => false,
        PropertyValue::Color(_) => takes_color(property),
        PropertyValue::Length(_) | PropertyValue::Calc(_) => takes_length(property),
        PropertyValue::Number(n) if property == PropertyId::Order => n.fract() == 0.0,
        PropertyValue::Number(_) => takes_number(property),
        PropertyValue::Keyword(_) => takes_keyword(property),
        PropertyValue::FontFamilies(_) => matches!(property, PropertyId::FontFamily),
        PropertyValue::LinearGradient(_)
        | PropertyValue::RadialGradient(_)
        | PropertyValue::ConicGradient(_)
        | PropertyValue::BackgroundImageUrl(_)
        | PropertyValue::BackgroundLayers(_) => matches!(property, PropertyId::BackgroundImage),
        PropertyValue::BackgroundSize(_) | PropertyValue::BackgroundSizes(_) => {
            matches!(property, PropertyId::BackgroundSize)
        }
        PropertyValue::BackgroundPositions(_) => matches!(property, PropertyId::BackgroundPosition),
        // One `<position>` grammar, two properties that take it.
        PropertyValue::BackgroundPosition(_) => matches!(
            property,
            PropertyId::BackgroundPosition | PropertyId::ObjectPosition
        ),
        PropertyValue::BackgroundAttachments(_) => {
            matches!(property, PropertyId::BackgroundAttachment)
        }
        PropertyValue::BackgroundOrigins(_) => matches!(property, PropertyId::BackgroundOrigin),
        PropertyValue::ClipPolygon { .. }
        | PropertyValue::ClipEllipse { .. }
        | PropertyValue::ClipRectangle { .. }
        | PropertyValue::ClipBox(_)
        | PropertyValue::ClipSvgPath { .. } => {
            matches!(property, PropertyId::ClipPath)
        }
        PropertyValue::BackgroundClips(_) => matches!(property, PropertyId::BackgroundClip),
        PropertyValue::BackgroundBlendModes(_) => {
            matches!(property, PropertyId::BackgroundBlendMode)
        }
        PropertyValue::BackgroundRepeat(_) | PropertyValue::BackgroundRepeats(_) => {
            matches!(property, PropertyId::BackgroundRepeat)
        }
        PropertyValue::ObjectFit(_) => matches!(property, PropertyId::ObjectFit),
        PropertyValue::BoxShadow(_) => matches!(property, PropertyId::BoxShadow),
        PropertyValue::Filter(_) | PropertyValue::FilterFunctions(_) => {
            matches!(property, PropertyId::Filter | PropertyId::BackdropFilter)
        }
        PropertyValue::Transform(_) | PropertyValue::TransformFunctions(_) => {
            matches!(property, PropertyId::Transform)
        }
        PropertyValue::TransformOrigin(_) => matches!(property, PropertyId::TransformOrigin),
        PropertyValue::GridTracks(_) => matches!(
            property,
            PropertyId::GridTemplateColumns
                | PropertyId::GridTemplateRows
                | PropertyId::GridAutoColumns
                | PropertyId::GridAutoRows
        ),
        PropertyValue::GridLine(_) => matches!(
            property,
            PropertyId::GridColumnStart
                | PropertyId::GridColumnEnd
                | PropertyId::GridRowStart
                | PropertyId::GridRowEnd
        ),
        PropertyValue::GridAreas(_) => matches!(property, PropertyId::GridTemplateAreas),
        PropertyValue::GridAreaName(_) => matches!(property, PropertyId::GridArea),
        PropertyValue::Name(_) => matches!(
            property,
            PropertyId::Painter | PropertyId::ContainerName | PropertyId::ContainerType
        ),
        PropertyValue::ContainerNames(_) => property == PropertyId::ContainerName,
        // A quoted string is only ever `content`'s (§4.16), and `content` has
        // no computed value at all — `ComputedValue::from_concrete` says so.
        PropertyValue::String(_) => false,
    }
}

/// The properties whose value is a `<color>`.
fn takes_color(property: PropertyId) -> bool {
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

/// The properties whose value is a `<length>` (or a `<percentage>`, or a math
/// function over either — all three are one `PropertyValue` family here).
fn takes_length(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::Width
            | PropertyId::Height
            | PropertyId::MinWidth
            | PropertyId::MinHeight
            | PropertyId::MaxWidth
            | PropertyId::MaxHeight
            | PropertyId::Top
            | PropertyId::Right
            | PropertyId::Bottom
            | PropertyId::Left
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
            | PropertyId::LineHeight
            | PropertyId::LetterSpacing
            | PropertyId::WordSpacing
            | PropertyId::TabSize
            // A logical longhand resolves onto its physical counterpart before
            // it reaches here (`cascaded_for`), so these arms are unreachable
            // in practice — listed so the family stays complete if that order
            // ever changes.
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
    )
}

#[doc = include_str!("../../docs/api/computed/substituted_value.doc/takes_number.md")]
fn takes_number(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::Opacity
            | PropertyId::FlexGrow
            | PropertyId::FlexShrink
            | PropertyId::FontWeight
            | PropertyId::ZIndex
            // Both take a `<number>` as well as a `<length>`: a unitless
            // `line-height` multiplies the font size, a unitless `tab-size`
            // counts spaces.
            | PropertyId::LineHeight
            | PropertyId::TabSize
    )
}

#[doc = include_str!("../../docs/api/computed/substituted_value.doc/takes_keyword.md")]
fn takes_keyword(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::Display
            | PropertyId::Position
            | PropertyId::Visibility
            | PropertyId::BoxSizing
            | PropertyId::Direction
            | PropertyId::WritingMode
            | PropertyId::FlexDirection
            | PropertyId::FlexWrap
            | PropertyId::JustifyContent
            | PropertyId::AlignContent
            | PropertyId::AlignItems
            | PropertyId::JustifyItems
            | PropertyId::AlignSelf
            | PropertyId::JustifySelf
            | PropertyId::GridAutoFlow
            | PropertyId::TextAlign
            | PropertyId::TextOverflow
            | PropertyId::TextTransform
            | PropertyId::WhiteSpace
            | PropertyId::OverflowWrap
            | PropertyId::WordBreak
            | PropertyId::OverflowX
            | PropertyId::OverflowY
            | PropertyId::PointerEvents
            | PropertyId::PixelSnap
            | PropertyId::OverscrollBehaviorX
            | PropertyId::OverscrollBehaviorY
            | PropertyId::MixBlendMode
            | PropertyId::Isolation
            | PropertyId::FontStyle
            // `bolder` / `lighter` / `bold` / `normal`, resolved against the
            // parent one step further on.
            | PropertyId::FontWeight
            // `z-index: auto`.
            | PropertyId::ZIndex
            | PropertyId::BorderTopStyle
            | PropertyId::BorderRightStyle
            | PropertyId::BorderBottomStyle
            | PropertyId::BorderLeftStyle
            | PropertyId::BorderInlineStartStyle
            | PropertyId::BorderInlineEndStyle
            | PropertyId::BorderBlockStartStyle
            | PropertyId::BorderBlockEndStyle
            | PropertyId::OutlineStyle
            // The intrinsic sizing keywords (`min-content`, `max-content`,
            // `fit-content`) are a keyword on a property that is otherwise
            // length-valued, so a length longhand has to accept a keyword too.
            | PropertyId::Width
            | PropertyId::Height
            | PropertyId::MinWidth
            | PropertyId::MinHeight
            | PropertyId::MaxWidth
            | PropertyId::MaxHeight
            | PropertyId::FlexBasis
            // `gap: normal`, `letter-spacing: normal`, `word-spacing: normal`.
            | PropertyId::Gap
            | PropertyId::RowGap
            | PropertyId::ColumnGap
            | PropertyId::LetterSpacing
            | PropertyId::WordSpacing
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::property::{Length, StyleColor};
    use obscur_identity::InternedSymbol;

    #[test]
    fn a_value_of_the_wrong_shape_becomes_unset_rather_than_being_kept() {
        // The shape of the regression: a `var()` that reaches `font-family`
        // carrying a length is not a family list, and keeping it would make the
        // property read as absent — a third behaviour CSS does not have.
        assert_eq!(
            substituted_value(
                PropertyId::FontFamily,
                &PropertyValue::Length(Length::px(12.0)),
                None
            ),
            PropertyValue::Unset
        );
        assert_eq!(
            substituted_value(
                PropertyId::Color,
                &PropertyValue::Keyword(InternedSymbol::from_raw(3)),
                None
            ),
            PropertyValue::Unset
        );
    }

    #[test]
    fn a_value_of_the_right_shape_is_carried_through_untouched() {
        let red = PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(substituted_value(PropertyId::Color, &red, None), red);
        let width = PropertyValue::Length(Length::px(4.0));
        assert_eq!(substituted_value(PropertyId::Width, &width, None), width);
        let families = PropertyValue::font_families(["Georgia"]);
        assert_eq!(
            substituted_value(PropertyId::FontFamily, &families, None),
            families
        );
    }

    #[test]
    fn a_css_wide_keyword_is_valid_on_every_property() {
        // `substitute` spells an unresolvable reference `unset`; judging that
        // against a grammar would turn "the name is missing" into a second,
        // different answer for properties whose grammar has no keywords.
        for property in [PropertyId::FontFamily, PropertyId::Transform] {
            for keyword in [
                PropertyValue::Inherit,
                PropertyValue::Initial,
                PropertyValue::Unset,
            ] {
                assert_eq!(substituted_value(property, &keyword, None), keyword);
            }
        }
    }

    /// A stand-in for the CSS grammar: `font-family` takes a comma-separated
    /// family list, `color` takes only the word `red`, and nothing else parses.
    fn grammar(property: PropertyId, tokens: &str) -> Option<PropertyValue> {
        match property {
            PropertyId::FontFamily => Some(PropertyValue::font_families(
                tokens.split(',').map(str::trim),
            )),
            PropertyId::Color if tokens == "red" => {
                Some(PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)))
            }
            _ => None,
        }
    }

    #[test]
    fn declared_text_is_parsed_for_the_property_it_landed_on() {
        // The defect this closes: the whole token sequence survives the
        // cascade, and the property it lands on is what finally reads it.
        let parser = grammar;
        assert_eq!(
            substituted_value(
                PropertyId::FontFamily,
                &PropertyValue::RawTokens("Inter, sans-serif".into()),
                Some(&parser)
            ),
            PropertyValue::font_families(["Inter", "sans-serif"])
        );
    }

    #[test]
    fn declared_text_no_grammar_can_read_is_unset() {
        let parser = grammar;
        // `color` accepts only `red` here, so the same text that made a family
        // list above is invalid at computed-value time on a colour longhand.
        assert_eq!(
            substituted_value(
                PropertyId::Color,
                &PropertyValue::RawTokens("Inter, sans-serif".into()),
                Some(&parser)
            ),
            PropertyValue::Unset
        );
    }

    #[test]
    fn declared_text_with_no_parser_installed_is_unset_rather_than_kept() {
        // A runtime with no CSS pipeline has no grammar to read the text with.
        // Keeping the text on the property would leave a computed value no
        // consumer can read, which is absence — the third behaviour CSS does
        // not have.
        assert_eq!(
            substituted_value(
                PropertyId::FontFamily,
                &PropertyValue::RawTokens("Inter, sans-serif".into()),
                None
            ),
            PropertyValue::Unset
        );
    }
}
