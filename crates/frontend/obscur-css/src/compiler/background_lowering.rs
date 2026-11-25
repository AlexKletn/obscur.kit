#![doc = include_str!("../../docs/api/compiler/background_lowering.doc/_module.md")]

mod position_component;

use position_component::{PositionComponent, position_component};

use super::CssSymbolInterner;
use super::color_lowering::parse_stop_color;
use super::gradient_lowering::{
    lower_conic_gradient, lower_linear_gradient, lower_radial_gradient,
    lower_repeating_linear_gradient,
};
use super::value_lowering::{css_wide_keyword, dimension_length, sole_custom_ref};
use super::{
    CompiledDeclaration,
    property_compiler::{narrow_f32, value_unsupported_kind},
};
use crate::error::UnsupportedCssFeature;
use crate::parser::{ParsedDeclaration, ParsedValue};
use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{
    BackgroundAttachment, BackgroundBox, BackgroundLayer, BackgroundPosition, BackgroundRepeat,
    BackgroundRepeatAxis, BackgroundSize, Length, ObjectFit, PropertyId, PropertyValue,
};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

#[doc = include_str!("../../docs/api/compiler/background_lowering.doc/expand_background.md")]
pub(super) fn expand_background(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
    _interner: &mut CssSymbolInterner,
) -> Vec<CompiledDeclaration> {
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    let standard = |property, value| {
        CompiledDeclaration::Standard(Declaration::new(
            property,
            value,
            important,
            origin,
            specificity,
            source_order,
            selector,
        ))
    };
    let longhands = [
        PropertyId::BackgroundImage,
        PropertyId::BackgroundColor,
        PropertyId::BackgroundSize,
        PropertyId::BackgroundPosition,
        PropertyId::BackgroundRepeat,
        PropertyId::BackgroundBlendMode,
        PropertyId::BackgroundClip,
        PropertyId::BackgroundOrigin,
        PropertyId::BackgroundAttachment,
    ];
    if let [ParsedValue::Keyword(keyword)] = decl.value.as_slice()
        && let Some(wide) = css_wide_keyword(keyword)
    {
        return longhands
            .into_iter()
            .map(|id| standard(id, wide.clone()))
            .collect();
    }

    let expanded = if let Some(reference) = sole_custom_ref(&decl.value) {
        // The existing shorthand custom-property path carries a color value.
        Some([
            PropertyValue::Initial,
            reference,
            PropertyValue::Initial,
            PropertyValue::Initial,
            PropertyValue::Initial,
            PropertyValue::Initial,
            PropertyValue::Initial,
            PropertyValue::Initial,
            PropertyValue::Initial,
        ])
    } else {
        lower_background_shorthand(&decl.value)
    };
    if let Some(values) = expanded {
        return longhands
            .into_iter()
            .zip(values)
            .map(|(id, value)| standard(id, value))
            .collect();
    }
    vec![CompiledDeclaration::Unsupported(
        UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
    )]
}

#[derive(Default)]
struct ShorthandLayer {
    image: Option<BackgroundLayer>,
    color: Option<PropertyValue>,
    size: Option<BackgroundSize>,
    position: Option<BackgroundPosition>,
    repeat: Option<BackgroundRepeat>,
    origin: Option<BackgroundBox>,
    clip: Option<BackgroundBox>,
    attachment: Option<BackgroundAttachment>,
}

fn lower_background_shorthand(values: &[ParsedValue]) -> Option<[PropertyValue; 9]> {
    let mut segments = values
        .split(|value| matches!(value, ParsedValue::Comma))
        .peekable();
    let mut layers = Vec::new();
    while let Some(segment) = segments.next() {
        layers.push(lower_shorthand_layer(segment, segments.peek().is_none())?);
    }
    let color = layers
        .last_mut()?
        .color
        .take()
        .unwrap_or(PropertyValue::Initial);
    let image = if layers.len() == 1 {
        layers[0]
            .image
            .take()
            .unwrap_or(BackgroundLayer::None)
            .into_property_value()
    } else {
        PropertyValue::BackgroundLayers(
            layers
                .iter_mut()
                .map(|layer| layer.image.take().unwrap_or(BackgroundLayer::None))
                .collect(),
        )
    };
    Some([
        image,
        color,
        shorthand_placement(
            layers.iter().map(|layer| layer.size),
            PropertyValue::BackgroundSize,
            PropertyValue::BackgroundSizes,
        ),
        shorthand_placement(
            layers.iter().map(|layer| layer.position),
            PropertyValue::BackgroundPosition,
            PropertyValue::BackgroundPositions,
        ),
        shorthand_placement(
            layers.iter().map(|layer| layer.repeat),
            PropertyValue::BackgroundRepeat,
            PropertyValue::BackgroundRepeats,
        ),
        PropertyValue::Initial,
        if layers.iter().any(|layer| layer.clip.is_some()) {
            PropertyValue::BackgroundClips(
                layers
                    .iter()
                    .map(|layer| layer.clip.unwrap_or(BackgroundBox::Border))
                    .collect(),
            )
        } else {
            PropertyValue::Initial
        },
        if layers.iter().any(|layer| layer.origin.is_some()) {
            PropertyValue::BackgroundOrigins(
                layers
                    .iter()
                    .map(|layer| layer.origin.unwrap_or(BackgroundBox::Padding))
                    .collect(),
            )
        } else {
            PropertyValue::Initial
        },
        if layers.iter().any(|layer| layer.attachment.is_some()) {
            PropertyValue::BackgroundAttachments(
                layers
                    .iter()
                    .map(|layer| layer.attachment.unwrap_or_default())
                    .collect(),
            )
        } else {
            PropertyValue::Initial
        },
    ])
}

pub(super) fn background_attachment(keyword: &str) -> Option<BackgroundAttachment> {
    if keyword.eq_ignore_ascii_case("scroll") {
        Some(BackgroundAttachment::Scroll)
    } else if keyword.eq_ignore_ascii_case("local") {
        Some(BackgroundAttachment::Local)
    } else if keyword.eq_ignore_ascii_case("fixed") {
        Some(BackgroundAttachment::Fixed)
    } else {
        None
    }
}

pub(super) fn background_box(keyword: &str) -> Option<BackgroundBox> {
    if keyword.eq_ignore_ascii_case("border-box") {
        Some(BackgroundBox::Border)
    } else if keyword.eq_ignore_ascii_case("padding-box") {
        Some(BackgroundBox::Padding)
    } else if keyword.eq_ignore_ascii_case("content-box") {
        Some(BackgroundBox::Content)
    } else {
        None
    }
}

fn shorthand_placement<T: Default>(
    values: impl Iterator<Item = Option<T>>,
    single: impl FnOnce(T) -> PropertyValue,
    multiple: impl FnOnce(Box<[T]>) -> PropertyValue,
) -> PropertyValue {
    let values: Vec<_> = values.collect();
    if values.iter().all(Option::is_none) {
        return PropertyValue::Initial;
    }
    if values.len() == 1 {
        return single(values.into_iter().next().flatten().unwrap_or_default());
    }
    multiple(values.into_iter().map(Option::unwrap_or_default).collect())
}

fn lower_shorthand_layer(mut values: &[ParsedValue], final_layer: bool) -> Option<ShorthandLayer> {
    if values.is_empty() {
        return None;
    }
    let mut layer = ShorthandLayer::default();
    while let Some((first, rest)) = values.split_first() {
        if layer.image.is_none()
            && let Some(image) = lower_background_layer(std::slice::from_ref(first))
        {
            layer.image = Some(image);
            values = rest;
        } else if final_layer
            && layer.color.is_none()
            && let Some(color) = parse_stop_color(first)
        {
            layer.color = Some(PropertyValue::Color(color));
            values = rest;
        } else if layer.repeat.is_none()
            && let Some((repeat, consumed)) = placement_prefix(values, lower_repeat_layer)
        {
            layer.repeat = Some(repeat);
            values = &values[consumed..];
        } else if layer.attachment.is_none()
            && let ParsedValue::Keyword(keyword) = first
            && let Some(attachment) = background_attachment(keyword)
        {
            layer.attachment = Some(attachment);
            values = rest;
        } else if layer.origin.is_none()
            && let ParsedValue::Keyword(keyword) = first
            && let Some(origin) = background_box(keyword)
        {
            layer.origin = Some(origin);
            values = rest;
            let second = match values.first() {
                Some(ParsedValue::Keyword(keyword)) => background_box(keyword),
                _ => None,
            };
            layer.clip = Some(second.unwrap_or(origin));
            if second.is_some() {
                values = &values[1..];
            }
        } else if layer.position.is_none() {
            let count = values
                .iter()
                .take_while(|value| position_component(value, position_length).is_some())
                .count();
            layer.position = Some(lower_position_layer(&values[..count])?);
            values = &values[count..];
            if let Some(after_slash) = values.strip_prefix(&[ParsedValue::Delimiter('/')]) {
                let (size, consumed) = placement_prefix(after_slash, lower_size_layer)?;
                layer.size = Some(size);
                values = &after_slash[consumed..];
            }
        } else {
            return None;
        }
    }
    Some(layer)
}

fn placement_prefix<T>(
    values: &[ParsedValue],
    lower: impl Fn(&[ParsedValue]) -> Option<T>,
) -> Option<(T, usize)> {
    [2, 1]
        .into_iter()
        .find_map(|count| lower(values.get(..count)?).map(|value| (value, count)))
}

#[doc = include_str!("../../docs/api/compiler/background_lowering.doc/lower_background_size.md")]
pub(super) fn lower_background_size(values: &[ParsedValue]) -> Option<PropertyValue> {
    lower_layer_values(
        values,
        lower_size_layer,
        PropertyValue::BackgroundSize,
        PropertyValue::BackgroundSizes,
    )
}

fn lower_size_layer(values: &[ParsedValue]) -> Option<BackgroundSize> {
    if let [ParsedValue::Keyword(keyword)] = values {
        if keyword.eq_ignore_ascii_case("cover") {
            return Some(BackgroundSize::Cover);
        }
        if keyword.eq_ignore_ascii_case("contain") {
            return Some(BackgroundSize::Contain);
        }
        if keyword.eq_ignore_ascii_case("auto") {
            return Some(BackgroundSize::Auto);
        }
    }
    // One or two `<length-percentage> | auto` components.
    let components: Vec<&ParsedValue> = values.iter().collect();
    let (width, height) = match components.as_slice() {
        [a] => (background_size_component(a)?, Length::Auto),
        [a, b] => (background_size_component(a)?, background_size_component(b)?),
        _ => return None,
    };
    // Both axes `auto` is exactly the intrinsic-size initial.
    if width.is_auto() && height.is_auto() {
        return Some(BackgroundSize::Auto);
    }
    Some(BackgroundSize::Explicit { width, height })
}

/// Lowers one `background-size` component into a [`Length`]: `auto` →
/// [`Length::Auto`], a percentage / dimension → its length, a unitless `0` →
/// `0px`. Any other component (including `cover` / `contain`, which are only
/// valid alone) yields [`None`].
fn background_size_component(value: &ParsedValue) -> Option<Length> {
    match value {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto") => Some(Length::Auto),
        ParsedValue::Percentage(p) => Some(Length::Percent(narrow_f32(*p))),
        ParsedValue::Dimension { value, unit } => dimension_length(*value, unit),
        ParsedValue::Number(n) if *n == 0.0 => Some(Length::px(0.0)),
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/background_lowering.doc/lower_object_fit.md")]
pub(super) fn lower_object_fit(values: &[ParsedValue]) -> Option<PropertyValue> {
    let [ParsedValue::Keyword(keyword)] = values else {
        return None;
    };
    let fit = if keyword.eq_ignore_ascii_case("fill") {
        ObjectFit::Fill
    } else if keyword.eq_ignore_ascii_case("contain") {
        ObjectFit::Contain
    } else if keyword.eq_ignore_ascii_case("cover") {
        ObjectFit::Cover
    } else if keyword.eq_ignore_ascii_case("none") {
        ObjectFit::None
    } else if keyword.eq_ignore_ascii_case("scale-down") {
        ObjectFit::ScaleDown
    } else {
        return None;
    };
    Some(PropertyValue::ObjectFit(fit))
}

#[doc = include_str!("../../docs/api/compiler/background_lowering.doc/lower_background_position.md")]
pub(super) fn lower_background_position(values: &[ParsedValue]) -> Option<PropertyValue> {
    lower_layer_values(
        values,
        lower_position_layer,
        PropertyValue::BackgroundPosition,
        PropertyValue::BackgroundPositions,
    )
}

pub(super) fn lower_object_position(values: &[ParsedValue]) -> Option<PropertyValue> {
    if values.len() == 3 {
        return None;
    }
    lower_position_layer(values).map(PropertyValue::BackgroundPosition)
}

pub(super) fn lower_position_layer(values: &[ParsedValue]) -> Option<BackgroundPosition> {
    let ([x, y], from_end) = lower_position(values, position_length, |length| length)?;
    Some(BackgroundPosition::from_edge_offsets(x, y, from_end))
}

pub(super) fn lower_position<T: Clone>(
    values: &[ParsedValue],
    lower: impl Fn(&ParsedValue) -> Option<T>,
    lift: impl Fn(Length) -> T,
) -> Option<([T; 2], [bool; 2])> {
    let components: Vec<PositionComponent<T>> = values
        .iter()
        .map(|value| position_component(value, &lower))
        .collect::<Option<Vec<_>>>()?;
    let (x, y) = match components.as_slice() {
        [only] => match only {
            PositionComponent::Horizontal(p) => {
                (lift(Length::Percent(*p)), lift(Length::Percent(50.0)))
            }
            PositionComponent::Vertical(p) => {
                (lift(Length::Percent(50.0)), lift(Length::Percent(*p)))
            }
            PositionComponent::Center => (lift(Length::Percent(50.0)), lift(Length::Percent(50.0))),
            PositionComponent::Length(l) => (l.clone(), lift(Length::Percent(50.0))),
        },
        [
            PositionComponent::Length(_),
            PositionComponent::Horizontal(_),
        ]
        | [PositionComponent::Vertical(_), PositionComponent::Length(_)] => return None,
        [_, _] => {
            let mut x: Option<T> = None;
            let mut y: Option<T> = None;
            // First pass: explicit-axis keywords claim their axis (order-free).
            for comp in &components {
                match comp {
                    PositionComponent::Horizontal(p) if x.is_none() => {
                        x = Some(lift(Length::Percent(*p)))
                    }
                    PositionComponent::Vertical(p) if y.is_none() => {
                        y = Some(lift(Length::Percent(*p)))
                    }
                    PositionComponent::Horizontal(_) | PositionComponent::Vertical(_) => {
                        return None;
                    }
                    PositionComponent::Center | PositionComponent::Length(_) => {}
                }
            }
            // Second pass: `center` / lengths fill the remaining axes in order.
            for comp in &components {
                let value = match comp {
                    PositionComponent::Center => lift(Length::Percent(50.0)),
                    PositionComponent::Length(l) => l.clone(),
                    // Axis keywords were placed above.
                    PositionComponent::Horizontal(_) | PositionComponent::Vertical(_) => continue,
                };
                if x.is_none() {
                    x = Some(value);
                } else if y.is_none() {
                    y = Some(value);
                }
            }
            (
                x.unwrap_or(lift(Length::Percent(50.0))),
                y.unwrap_or(lift(Length::Percent(50.0))),
            )
        }
        [_, _, _] | [_, _, _, _] => return lower_edge_position(&components, lift),
        _ => return None,
    };
    Some(([x, y], [false; 2]))
}

fn lower_edge_position<T: Clone>(
    mut components: &[PositionComponent<T>],
    lift: impl Fn(Length) -> T,
) -> Option<([T; 2], [bool; 2])> {
    let mut x = None;
    let mut y = None;
    let mut center = false;
    while let Some((edge, rest)) = components.split_first() {
        components = rest;
        let axis = match edge {
            PositionComponent::Horizontal(side) if x.is_none() => (&mut x, *side),
            PositionComponent::Vertical(side) if y.is_none() => (&mut y, *side),
            PositionComponent::Center if !center => {
                center = true;
                continue;
            }
            _ => return None,
        };
        let offset = if let Some((PositionComponent::Length(offset), remaining)) =
            components.split_first()
        {
            components = remaining;
            offset.clone()
        } else {
            lift(Length::px(0.0))
        };
        *axis.0 = Some((offset, axis.1 == 100.0));
    }
    let (x, y) = match (x, y, center) {
        (Some(x), Some(y), false) => (x, y),
        (Some(x), None, true) => (x, (lift(Length::Percent(50.0)), false)),
        (None, Some(y), true) => ((lift(Length::Percent(50.0)), false), y),
        _ => return None,
    };
    Some(([x.0, y.0], [x.1, y.1]))
}

fn position_length(value: &ParsedValue) -> Option<Length> {
    match value {
        ParsedValue::Percentage(p) => Some(Length::Percent(narrow_f32(*p))),
        ParsedValue::Dimension { value, unit } => dimension_length(*value, unit),
        ParsedValue::Number(n) if *n == 0.0 => Some(Length::px(0.0)),
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/background_lowering.doc/lower_background_repeat.md")]
pub(super) fn lower_background_repeat(values: &[ParsedValue]) -> Option<PropertyValue> {
    lower_layer_values(
        values,
        lower_repeat_layer,
        PropertyValue::BackgroundRepeat,
        PropertyValue::BackgroundRepeats,
    )
}

fn lower_layer_values<T>(
    values: &[ParsedValue],
    lower: impl Fn(&[ParsedValue]) -> Option<T>,
    single: impl FnOnce(T) -> PropertyValue,
    list: impl FnOnce(Box<[T]>) -> PropertyValue,
) -> Option<PropertyValue> {
    if !values.contains(&ParsedValue::Comma) {
        return lower(values).map(single);
    }
    values
        .split(|value| matches!(value, ParsedValue::Comma))
        .map(lower)
        .collect::<Option<Box<[_]>>>()
        .map(list)
}

fn lower_repeat_layer(values: &[ParsedValue]) -> Option<BackgroundRepeat> {
    use BackgroundRepeatAxis::{NoRepeat, Repeat};
    let (x, y) = match values {
        [ParsedValue::Keyword(keyword)] if keyword.eq_ignore_ascii_case("repeat-x") => {
            (Repeat, NoRepeat)
        }
        [ParsedValue::Keyword(keyword)] if keyword.eq_ignore_ascii_case("repeat-y") => {
            (NoRepeat, Repeat)
        }
        [ParsedValue::Keyword(keyword)] => {
            let axis = repeat_axis(keyword)?;
            (axis, axis)
        }
        [ParsedValue::Keyword(x), ParsedValue::Keyword(y)] => (repeat_axis(x)?, repeat_axis(y)?),
        _ => return None,
    };
    Some(match (x, y) {
        (Repeat, Repeat) => BackgroundRepeat::Repeat,
        (NoRepeat, NoRepeat) => BackgroundRepeat::NoRepeat,
        (Repeat, NoRepeat) => BackgroundRepeat::RepeatX,
        (NoRepeat, Repeat) => BackgroundRepeat::RepeatY,
        _ => BackgroundRepeat::Axes(x, y),
    })
}

fn repeat_axis(keyword: &str) -> Option<BackgroundRepeatAxis> {
    use BackgroundRepeatAxis::{NoRepeat, Repeat, Round, Space};
    if keyword.eq_ignore_ascii_case("repeat") {
        Some(Repeat)
    } else if keyword.eq_ignore_ascii_case("no-repeat") {
        Some(NoRepeat)
    } else if keyword.eq_ignore_ascii_case("space") {
        Some(Space)
    } else if keyword.eq_ignore_ascii_case("round") {
        Some(Round)
    } else {
        None
    }
}

#[doc = include_str!("../../docs/api/compiler/background_lowering.doc/lower_background_image.md")]
pub(super) fn lower_background_image(values: &[ParsedValue]) -> Option<PropertyValue> {
    let mut layers: Vec<BackgroundLayer> = Vec::new();
    for segment in values.split(|v| matches!(v, ParsedValue::Comma)) {
        layers.push(lower_background_layer(segment)?);
    }
    match layers.len() {
        0 => None,
        // One layer keeps today's exact flat single-value result.
        1 => Some(
            layers
                .pop()
                .expect("one layer present")
                .into_property_value(),
        ),
        _ => Some(PropertyValue::BackgroundLayers(layers.into_boxed_slice())),
    }
}

#[doc = include_str!("../../docs/api/compiler/background_lowering.doc/lower_background_layer.md")]
fn lower_background_layer(values: &[ParsedValue]) -> Option<BackgroundLayer> {
    match values {
        [ParsedValue::Keyword(keyword)] if keyword.eq_ignore_ascii_case("none") => {
            Some(BackgroundLayer::None)
        }
        [ParsedValue::Function { name, args }] if name.eq_ignore_ascii_case("linear-gradient") => {
            lower_linear_gradient(args).map(BackgroundLayer::Linear)
        }
        [ParsedValue::Function { name, args }]
            if name.eq_ignore_ascii_case("repeating-linear-gradient") =>
        {
            lower_repeating_linear_gradient(args).map(BackgroundLayer::Linear)
        }
        [ParsedValue::Function { name, args }] if name.eq_ignore_ascii_case("radial-gradient") => {
            lower_radial_gradient(args, false).map(BackgroundLayer::Radial)
        }
        [ParsedValue::Function { name, args }]
            if name.eq_ignore_ascii_case("repeating-radial-gradient") =>
        {
            lower_radial_gradient(args, true).map(BackgroundLayer::Radial)
        }
        [ParsedValue::Function { name, args }] if name.eq_ignore_ascii_case("conic-gradient") => {
            lower_conic_gradient(args).map(BackgroundLayer::Conic)
        }
        [ParsedValue::Function { name, args }] if name.eq_ignore_ascii_case("url") => {
            lower_url(args).map(BackgroundLayer::Url)
        }
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/background_lowering.doc/lower_url.md")]
fn lower_url(args: &[ParsedValue]) -> Option<Box<str>> {
    if let [ParsedValue::String(s)] = args {
        // The clean, quoted form: the string content is the URL verbatim.
        return (!s.is_empty()).then(|| s.clone());
    }
    let mut url = String::new();
    for arg in args {
        append_url_component(&mut url, arg);
    }
    (!url.is_empty()).then(|| url.into_boxed_str())
}

/// Appends one component of an unquoted `url(...)` body to `out` in its authored
/// textual form (a best-effort reconstruction, since the tokenizer splits an
/// unquoted URL into ident/number/delimiter pieces).
fn append_url_component(out: &mut String, value: &ParsedValue) {
    use std::fmt::Write as _;
    match value {
        ParsedValue::Keyword(text) | ParsedValue::String(text) => out.push_str(text),
        ParsedValue::Hash(text) => {
            out.push('#');
            out.push_str(text);
        }
        ParsedValue::Number(n) => {
            let _ = write!(out, "{n}");
        }
        ParsedValue::Percentage(p) => {
            let _ = write!(out, "{p}%");
        }
        ParsedValue::Dimension { value, unit } => {
            let _ = write!(out, "{value}{unit}");
        }
        ParsedValue::Delimiter(ch) => out.push(*ch),
        // Commas separate multiple images (out of scope) and bracketed line-name
        // groups never appear in a URL; both contribute nothing.
        ParsedValue::Comma | ParsedValue::LineNames(_) | ParsedValue::Function { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::super::color_lowering::tests::color;
    use super::super::gradient_lowering::tests::gradient_of;
    use super::super::property_compiler::tests::{compile, compile_all, standard_of, value_of};

    use crate::compiler::CompiledDeclaration;

    use obscur_style::property::{
        BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize, Length, PropertyId,
        PropertyValue,
    };

    #[test]
    fn single_gradient_stays_one_flat_layer_byte_identical() {
        // One layer keeps the flat single-value result — NOT a `BackgroundLayers`.
        let value = value_of("background-image: linear-gradient(45deg, red, blue)");
        assert!(matches!(value, PropertyValue::LinearGradient(_)));
        assert_eq!(value.as_background_layers(), None);
    }

    #[test]
    fn none_consumes_its_slot_in_an_image_list() {
        let value = value_of(r#"background-image: none, url("x.png"), none"#);
        assert_eq!(
            value.as_background_layers().unwrap(),
            [
                BackgroundLayer::None,
                BackgroundLayer::Url("x.png".into()),
                BackgroundLayer::None
            ]
        );
        assert_eq!(value_of("background-image: none"), PropertyValue::Initial);
    }

    #[test]
    fn two_layers_gradient_then_url_lower_to_a_layer_list_in_order() {
        // `gradient, url` → two layers, in author order (gradient first = top).
        let value =
            value_of(r#"background-image: linear-gradient(90deg, red, blue), url("x.png")"#);
        let layers = value
            .as_background_layers()
            .expect("two layers lower to a BackgroundLayers list");
        assert_eq!(layers.len(), 2);
        match &layers[0] {
            BackgroundLayer::Linear(g) => assert_eq!(g.angle_deg(), 90.0),
            other => panic!("expected a linear-gradient top layer, got {other:?}"),
        }
        assert_eq!(layers[1], BackgroundLayer::Url("x.png".into()));
    }

    #[test]
    fn top_level_comma_split_does_not_split_inside_rgb_args() {
        // The comma inside `rgb(0, 128, 255)` is a FUNCTION-argument comma, not a
        // layer separator: this must stay ONE gradient layer with the rgb color,
        // never two layers.
        let value = value_of("background-image: linear-gradient(rgb(0, 128, 255), rgb(255, 0, 0))");
        match value {
            PropertyValue::LinearGradient(g) => {
                assert_eq!(g.stops().len(), 2);
                let first = g.stops()[0].color();
                assert!((first.r - 0.0).abs() < 1e-6);
                assert!((first.b - 1.0).abs() < 1e-6);
            }
            other => panic!("expected a single linear gradient, got {other:?}"),
        }
    }

    #[test]
    fn three_repeating_and_url_layers_preserve_kind_and_order() {
        // The graph-paper grid form plus a url: three layers top→bottom.
        let value = value_of(concat!(
            "background-image: repeating-linear-gradient(#e2e0d9 0 1px, transparent 1px 24px), ",
            "repeating-linear-gradient(90deg, #e2e0d9 0 1px, transparent 1px 24px), ",
            r#"url("bg.png")"#
        ));
        let layers = value.as_background_layers().expect("three layers");
        assert_eq!(layers.len(), 3);
        assert!(matches!(layers[0], BackgroundLayer::Linear(_)));
        assert!(matches!(layers[1], BackgroundLayer::Linear(_)));
        assert_eq!(layers[2], BackgroundLayer::Url("bg.png".into()));
    }

    #[test]
    fn quoted_url_background_image_lowers_to_url() {
        assert_eq!(
            value_of(r#"background-image: url("foo.png")"#),
            PropertyValue::BackgroundImageUrl("foo.png".into())
        );
        // Single quotes work the same way.
        assert_eq!(
            value_of("background-image: url('bar/baz.png')"),
            PropertyValue::BackgroundImageUrl("bar/baz.png".into())
        );
    }

    #[test]
    fn unquoted_url_background_image_lowers_to_url() {
        assert_eq!(
            value_of("background-image: url(bar.png)"),
            PropertyValue::BackgroundImageUrl("bar.png".into())
        );
        // A relative path with directory separators round-trips too.
        assert_eq!(
            value_of("background-image: url(assets/logo.png)"),
            PropertyValue::BackgroundImageUrl("assets/logo.png".into())
        );
    }

    #[test]
    fn empty_url_background_image_is_unsupported() {
        assert!(matches!(
            compile("background-image: url()").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }

    #[test]
    fn background_size_keywords_and_lengths() {
        assert_eq!(
            value_of("background-size: cover"),
            PropertyValue::BackgroundSize(BackgroundSize::Cover)
        );
        assert_eq!(
            value_of("background-size: contain"),
            PropertyValue::BackgroundSize(BackgroundSize::Contain)
        );
        assert_eq!(
            value_of("background-size: auto"),
            PropertyValue::BackgroundSize(BackgroundSize::Auto)
        );
        // Two explicit percentages.
        assert_eq!(
            value_of("background-size: 50% 100%"),
            PropertyValue::BackgroundSize(BackgroundSize::Explicit {
                width: Length::Percent(50.0),
                height: Length::Percent(100.0),
            })
        );
        // A single length leaves the other axis `auto`.
        assert_eq!(
            value_of("background-size: 40px"),
            PropertyValue::BackgroundSize(BackgroundSize::Explicit {
                width: Length::px(40.0),
                height: Length::Auto,
            })
        );
        // Both axes `auto` collapses to the intrinsic-size initial.
        assert_eq!(
            value_of("background-size: auto auto"),
            PropertyValue::BackgroundSize(BackgroundSize::Auto)
        );
        // Malformed → dropped (unsupported), so the initial applies.
        assert!(matches!(
            compile("background-size: bogus").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }

    #[test]
    fn background_position_keywords_and_lengths() {
        // `right bottom` → x=100%, y=100% (order-free keyword axes).
        assert_eq!(
            value_of("background-position: right bottom"),
            PropertyValue::BackgroundPosition(BackgroundPosition::new(
                Length::Percent(100.0),
                Length::Percent(100.0)
            ))
        );
        // `bottom right` resolves identically (axis keywords are order-free).
        assert_eq!(
            value_of("background-position: bottom right"),
            value_of("background-position: right bottom")
        );
        // A single `center` centers both axes.
        assert_eq!(
            value_of("background-position: center"),
            PropertyValue::BackgroundPosition(BackgroundPosition::new(
                Length::Percent(50.0),
                Length::Percent(50.0)
            ))
        );
        // Two lengths, x then y.
        assert_eq!(
            value_of("background-position: 10px 20px"),
            PropertyValue::BackgroundPosition(BackgroundPosition::new(
                Length::px(10.0),
                Length::px(20.0)
            ))
        );
        // Malformed → dropped.
        assert!(matches!(
            compile("background-position: nope").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }

    #[test]
    fn background_geometry_lists_preserve_layer_order() {
        assert_eq!(
            value_of("background-size: contain, 20px 30%"),
            PropertyValue::BackgroundSizes(Box::new([
                BackgroundSize::Contain,
                BackgroundSize::Explicit {
                    width: Length::px(20.0),
                    height: Length::Percent(30.0)
                },
            ]))
        );
        assert_eq!(
            value_of("background-position: left top, right bottom"),
            PropertyValue::BackgroundPositions(Box::new([
                BackgroundPosition::new(Length::Percent(0.0), Length::Percent(0.0)),
                BackgroundPosition::new(Length::Percent(100.0), Length::Percent(100.0)),
            ]))
        );
        assert!(
            super::lower_object_position(&[
                crate::parser::ParsedValue::Keyword("left".into()),
                crate::parser::ParsedValue::Comma,
                crate::parser::ParsedValue::Keyword("right".into()),
            ])
            .is_none()
        );
    }

    #[test]
    fn repeat_lists_keep_each_layers_axis_pair() {
        assert_eq!(
            value_of("background-repeat: repeat no-repeat, no-repeat repeat"),
            PropertyValue::BackgroundRepeats(Box::new([
                BackgroundRepeat::RepeatX,
                BackgroundRepeat::RepeatY,
            ]))
        );
    }

    #[test]
    fn background_repeat_keywords() {
        assert_eq!(
            value_of("background-repeat: no-repeat"),
            PropertyValue::BackgroundRepeat(BackgroundRepeat::NoRepeat)
        );
        assert_eq!(
            value_of("background-repeat: repeat-x"),
            PropertyValue::BackgroundRepeat(BackgroundRepeat::RepeatX)
        );
        assert_eq!(
            value_of("background-repeat: repeat"),
            PropertyValue::BackgroundRepeat(BackgroundRepeat::Repeat)
        );
        assert_eq!(
            value_of("background-repeat: space round"),
            PropertyValue::BackgroundRepeat(BackgroundRepeat::Axes(
                super::BackgroundRepeatAxis::Space,
                super::BackgroundRepeatAxis::Round,
            ))
        );
        // Two-value axis form: `repeat no-repeat` → repeat-x.
        assert_eq!(
            value_of("background-repeat: repeat no-repeat"),
            PropertyValue::BackgroundRepeat(BackgroundRepeat::RepeatX)
        );
        // Malformed → dropped.
        assert!(matches!(
            compile("background-repeat: bogus").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }

    #[test]
    fn gradient_background_image_still_lowers_alongside_url_support() {
        // Regression: adding url() support must not disturb gradient lowering.
        let g = gradient_of("background-image: linear-gradient(90deg, red, blue)");
        assert!((g.angle_deg() - 90.0).abs() < 1e-6);
        assert_eq!(g.stops().len(), 2);
    }

    #[test]
    fn background_shorthand_maps_radial_and_conic_to_image() {
        let radial = compile_all("background: radial-gradient(red, blue)");
        let d = standard_of(&radial[0]);
        assert_eq!(d.property(), PropertyId::BackgroundImage);
        assert!(matches!(d.value(), PropertyValue::RadialGradient(_)));

        let conic = compile_all("background: conic-gradient(from 45deg, red, blue)");
        let d = standard_of(&conic[0]);
        assert_eq!(d.property(), PropertyId::BackgroundImage);
        match d.value() {
            PropertyValue::ConicGradient(g) => assert_eq!(g.from_angle_deg(), 45.0),
            other => panic!("expected a conic gradient, got {other:?}"),
        }
    }

    #[test]
    fn background_shorthand_maps_gradient_to_image() {
        let compiled = compile_all("background: linear-gradient(45deg, red, blue)");
        assert_eq!(compiled.len(), 9);
        let d = standard_of(&compiled[0]);
        assert_eq!(d.property(), PropertyId::BackgroundImage);
        match d.value() {
            PropertyValue::LinearGradient(g) => assert_eq!(g.angle_deg(), 45.0),
            other => panic!("expected a linear gradient, got {other:?}"),
        }
    }

    #[test]
    fn background_shorthand_resets_omitted_longhands_and_keeps_the_final_color() {
        let compiled = compile_all("background: none, blue linear-gradient(red, red)");
        assert_eq!(compiled.len(), 9);
        assert_eq!(
            standard_of(&compiled[0])
                .value()
                .as_background_layers()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            standard_of(&compiled[1]).value(),
            &PropertyValue::Color(color(0.0, 0.0, 1.0))
        );
        assert!(
            compiled[2..]
                .iter()
                .all(|decl| standard_of(decl).value() == &PropertyValue::Initial)
        );
        assert_eq!(
            standard_of(&compiled[5]).property(),
            PropertyId::BackgroundBlendMode
        );
    }

    #[test]
    fn background_shorthand_accepts_per_layer_position_size_and_repeat() {
        let compiled =
            compile_all("background: none, right bottom / 12px 25% space no-repeat blue");
        assert_eq!(compiled.len(), 9);
        assert_eq!(
            standard_of(&compiled[2]).value(),
            &PropertyValue::BackgroundSizes(Box::new([
                BackgroundSize::Auto,
                BackgroundSize::Explicit {
                    width: Length::px(12.0),
                    height: Length::Percent(25.0)
                },
            ]))
        );
        assert_eq!(
            standard_of(&compiled[3]).value(),
            &PropertyValue::BackgroundPositions(Box::new([
                BackgroundPosition::default(),
                BackgroundPosition::new(Length::Percent(100.0), Length::Percent(100.0)),
            ]))
        );
        assert_eq!(
            standard_of(&compiled[4]).value(),
            &PropertyValue::BackgroundRepeats(Box::new([
                BackgroundRepeat::Repeat,
                BackgroundRepeat::Axes(
                    super::BackgroundRepeatAxis::Space,
                    super::BackgroundRepeatAxis::NoRepeat
                ),
            ]))
        );
    }

    #[test]
    fn invalid_background_layer_components_reject_the_whole_shorthand() {
        for invalid in [
            "background: blue, none",
            "background: / 10px none",
            "background: left / none 10px",
            "background: left right / 20px",
            "background: no-repeat none repeat",
        ] {
            assert!(
                matches!(
                    compile_all(invalid).as_slice(),
                    [CompiledDeclaration::Unsupported(_)]
                ),
                "{invalid}"
            );
        }
    }

    #[test]
    fn background_shorthand_maps_color_to_background_color() {
        let compiled = compile_all("background: #ff0000");
        let d = standard_of(&compiled[1]);
        assert_eq!(d.property(), PropertyId::BackgroundColor);
        assert_eq!(d.value(), &PropertyValue::Color(color(1.0, 0.0, 0.0)));
    }
}
