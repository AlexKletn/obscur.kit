use obscur_style::property::{Length, PropertyValue};

use crate::parser::ParsedValue;

use super::property_compiler::narrow_f32;
use super::value_lowering::css_wide_keyword;

fn clip_reference_box(keyword: &str) -> Option<obscur_style::property::BackgroundBox> {
    use obscur_style::property::BackgroundBox;

    // Clip paths on kit nodes use CSS layout boxes.
    if keyword.eq_ignore_ascii_case("fill-box") {
        Some(BackgroundBox::Content)
    } else if keyword.eq_ignore_ascii_case("stroke-box") || keyword.eq_ignore_ascii_case("view-box")
    {
        Some(BackgroundBox::Border)
    } else {
        super::background_lowering::background_box(keyword)
    }
}

pub(super) fn lower_clip_path(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [ParsedValue::Keyword(keyword)] = values {
        return if keyword.eq_ignore_ascii_case("none") {
            Some(PropertyValue::Initial)
        } else {
            clip_reference_box(keyword)
                .map(PropertyValue::ClipBox)
                .or_else(|| css_wide_keyword(keyword))
        };
    }
    let (function, reference_box) = match values {
        [function @ ParsedValue::Function { .. }] => {
            (function, obscur_style::property::BackgroundBox::Border)
        }
        [
            function @ ParsedValue::Function { .. },
            ParsedValue::Keyword(keyword),
        ]
        | [
            ParsedValue::Keyword(keyword),
            function @ ParsedValue::Function { .. },
        ] => (function, clip_reference_box(keyword)?),
        _ => return None,
    };
    let ParsedValue::Function { name, args } = function else {
        return None;
    };
    if name.eq_ignore_ascii_case("path") {
        let (data, rule) = match args.as_slice() {
            [ParsedValue::String(data)] => (data, obscur_scene::path::FillRule::NonZero),
            [
                ParsedValue::Keyword(rule),
                ParsedValue::Comma,
                ParsedValue::String(data),
            ] => {
                let rule = if rule.eq_ignore_ascii_case("evenodd") {
                    obscur_scene::path::FillRule::EvenOdd
                } else if rule.eq_ignore_ascii_case("nonzero") {
                    obscur_scene::path::FillRule::NonZero
                } else {
                    return None;
                };
                (data, rule)
            }
            _ => return None,
        };
        let path = super::svg_path_lowering::lower_svg_path(data, rule)?;
        let outline = obscur_scene::path::PathTrim::of(&path);
        return Some(PropertyValue::ClipSvgPath {
            reference_box,
            path: std::sync::Arc::new(path),
            outline: std::sync::Arc::new(outline),
        });
    }
    use obscur_style::property::ClipRectangleKind;
    let rectangle = if name.eq_ignore_ascii_case("inset") {
        Some(ClipRectangleKind::Inset)
    } else if name.eq_ignore_ascii_case("rect") {
        Some(ClipRectangleKind::Edges)
    } else if name.eq_ignore_ascii_case("xywh") {
        Some(ClipRectangleKind::OriginAndSize)
    } else {
        None
    };
    if let Some(kind) = rectangle {
        return lower_clip_rectangle(args, reference_box, kind);
    }
    if name.eq_ignore_ascii_case("circle") || name.eq_ignore_ascii_case("ellipse") {
        return lower_clip_ellipse(args, reference_box, name.eq_ignore_ascii_case("circle"));
    }
    if !name.eq_ignore_ascii_case("polygon") {
        return None;
    }
    let mut vertices = args.as_slice();
    let mut even_odd = false;
    if let [ParsedValue::Keyword(rule), ParsedValue::Comma, rest @ ..] = vertices {
        if rule.eq_ignore_ascii_case("evenodd") {
            even_odd = true;
        } else if !rule.eq_ignore_ascii_case("nonzero") {
            return None;
        }
        vertices = rest;
    }
    let points = vertices
        .split(|value| matches!(value, ParsedValue::Comma))
        .map(|pair| {
            let [x, y] = pair else {
                return None;
            };
            Some([clip_coordinate(x)?, clip_coordinate(y)?])
        })
        .collect::<Option<Box<[_]>>>()?;
    (points.len() >= 3).then_some(PropertyValue::ClipPolygon {
        even_odd,
        reference_box,
        points,
    })
}

fn clip_coordinate(value: &ParsedValue) -> Option<obscur_style::property::LengthExpr> {
    use obscur_style::property::LengthExpr;

    let length = match value {
        ParsedValue::Dimension { value, unit } => {
            super::value_lowering::dimension_length(*value, unit)?
        }
        ParsedValue::Percentage(value) => Length::Percent(narrow_f32(*value)),
        ParsedValue::Number(value) if *value == 0.0 => Length::px(0.0),
        ParsedValue::Function { name, args } => {
            return super::calc_lowering::lower_math_function(name, args);
        }
        _ => return None,
    };
    Some(LengthExpr::leaf(length))
}

fn lower_clip_rectangle(
    args: &[ParsedValue],
    reference_box: obscur_style::property::BackgroundBox,
    kind: obscur_style::property::ClipRectangleKind,
) -> Option<PropertyValue> {
    use obscur_style::property::{ClipRectangleKind, LengthExpr};
    let mut pieces = args.split(|value| {
        matches!(value, ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("round"))
    });
    let values = pieces.next()?;
    let coordinates = match (kind, values) {
        (ClipRectangleKind::Inset, values) => clip_sides(values, clip_coordinate)?,
        (ClipRectangleKind::Edges, [top, right, bottom, left]) => {
            let edge = |value: &ParsedValue| match value {
                ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto") => {
                    Some(LengthExpr::leaf(Length::Auto))
                }
                _ => clip_coordinate(value),
            };
            [edge(top)?, edge(right)?, edge(bottom)?, edge(left)?]
        }
        (ClipRectangleKind::OriginAndSize, [x, y, width, height]) => [
            clip_coordinate(x)?,
            clip_coordinate(y)?,
            clip_radius_length(width)?,
            clip_radius_length(height)?,
        ],
        _ => return None,
    };
    let radii = if let Some(round) = pieces.next() {
        let mut axes = round.split(|value| matches!(value, ParsedValue::Delimiter('/')));
        let horizontal = clip_sides(axes.next()?, clip_radius_length)?;
        let vertical = match axes.next() {
            Some(values) => clip_sides(values, clip_radius_length)?,
            None => horizontal.clone(),
        };
        if axes.next().is_some() {
            return None;
        }
        Some(Box::new(std::array::from_fn(|index| {
            [horizontal[index].clone(), vertical[index].clone()]
        })))
    } else {
        None
    };
    if pieces.next().is_some() {
        return None;
    }
    Some(PropertyValue::ClipRectangle {
        reference_box,
        kind,
        coordinates: Box::new(coordinates),
        radii,
    })
}

fn clip_sides(
    values: &[ParsedValue],
    lower: impl Fn(&ParsedValue) -> Option<obscur_style::property::LengthExpr>,
) -> Option<[obscur_style::property::LengthExpr; 4]> {
    let [first, rest @ ..] = values else {
        return None;
    };
    let a = lower(first)?;
    Some(match rest {
        [] => [a.clone(), a.clone(), a.clone(), a],
        [b] => {
            let b = lower(b)?;
            [a.clone(), b.clone(), a, b]
        }
        [b, c] => {
            let b = lower(b)?;
            [a, b.clone(), lower(c)?, b]
        }
        [b, c, d] => [a, lower(b)?, lower(c)?, lower(d)?],
        _ => return None,
    })
}

fn clip_radius_length(value: &ParsedValue) -> Option<obscur_style::property::LengthExpr> {
    match value {
        ParsedValue::Function { .. } => clip_coordinate(value),
        ParsedValue::Dimension { value: number, .. }
        | ParsedValue::Percentage(number)
        | ParsedValue::Number(number)
            if *number >= 0.0 =>
        {
            clip_coordinate(value)
        }
        _ => None,
    }
}

fn lower_clip_ellipse(
    args: &[ParsedValue],
    reference_box: obscur_style::property::BackgroundBox,
    circle: bool,
) -> Option<PropertyValue> {
    use obscur_style::property::{ClipRadius, LengthExpr};

    let (radius_args, center) = if let Some(at) = args.iter().position(|value| {
        matches!(value, ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("at"))
    }) {
        let ([x, y], [right, bottom]) = super::background_lowering::lower_position(
            &args[at + 1..],
            clip_coordinate,
            LengthExpr::leaf,
        )?;
        let coordinate = |offset, from_end| {
            if from_end {
                LengthExpr::Diff(
                    Box::new(LengthExpr::leaf(Length::Percent(100.0))),
                    Box::new(offset),
                )
            } else {
                offset
            }
        };
        (&args[..at], [coordinate(x, right), coordinate(y, bottom)])
    } else {
        (args, [const { LengthExpr::leaf(Length::Percent(50.0)) }; 2])
    };
    let radius = |value: &ParsedValue| match value {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("closest-side") => {
            Some(ClipRadius::ClosestSide)
        }
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("farthest-side") => {
            Some(ClipRadius::FarthestSide)
        }
        _ => clip_radius_length(value).map(ClipRadius::Length),
    };
    let radii = match radius_args {
        [] => [const { ClipRadius::ClosestSide }; 2],
        [r] if circle => {
            let radius = radius(r)?;
            [radius.clone(), radius]
        }
        [x, y] if !circle => [radius(x)?, radius(y)?],
        _ => return None,
    };
    Some(PropertyValue::ClipEllipse {
        reference_box,
        center,
        radii,
        circle,
    })
}
