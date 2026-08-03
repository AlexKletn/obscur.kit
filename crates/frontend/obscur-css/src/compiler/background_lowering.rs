#![doc = include_str!("background_lowering.doc/_module.md")]

use super::color_lowering::parse_stop_color;
use super::gradient_lowering::{
    lower_conic_gradient, lower_linear_gradient, lower_radial_gradient,
    lower_repeating_linear_gradient,
};
use super::property_compiler::{CompiledDeclaration, narrow_f32, value_unsupported_kind};
use super::stylesheet_compiler::CssSymbolInterner;
use super::value_lowering::dimension_length;
use crate::error::UnsupportedCssFeature;
use crate::parser::{ParsedDeclaration, ParsedValue};
use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{
    BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize, Length, ObjectFit,
    PropertyId, PropertyValue,
};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

#[doc = include_str!("background_lowering.doc/expand_background.md")]
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
        vec![CompiledDeclaration::Standard(Declaration::new(
            property,
            value,
            important,
            origin,
            specificity,
            source_order,
            selector,
        ))]
    };
    if let Some(value) = lower_background_image(&decl.value) {
        return standard(PropertyId::BackgroundImage, value);
    }
    if let [one] = decl.value.as_slice()
        && let Some(color) = parse_stop_color(one)
    {
        return standard(PropertyId::BackgroundColor, PropertyValue::Color(color));
    }
    vec![CompiledDeclaration::Unsupported(
        UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
    )]
}

#[doc = include_str!("background_lowering.doc/lower_background_size.md")]
pub(super) fn lower_background_size(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [ParsedValue::Keyword(keyword)] = values {
        if keyword.eq_ignore_ascii_case("cover") {
            return Some(PropertyValue::BackgroundSize(BackgroundSize::Cover));
        }
        if keyword.eq_ignore_ascii_case("contain") {
            return Some(PropertyValue::BackgroundSize(BackgroundSize::Contain));
        }
        if keyword.eq_ignore_ascii_case("auto") {
            return Some(PropertyValue::BackgroundSize(BackgroundSize::Auto));
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
        return Some(PropertyValue::BackgroundSize(BackgroundSize::Auto));
    }
    Some(PropertyValue::BackgroundSize(BackgroundSize::Explicit {
        width,
        height,
    }))
}

#[doc = include_str!("background_lowering.doc/background_size_component.md")]
fn background_size_component(value: &ParsedValue) -> Option<Length> {
    match value {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto") => Some(Length::Auto),
        ParsedValue::Percentage(p) => Some(Length::Percent(narrow_f32(*p))),
        ParsedValue::Dimension { value, unit } => dimension_length(*value, unit),
        ParsedValue::Number(n) if *n == 0.0 => Some(Length::px(0.0)),
        _ => None,
    }
}

#[doc = include_str!("background_lowering.doc/lower_object_fit.md")]
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

#[doc = include_str!("background_lowering.doc/lower_background_position.md")]
pub(super) fn lower_background_position(values: &[ParsedValue]) -> Option<PropertyValue> {
    let components: Vec<PositionComponent> = values
        .iter()
        .map(position_component)
        .collect::<Option<Vec<_>>>()?;
    let (x, y) = match components.as_slice() {
        [only] => match only {
            PositionComponent::Horizontal(p) => (Length::Percent(*p), Length::Percent(50.0)),
            PositionComponent::Vertical(p) => (Length::Percent(50.0), Length::Percent(*p)),
            PositionComponent::Center => (Length::Percent(50.0), Length::Percent(50.0)),
            PositionComponent::Length(l) => (*l, Length::Percent(50.0)),
        },
        [_, _] => {
            let mut x: Option<Length> = None;
            let mut y: Option<Length> = None;
            // First pass: explicit-axis keywords claim their axis (order-free).
            for comp in &components {
                match comp {
                    PositionComponent::Horizontal(p) => x = Some(Length::Percent(*p)),
                    PositionComponent::Vertical(p) => y = Some(Length::Percent(*p)),
                    PositionComponent::Center | PositionComponent::Length(_) => {}
                }
            }
            // Second pass: `center` / lengths fill the remaining axes in order.
            for comp in &components {
                let value = match comp {
                    PositionComponent::Center => Length::Percent(50.0),
                    PositionComponent::Length(l) => *l,
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
                x.unwrap_or(Length::Percent(50.0)),
                y.unwrap_or(Length::Percent(50.0)),
            )
        }
        _ => return None,
    };
    Some(PropertyValue::BackgroundPosition(BackgroundPosition::new(
        x, y,
    )))
}

#[doc = include_str!("background_lowering.doc/PositionComponent.md")]
enum PositionComponent {
    #[doc = include_str!("background_lowering.doc/Horizontal.md")]
    Horizontal(f32),
    #[doc = include_str!("background_lowering.doc/Vertical.md")]
    Vertical(f32),
    #[doc = include_str!("background_lowering.doc/Center.md")]
    Center,
    #[doc = include_str!("background_lowering.doc/Length.md")]
    Length(Length),
}

#[doc = include_str!("background_lowering.doc/position_component.md")]
fn position_component(value: &ParsedValue) -> Option<PositionComponent> {
    match value {
        ParsedValue::Keyword(keyword) => {
            if keyword.eq_ignore_ascii_case("left") {
                Some(PositionComponent::Horizontal(0.0))
            } else if keyword.eq_ignore_ascii_case("right") {
                Some(PositionComponent::Horizontal(100.0))
            } else if keyword.eq_ignore_ascii_case("top") {
                Some(PositionComponent::Vertical(0.0))
            } else if keyword.eq_ignore_ascii_case("bottom") {
                Some(PositionComponent::Vertical(100.0))
            } else if keyword.eq_ignore_ascii_case("center") {
                Some(PositionComponent::Center)
            } else {
                None
            }
        }
        ParsedValue::Percentage(p) => {
            Some(PositionComponent::Length(Length::Percent(narrow_f32(*p))))
        }
        ParsedValue::Dimension { value, unit } => {
            dimension_length(*value, unit).map(PositionComponent::Length)
        }
        ParsedValue::Number(n) if *n == 0.0 => Some(PositionComponent::Length(Length::px(0.0))),
        _ => None,
    }
}

#[doc = include_str!("background_lowering.doc/lower_background_repeat.md")]
pub(super) fn lower_background_repeat(values: &[ParsedValue]) -> Option<PropertyValue> {
    match values {
        [ParsedValue::Keyword(keyword)] => {
            let repeat = if keyword.eq_ignore_ascii_case("repeat-x") {
                BackgroundRepeat::RepeatX
            } else if keyword.eq_ignore_ascii_case("repeat-y") {
                BackgroundRepeat::RepeatY
            } else if keyword.eq_ignore_ascii_case("no-repeat") {
                BackgroundRepeat::NoRepeat
            } else if keyword.eq_ignore_ascii_case("repeat")
                // `space` / `round` fold to `repeat` (documented approximation).
                || keyword.eq_ignore_ascii_case("space")
                || keyword.eq_ignore_ascii_case("round")
            {
                BackgroundRepeat::Repeat
            } else {
                return None;
            };
            Some(PropertyValue::BackgroundRepeat(repeat))
        }
        [ParsedValue::Keyword(x), ParsedValue::Keyword(y)] => {
            let rx = repeat_axis_tiles(x)?;
            let ry = repeat_axis_tiles(y)?;
            let repeat = match (rx, ry) {
                (true, true) => BackgroundRepeat::Repeat,
                (false, false) => BackgroundRepeat::NoRepeat,
                (true, false) => BackgroundRepeat::RepeatX,
                (false, true) => BackgroundRepeat::RepeatY,
            };
            Some(PropertyValue::BackgroundRepeat(repeat))
        }
        _ => None,
    }
}

#[doc = include_str!("background_lowering.doc/repeat_axis_tiles.md")]
fn repeat_axis_tiles(keyword: &str) -> Option<bool> {
    if keyword.eq_ignore_ascii_case("repeat")
        || keyword.eq_ignore_ascii_case("space")
        || keyword.eq_ignore_ascii_case("round")
    {
        Some(true)
    } else if keyword.eq_ignore_ascii_case("no-repeat") {
        Some(false)
    } else {
        None
    }
}

#[doc = include_str!("background_lowering.doc/lower_background_image.md")]
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

#[doc = include_str!("background_lowering.doc/lower_background_layer.md")]
fn lower_background_layer(values: &[ParsedValue]) -> Option<BackgroundLayer> {
    match values {
        [ParsedValue::Function { name, args }] if name.eq_ignore_ascii_case("linear-gradient") => {
            lower_linear_gradient(args).map(BackgroundLayer::Linear)
        }
        [ParsedValue::Function { name, args }]
            if name.eq_ignore_ascii_case("repeating-linear-gradient") =>
        {
            lower_repeating_linear_gradient(args).map(BackgroundLayer::Linear)
        }
        [ParsedValue::Function { name, args }] if name.eq_ignore_ascii_case("radial-gradient") => {
            lower_radial_gradient(args).map(BackgroundLayer::Radial)
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

#[doc = include_str!("background_lowering.doc/lower_url.md")]
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

#[doc = include_str!("background_lowering.doc/append_url_component.md")]
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
        // `space` / `round` fold to `repeat` (documented approximation).
        assert_eq!(
            value_of("background-repeat: space"),
            PropertyValue::BackgroundRepeat(BackgroundRepeat::Repeat)
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
        assert_eq!(compiled.len(), 1);
        let d = standard_of(&compiled[0]);
        assert_eq!(d.property(), PropertyId::BackgroundImage);
        match d.value() {
            PropertyValue::LinearGradient(g) => assert_eq!(g.angle_deg(), 45.0),
            other => panic!("expected a linear gradient, got {other:?}"),
        }
    }

    #[test]
    fn background_shorthand_maps_color_to_background_color() {
        let compiled = compile_all("background: #ff0000");
        let d = standard_of(&compiled[0]);
        assert_eq!(d.property(), PropertyId::BackgroundColor);
        assert_eq!(d.value(), &PropertyValue::Color(color(1.0, 0.0, 0.0)));
    }
}
