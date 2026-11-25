#![doc = include_str!("../../docs/api/compiler/grid_track_lowering.doc/_module.md")]

use super::calc_lowering::lower_math_function;
use super::property_compiler::narrow_f32;
use super::value_lowering::{css_wide_keyword, dimension_length};
use crate::parser::ParsedValue;
use obscur_style::property::{
    GridAreaTemplate, GridTrack, GridTrackList, Length, PropertyValue, layout_keyword,
};

/// The largest number of tracks a single `repeat(N, …)` is allowed to expand to,
/// so a pathological repetition count cannot blow up the track list.
const MAX_REPEAT: usize = 1024;

#[doc = include_str!("../../docs/api/compiler/grid_track_lowering.doc/lower_grid_template.md")]
pub(super) fn lower_grid_template(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [ParsedValue::Keyword(keyword)] = values
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(PropertyValue::Initial);
    }
    let (tracks, line_names) = track_list_from_components(values, true)?;
    if tracks.is_empty() || !valid_auto_track_list(&tracks) {
        return None;
    }
    Some(PropertyValue::grid_track_list(GridTrackList::new(
        tracks.into_boxed_slice(),
        compact_line_names(line_names),
    )))
}

/// Omits the boundary buckets when the track list contains no authored names.
fn compact_line_names(names: Vec<Vec<Box<str>>>) -> Box<[Box<[Box<str>]>]> {
    if names.iter().all(Vec::is_empty) {
        Box::from([])
    } else {
        names.into_iter().map(Vec::into_boxed_slice).collect()
    }
}

/// Lowers the rows, columns and areas of the grid-template shorthand.
pub(super) fn lower_grid_template_shorthand(values: &[ParsedValue]) -> Option<[PropertyValue; 3]> {
    if let [ParsedValue::Keyword(keyword)] = values {
        let value = css_wide_keyword(keyword).or_else(|| {
            keyword
                .eq_ignore_ascii_case("none")
                .then_some(PropertyValue::Initial)
        })?;
        return Some([value.clone(), value.clone(), value]);
    }

    let mut axes = values.split(|value| matches!(value, ParsedValue::Delimiter('/')));
    let rows = axes.next()?;
    let columns = axes.next();
    if axes.next().is_some() {
        return None;
    }
    if !rows
        .iter()
        .any(|value| matches!(value, ParsedValue::String(_)))
    {
        return Some([
            lower_grid_template(rows)?,
            lower_grid_template(columns?)?,
            PropertyValue::Initial,
        ]);
    }

    let mut row_tracks = Vec::new();
    let mut area_rows = Vec::new();
    let mut tokens = rows.iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            ParsedValue::LineNames(_) => row_tracks.push(token.clone()),
            ParsedValue::String(_) => {
                area_rows.push(token.clone());
                let size = match tokens.peek() {
                    None | Some(ParsedValue::LineNames(_) | ParsedValue::String(_)) => {
                        ParsedValue::Keyword("auto".into())
                    }
                    Some(size) => {
                        grid_track_scalar(size)?;
                        tokens.next()?.clone()
                    }
                };
                row_tracks.push(size);
            }
            _ => return None,
        }
    }
    let columns = match columns {
        Some(values) => {
            for value in values {
                if !matches!(value, ParsedValue::LineNames(_)) {
                    grid_track_scalar(value)?;
                }
            }
            lower_grid_template(values)?
        }
        None => PropertyValue::Initial,
    };
    Some([
        lower_grid_template(&row_tracks)?,
        columns,
        lower_grid_template_areas(&area_rows)?,
    ])
}

/// Lowers the explicit and implicit longhands of the grid shorthand.
pub(super) fn lower_grid_shorthand(values: &[ParsedValue]) -> Option<[PropertyValue; 6]> {
    if let [ParsedValue::Keyword(keyword)] = values
        && let Some(value) = css_wide_keyword(keyword)
    {
        return Some(std::array::from_fn(|_| value.clone()));
    }
    if let Some([rows, columns, areas]) = lower_grid_template_shorthand(values) {
        return Some([
            rows,
            columns,
            areas,
            PropertyValue::Initial,
            PropertyValue::Initial,
            PropertyValue::Initial,
        ]);
    }

    let mut axes = values.split(|value| matches!(value, ParsedValue::Delimiter('/')));
    let rows = axes.next()?;
    let columns = axes.next()?;
    if axes.next().is_some() {
        return None;
    }
    let initial = PropertyValue::Initial;
    match (auto_flow_prefix(rows), auto_flow_prefix(columns)) {
        (Some((dense, implicit)), None) => Some([
            initial.clone(),
            lower_grid_template(columns)?,
            initial.clone(),
            lower_implicit_sizes(implicit)?,
            initial,
            flow_value(false, dense),
        ]),
        (None, Some((dense, implicit))) => Some([
            lower_grid_template(rows)?,
            initial.clone(),
            initial.clone(),
            initial,
            lower_implicit_sizes(implicit)?,
            flow_value(true, dense),
        ]),
        _ => None,
    }
}

/// Reads the auto-flow marker, an optional dense modifier, and the remaining tracks.
fn auto_flow_prefix(values: &[ParsedValue]) -> Option<(bool, &[ParsedValue])> {
    let keyword = |token: &ParsedValue, name: &str| matches!(token, ParsedValue::Keyword(value) if value.eq_ignore_ascii_case(name));
    match values {
        [first, second, rest @ ..]
            if keyword(first, "auto-flow") && keyword(second, "dense")
                || keyword(first, "dense") && keyword(second, "auto-flow") =>
        {
            Some((true, rest))
        }
        [first, rest @ ..] if keyword(first, "auto-flow") => Some((false, rest)),
        _ => None,
    }
}

fn lower_implicit_sizes(values: &[ParsedValue]) -> Option<PropertyValue> {
    if values.is_empty() {
        return Some(PropertyValue::Initial);
    }
    let tracks = values
        .iter()
        .map(grid_track_scalar)
        .collect::<Option<Vec<_>>>()?;
    Some(PropertyValue::grid_tracks(tracks))
}

fn flow_value(column: bool, dense: bool) -> PropertyValue {
    let symbol = match (column, dense) {
        (false, false) => layout_keyword::GRID_AUTO_FLOW_ROW,
        (false, true) => layout_keyword::GRID_AUTO_FLOW_ROW_DENSE,
        (true, false) => layout_keyword::GRID_AUTO_FLOW_COLUMN,
        (true, true) => layout_keyword::GRID_AUTO_FLOW_COLUMN_DENSE,
    };
    PropertyValue::Keyword(symbol)
}

/// Checks the fixed-size grammar when the template contains an auto-repeat.
fn valid_auto_track_list(tracks: &[GridTrack]) -> bool {
    let mut repeats = tracks.iter().filter_map(|track| match track {
        GridTrack::AutoRepeat { tracks, .. } => Some(tracks),
        _ => None,
    });
    let Some(pattern) = repeats.next() else {
        return true;
    };
    repeats.next().is_none()
        && pattern.iter().all(is_fixed_track_size)
        && tracks.iter().all(|track| {
            matches!(track, GridTrack::AutoRepeat { .. }) || is_fixed_track_size(track)
        })
}

/// A fixed breadth or a minmax with at least one fixed breadth.
fn is_fixed_track_size(track: &GridTrack) -> bool {
    match track {
        GridTrack::Fixed(_) => true,
        GridTrack::MinMax(min, max) => {
            matches!(min.as_ref(), GridTrack::Fixed(_))
                || matches!(max.as_ref(), GridTrack::Fixed(_))
        }
        _ => false,
    }
}

/// The parsed track list of a `grid-template-*` value: the flat [`GridTrack`]
/// sizing functions plus one name bucket per grid line (`tracks.len() + 1`).
type ParsedTrackList = (Vec<GridTrack>, Vec<Vec<Box<str>>>);

/// Parses the tracks and names of a grid track list. Each name group applies
/// to the current boundary, producing one bucket per grid line. Numeric
/// repeats expand their tracks and merge the names at shared boundaries.
/// Automatic repeats retain names on their pattern boundaries for later
/// expansion. A repeat pattern cannot contain another repeat.
fn track_list_from_components(
    values: &[ParsedValue],
    allow_repeat: bool,
) -> Option<ParsedTrackList> {
    let mut tracks = Vec::new();
    // Start with the bucket for line 0 (before the first track).
    let mut line_names: Vec<Vec<Box<str>>> = vec![Vec::new()];
    for value in values {
        match value {
            // A comma has no meaning inside a track list; reject rather than drop.
            ParsedValue::Comma => return None,
            ParsedValue::LineNames(names) => {
                if let Some(bucket) = line_names.last_mut() {
                    bucket.extend(names.iter().cloned());
                }
            }
            ParsedValue::Function { name, args }
                if allow_repeat && name.eq_ignore_ascii_case("repeat") =>
            {
                let (repeated_tracks, repeated_names) = expand_repeat(args)?;
                let mut names = repeated_names.into_iter();
                line_names.last_mut()?.extend(names.next()?);
                line_names.extend(names);
                tracks.extend(repeated_tracks);
            }
            other => {
                tracks.push(grid_track_scalar(other)?);
                line_names.push(Vec::new());
            }
        }
    }
    Some((tracks, line_names))
}

#[doc = include_str!("../../docs/api/compiler/grid_track_lowering.doc/lower_grid_auto_tracks.md")]
pub(super) fn lower_grid_auto_tracks(values: &[ParsedValue]) -> Option<PropertyValue> {
    let (tracks, _) = track_list_from_components(values, true)?;
    if tracks.is_empty() {
        return None;
    }
    // `auto-fill` / `auto-fit` is only valid on `grid-template-*`; reject it here.
    if tracks
        .iter()
        .any(|t| matches!(t, GridTrack::AutoRepeat { .. }))
    {
        return None;
    }
    Some(PropertyValue::grid_tracks(tracks))
}

#[doc = include_str!("../../docs/api/compiler/grid_track_lowering.doc/lower_grid_template_areas.md")]
pub(super) fn lower_grid_template_areas(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [ParsedValue::Keyword(keyword)] = values
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(PropertyValue::Initial);
    }
    let mut rows: Vec<Vec<Option<Box<str>>>> = Vec::new();
    for value in values {
        let ParsedValue::String(row) = value else {
            return None;
        };
        let cells: Vec<Option<Box<str>>> = row
            .split_whitespace()
            .map(|cell| {
                // A run of one-or-more `.` is a null cell; any other token is a name.
                if cell.bytes().all(|b| b == b'.') {
                    None
                } else {
                    Some(cell.into())
                }
            })
            .collect();
        rows.push(cells);
    }
    GridAreaTemplate::from_rows(&rows).map(PropertyValue::GridAreas)
}

#[doc = include_str!("../../docs/api/compiler/grid_track_lowering.doc/grid_track_scalar.md")]
fn grid_track_scalar(value: &ParsedValue) -> Option<GridTrack> {
    match value {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto") => {
            Some(GridTrack::Auto)
        }

        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("min-content") => {
            Some(GridTrack::MinContent)
        }
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("max-content") => {
            Some(GridTrack::MaxContent)
        }
        ParsedValue::Dimension { value, unit } if unit.eq_ignore_ascii_case("fr") => {
            Some(GridTrack::Fr(narrow_f32(*value)))
        }
        ParsedValue::Dimension { value, unit } => {
            dimension_length(*value, unit).map(GridTrack::Fixed)
        }
        ParsedValue::Percentage(p) => Some(GridTrack::Fixed(Length::Percent(narrow_f32(*p)))),
        ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("fit-content") => {
            let limit = match single_component(args)? {
                ParsedValue::Number(value) if *value == 0.0 => Length::px(0.0),
                ParsedValue::Dimension { value, unit } if *value >= 0.0 => {
                    dimension_length(*value, unit)?
                }
                ParsedValue::Percentage(value) if *value >= 0.0 => {
                    Length::Percent(narrow_f32(*value))
                }
                ParsedValue::Function { name, args } => {
                    return lower_math_function(name, args).map(GridTrack::FitContentMath);
                }
                _ => return None,
            };
            Some(GridTrack::FitContent(limit))
        }
        ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("minmax") => {
            lower_minmax(args)
        }
        _ => None,
    }
}

/// Lowers `minmax(min, max)` into a [`GridTrack::MinMax`]; each bound is a single
/// non-`repeat` track. Returns [`None`] unless exactly two comma-separated,
/// single-component bounds are present.
fn lower_minmax(args: &[ParsedValue]) -> Option<GridTrack> {
    let groups: Vec<&[ParsedValue]> = args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    let [min_group, max_group] = groups.as_slice() else {
        return None;
    };
    let min = grid_track_scalar(single_component(min_group)?)?;
    let max = grid_track_scalar(single_component(max_group)?)?;
    if matches!(
        min,
        GridTrack::Fr(_)
            | GridTrack::MinMax(..)
            | GridTrack::FitContent(_)
            | GridTrack::FitContentMath(_)
    ) || matches!(
        max,
        GridTrack::MinMax(..) | GridTrack::FitContent(_) | GridTrack::FitContentMath(_)
    ) {
        return None;
    }
    Some(GridTrack::minmax(min, max))
}

/// Expands numerical repeats into tracks and line-name buckets. Names at shared
/// boundaries are merged. Automatic repeats retain their track sizing pattern.
fn expand_repeat(args: &[ParsedValue]) -> Option<ParsedTrackList> {
    let comma = args.iter().position(|v| matches!(v, ParsedValue::Comma))?;
    let (count_part, rest) = args.split_at(comma);
    let (inner, names) = track_list_from_components(&rest[1..], false)?;
    if inner.is_empty() {
        return None;
    }
    // Automatic repeats retain a single sizing pattern for layout expansion.
    if let Some(fill) = auto_repeat_fill(count_part) {
        return Some((
            vec![GridTrack::AutoRepeat {
                fill,
                tracks: inner.into_boxed_slice(),
                line_names: compact_line_names(names),
            }],
            vec![Vec::new(), Vec::new()],
        ));
    }

    let count = repeat_count(count_part)?;
    let mut tracks = Vec::with_capacity(inner.len().saturating_mul(count));
    let mut line_names = vec![Vec::new()];
    let (first, rest) = names.split_first()?;
    for _ in 0..count {
        // The last line of one copy is also the first line of the next.
        line_names.last_mut()?.extend(first.iter().cloned());
        line_names.extend(rest.iter().cloned());
        tracks.extend(inner.iter().cloned());
    }
    Some((tracks, line_names))
}

/// Reads the `auto-fill` / `auto-fit` keyword of a `repeat(…)` count: `Some(true)`
/// for `auto-fill`, `Some(false)` for `auto-fit`, and [`None`] for any other count
/// (a numeric repetition or an invalid keyword).
fn auto_repeat_fill(part: &[ParsedValue]) -> Option<bool> {
    match single_component(part)? {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto-fill") => Some(true),
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto-fit") => Some(false),
        _ => None,
    }
}

/// Resolves the numeric count component of an integer `repeat(N, …)`: a positive
/// integer clamped to [`MAX_REPEAT`]. [`None`] for any non-numeric count (the
/// `auto-fill` / `auto-fit` keywords are handled by [`auto_repeat_fill`]).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn repeat_count(part: &[ParsedValue]) -> Option<usize> {
    match single_component(part)? {
        ParsedValue::Number(n) if *n >= 1.0 => Some((*n as usize).min(MAX_REPEAT)),
        _ => None,
    }
}

/// Returns the sole component of a group, or [`None`] when it has zero or more
/// than one component.
fn single_component(group: &[ParsedValue]) -> Option<&ParsedValue> {
    match group {
        [one] => Some(one),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::property_compiler::tests::{compile, standard, value_of};

    use crate::compiler::CompiledDeclaration;

    use obscur_style::property::{GridTrack, GridTrackList, Length, PropertyId, PropertyValue};

    /// Extracts the lowered [`GridTrack`] list for a `grid-template-*` value.
    fn tracks(text: &str) -> Vec<GridTrack> {
        match value_of(text) {
            PropertyValue::GridTracks(list) => list.into_tracks().into_vec(),
            other => panic!("expected grid tracks, got {other:?}"),
        }
    }

    /// The full track list (tracks + line names) for a `grid-template-*` value.
    fn track_list(text: &str) -> GridTrackList {
        match value_of(text) {
            PropertyValue::GridTracks(list) => list,
            other => panic!("expected grid tracks, got {other:?}"),
        }
    }

    #[test]
    fn lowers_fixed_and_fr_track_list() {
        assert_eq!(
            tracks("grid-template-columns: 100px 1fr 2fr"),
            vec![
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
                GridTrack::Fr(2.0),
            ]
        );
        // Carries its own property id, and rows lower identically.
        assert_eq!(
            standard("grid-template-columns: 1fr").0.property(),
            PropertyId::GridTemplateColumns
        );
        assert_eq!(
            standard("grid-template-rows: 1fr").0.property(),
            PropertyId::GridTemplateRows
        );
    }

    #[test]
    fn lowers_auto_and_percent_and_em_tracks() {
        assert_eq!(tracks("grid-template-columns: auto"), vec![GridTrack::Auto]);
        assert_eq!(
            tracks("grid-template-columns: 50% 2em"),
            vec![
                GridTrack::Fixed(Length::Percent(50.0)),
                GridTrack::Fixed(Length::Em(2.0)),
            ]
        );
    }

    #[test]
    fn lowers_the_content_keyword_tracks() {
        assert_eq!(
            tracks("grid-template-columns: min-content"),
            vec![GridTrack::MinContent]
        );
        assert_eq!(
            tracks("grid-template-columns: MAX-CONTENT"),
            vec![GridTrack::MaxContent]
        );
        assert_eq!(
            tracks("grid-auto-columns: min-content"),
            vec![GridTrack::MinContent]
        );
    }

    #[test]
    fn one_content_keyword_track_no_longer_drops_the_whole_list() {
        // The defect this test exists for: `min-content` was not a track the
        // grammar knew, and an unknown track aborts the ENTIRE declaration — so
        // one keyword took the `1fr` and the `100px` down with it and the element
        // laid out with no template at all.
        assert_eq!(
            tracks("grid-template-columns: min-content 1fr 100px"),
            vec![
                GridTrack::MinContent,
                GridTrack::Fr(1.0),
                GridTrack::Fixed(Length::px(100.0)),
            ]
        );
    }

    #[test]
    fn lowers_content_keywords_inside_minmax_and_repeat() {
        assert_eq!(
            tracks("grid-template-columns: minmax(min-content, 1fr)"),
            vec![GridTrack::minmax(GridTrack::MinContent, GridTrack::Fr(1.0))]
        );
        assert_eq!(
            tracks("grid-template-columns: repeat(2, max-content)"),
            vec![GridTrack::MaxContent, GridTrack::MaxContent]
        );
    }

    #[test]
    fn lowers_fit_content_track_caps_and_checks_the_track_grammar() {
        assert_eq!(
            tracks("grid-template-columns: fit-content(2em) repeat(2, fit-content(25%))"),
            [
                GridTrack::FitContent(Length::Em(2.0)),
                GridTrack::FitContent(Length::Percent(25.0)),
                GridTrack::FitContent(Length::Percent(25.0)),
            ],
        );
        for value in [
            "fit-content(-10px)",
            "fit-content(1fr)",
            "fit-content(auto)",
            "minmax(auto, fit-content(10px))",
            "repeat(auto-fill, fit-content(10px))",
        ] {
            assert!(
                matches!(
                    compile(&format!("grid-template-columns: {value}")).0,
                    CompiledDeclaration::Unsupported(_)
                ),
                "{value}",
            );
        }
    }

    #[test]
    fn a_bare_fit_content_track_is_still_unsupported() {
        // CSS Grid has no bare `fit-content` track — it is
        // `fit-content(<length-percentage>)`.
        assert!(matches!(
            compile("grid-template-columns: fit-content").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }

    #[test]
    fn lowers_repeat_to_flat_list() {
        assert_eq!(
            tracks("grid-template-columns: repeat(2, 1fr)"),
            vec![GridTrack::Fr(1.0), GridTrack::Fr(1.0)]
        );
        // A multi-track repeat body expands each copy in order.
        assert_eq!(
            tracks("grid-template-columns: repeat(2, 100px 1fr)"),
            vec![
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
            ]
        );
        // repeat mixes with surrounding tracks.
        assert_eq!(
            tracks("grid-template-columns: 20px repeat(2, 1fr)"),
            vec![
                GridTrack::Fixed(Length::px(20.0)),
                GridTrack::Fr(1.0),
                GridTrack::Fr(1.0),
            ]
        );
    }

    #[test]
    fn lowers_line_names_interleaved_with_tracks() {
        // `[a] 100px [b] 1fr [c]` -> tracks [Fixed(100), Fr(1)] with one name bucket
        // per grid line: [[a], [b], [c]].
        let list = track_list("grid-template-columns: [a] 100px [b] 1fr [c]");
        assert_eq!(
            list.tracks(),
            &[GridTrack::Fixed(Length::px(100.0)), GridTrack::Fr(1.0)]
        );
        let names: Vec<Vec<&str>> = list
            .line_names()
            .iter()
            .map(|b| b.iter().map(|n| &**n).collect())
            .collect();
        assert_eq!(names, vec![vec!["a"], vec!["b"], vec!["c"]]);
    }

    #[test]
    fn numeric_repeat_merges_names_at_each_shared_line() {
        let list = track_list(
            "grid-template-columns: [outer] repeat(2, [a] 40px [middle] 60px [b]) [tail]",
        );
        let names: Vec<Vec<&str>> = list
            .line_names()
            .iter()
            .map(|bucket| bucket.iter().map(Box::as_ref).collect())
            .collect();
        assert_eq!(
            names,
            [
                vec!["outer", "a"],
                vec!["middle"],
                vec!["b", "a"],
                vec!["middle"],
                vec!["b", "tail"],
            ]
        );
    }

    #[test]
    fn line_can_carry_multiple_names() {
        // `[sidebar-end main-start]` names one line twice.
        let list =
            track_list("grid-template-columns: [s-start] 200px [s-end main-start] 1fr [main-end]");
        let names: Vec<Vec<&str>> = list
            .line_names()
            .iter()
            .map(|b| b.iter().map(|n| &**n).collect())
            .collect();
        assert_eq!(
            names,
            vec![
                vec!["s-start"],
                vec!["s-end", "main-start"],
                vec!["main-end"]
            ]
        );
    }

    #[test]
    fn track_list_without_names_has_empty_line_names() {
        // A no-`[name]` template stays byte-identical: no line-name buckets.
        let list = track_list("grid-template-columns: 100px 1fr");
        assert!(list.line_names().is_empty());
    }

    #[test]
    fn lowers_minmax_track() {
        assert_eq!(
            tracks("grid-template-columns: minmax(100px, 1fr)"),
            vec![GridTrack::minmax(
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0)
            )]
        );
    }

    #[test]
    fn auto_fill_repeat_lowers_to_auto_repeat() {
        // `auto-fill` cannot be resolved without a container width, so its pattern
        // is carried as a single `AutoRepeat` (not expanded to one repetition) and
        // the layout algorithm computes the count from the container size.
        assert_eq!(
            tracks("grid-template-columns: repeat(auto-fill, 100px)"),
            vec![GridTrack::AutoRepeat {
                line_names: Box::from([]),
                fill: true,
                tracks: Box::from([GridTrack::Fixed(Length::px(100.0))]),
            }]
        );
    }

    #[test]
    fn automatic_repeat_preserves_pattern_boundaries_for_layout() {
        let list = track_list(
            "grid-template-columns: [outer] repeat(auto-fill, [start] 40px [end]) [tail]",
        );
        let [GridTrack::AutoRepeat { line_names, .. }] = list.tracks() else {
            panic!("expected one automatic repeat");
        };
        assert_eq!(line_names[0].as_ref(), [Box::<str>::from("start")]);
        assert_eq!(line_names[1].as_ref(), [Box::<str>::from("end")]);
        assert_eq!(list.line_names()[0].as_ref(), [Box::<str>::from("outer")]);
        assert_eq!(list.line_names()[1].as_ref(), [Box::<str>::from("tail")]);
    }

    #[test]
    fn auto_fit_repeat_lowers_to_auto_repeat() {
        assert_eq!(
            tracks("grid-template-columns: repeat(auto-fit, 100px minmax(20px, 1fr))"),
            vec![GridTrack::AutoRepeat {
                line_names: Box::from([]),
                fill: false,
                tracks: Box::from([
                    GridTrack::Fixed(Length::px(100.0)),
                    GridTrack::minmax(GridTrack::Fixed(Length::px(20.0)), GridTrack::Fr(1.0)),
                ]),
            }]
        );
    }

    #[test]
    fn grid_shorthand_rejects_invalid_auto_flow_forms() {
        for value in [
            "auto-flow 20px / auto-flow 30px",
            "auto-flow dense dense / 50px",
            "auto-flow /",
            "none / auto-flow repeat(2, 20px)",
        ] {
            let (result, _) = compile(&format!("grid: {value}"));
            assert!(
                matches!(result, CompiledDeclaration::Unsupported(_)),
                "{value}"
            );
        }
    }

    #[test]
    fn invalid_auto_repeat_track_lists_are_rejected() {
        for value in [
            "repeat(auto-fit, 100px 1fr)",
            "repeat(auto-fill, auto)",
            "repeat(auto-fill, minmax(1fr, 100px))",
            "repeat(auto-fill, minmax(20px, minmax(30px, 40px)))",
            "repeat(auto-fill, 100px) 1fr",
            "repeat(auto-fill, 100px) repeat(auto-fit, 100px)",
            "repeat(2, repeat(auto-fill, 100px))",
        ] {
            let (result, _) = compile(&format!("grid-template-columns: {value}"));
            assert!(
                matches!(result, CompiledDeclaration::Unsupported(_)),
                "{value}"
            );
        }
    }

    #[test]
    fn auto_fill_repeat_keeps_surrounding_tracks() {
        // A leading fixed track and a trailing track flank the auto-repeat.
        assert_eq!(
            tracks("grid-template-columns: 50px repeat(auto-fill, 100px) 20px"),
            vec![
                GridTrack::Fixed(Length::px(50.0)),
                GridTrack::AutoRepeat {
                    line_names: Box::from([]),
                    fill: true,
                    tracks: Box::from([GridTrack::Fixed(Length::px(100.0))]),
                },
                GridTrack::Fixed(Length::px(20.0)),
            ]
        );
    }

    #[test]
    fn grid_template_honours_css_wide_keywords() {
        assert_eq!(
            value_of("grid-template-columns: inherit"),
            PropertyValue::Inherit
        );
        assert_eq!(
            value_of("grid-template-rows: initial"),
            PropertyValue::Initial
        );
    }

    #[test]
    fn invalid_grid_template_is_unsupported() {
        // A bare colour is not a valid track.
        let (result, _) = compile("grid-template-columns: red");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn lowers_grid_auto_rows_track_list() {
        // `grid-auto-rows: 40px` -> a single fixed 40px implicit-track size.
        assert_eq!(
            tracks("grid-auto-rows: 40px"),
            vec![GridTrack::Fixed(Length::px(40.0))]
        );
        assert_eq!(
            standard("grid-auto-rows: 40px").0.property(),
            PropertyId::GridAutoRows
        );
        // Cycled multi-track and minmax reuse the same track grammar.
        assert_eq!(
            tracks("grid-auto-columns: 30px 50px"),
            vec![
                GridTrack::Fixed(Length::px(30.0)),
                GridTrack::Fixed(Length::px(50.0))
            ]
        );
        assert_eq!(
            tracks("grid-auto-columns: minmax(50px, 1fr)"),
            vec![GridTrack::minmax(
                GridTrack::Fixed(Length::px(50.0)),
                GridTrack::Fr(1.0)
            )]
        );
    }

    #[test]
    fn grid_auto_tracks_honour_css_wide_keywords() {
        assert_eq!(value_of("grid-auto-rows: inherit"), PropertyValue::Inherit);
        assert_eq!(
            value_of("grid-auto-columns: initial"),
            PropertyValue::Initial
        );
    }

    #[test]
    fn grid_auto_tracks_reject_auto_repeat() {
        // `auto-fill` / `auto-fit` is invalid for implicit tracks (no container
        // basis for a repetition count); the declaration is unsupported.
        let (result, _) = compile("grid-auto-columns: repeat(auto-fill, 100px)");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn grid_template_areas_parses_into_a_template() {
        // `"h h" "s m" "f f"`: three rows of two columns.
        let PropertyValue::GridAreas(template) =
            value_of("grid-template-areas: \"h h\" \"s m\" \"f f\"")
        else {
            panic!("expected a grid-areas value");
        };
        assert_eq!(template.rows(), 3);
        assert_eq!(template.columns(), 2);
        // `h` spans row 1, columns 1-2 → line rect (1..2, 1..3).
        let h = template.area("h").expect("h");
        assert_eq!(
            (h.row_start, h.row_end, h.column_start, h.column_end),
            (1, 2, 1, 3)
        );
        // `s` is the single cell at row 2, column 1.
        let s = template.area("s").expect("s");
        assert_eq!(
            (s.row_start, s.row_end, s.column_start, s.column_end),
            (2, 3, 1, 2)
        );
    }

    #[test]
    fn grid_template_areas_null_cell_is_empty() {
        let PropertyValue::GridAreas(template) = value_of("grid-template-areas: \"a .\" \". b\"")
        else {
            panic!("expected a grid-areas value");
        };
        assert!(template.area("a").is_some());
        assert!(template.area("b").is_some());
        assert!(template.area(".").is_none());
    }

    #[test]
    fn grid_template_areas_non_rectangular_is_unsupported() {
        // `a` forms an L-shape (a hole in its bounding box) → not a filled rectangle.
        let (result, _) = compile("grid-template-areas: \"a a\" \"a .\"");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
        // A ragged grid (unequal columns) is likewise rejected.
        let (result, _) = compile("grid-template-areas: \"a b\" \"a\"");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn grid_template_areas_honours_css_wide_keywords() {
        assert_eq!(
            value_of("grid-template-areas: inherit"),
            PropertyValue::Inherit
        );
        assert_eq!(
            value_of("grid-template-areas: initial"),
            PropertyValue::Initial
        );
    }

    // ---- the `flex` shorthand (CSS Flexbox 1 §7.1) ----
}
