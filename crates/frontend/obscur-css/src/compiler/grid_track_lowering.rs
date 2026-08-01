#![doc = include_str!("grid_track_lowering.doc/_module.md")]

use super::property_compiler::narrow_f32;
use super::value_lowering::dimension_length;
use crate::parser::ParsedValue;
use obscur_style::property::{GridAreaTemplate, GridTrack, GridTrackList, Length, PropertyValue};

#[doc = include_str!("grid_track_lowering.doc/MAX_REPEAT.md")]
const MAX_REPEAT: usize = 1024;

#[doc = include_str!("grid_track_lowering.doc/lower_grid_template.md")]
pub(super) fn lower_grid_template(values: &[ParsedValue]) -> Option<PropertyValue> {
    let (tracks, line_names) = track_list_from_components(values)?;
    if tracks.is_empty() {
        return None;
    }
    // A template with no authored `[name]` group carries an empty `line_names`,
    // keeping the value byte-identical to the pre-named-lines list; otherwise the
    // buckets are one per grid line (`tracks.len() + 1`).
    let line_names = if line_names.iter().all(Vec::is_empty) {
        Box::from([])
    } else {
        line_names
            .into_iter()
            .map(Vec::into_boxed_slice)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    };
    Some(PropertyValue::grid_track_list(GridTrackList::new(
        tracks.into_boxed_slice(),
        line_names,
    )))
}

#[doc = include_str!("grid_track_lowering.doc/ParsedTrackList.md")]
type ParsedTrackList = (Vec<GridTrack>, Vec<Vec<Box<str>>>);

#[doc = include_str!("grid_track_lowering.doc/track_list_from_components.md")]
fn track_list_from_components(values: &[ParsedValue]) -> Option<ParsedTrackList> {
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
            ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("repeat") => {
                for track in expand_repeat(args)? {
                    tracks.push(track);
                    line_names.push(Vec::new());
                }
            }
            other => {
                tracks.push(grid_track_scalar(other)?);
                line_names.push(Vec::new());
            }
        }
    }
    Some((tracks, line_names))
}

#[doc = include_str!("grid_track_lowering.doc/lower_grid_auto_tracks.md")]
pub(super) fn lower_grid_auto_tracks(values: &[ParsedValue]) -> Option<PropertyValue> {
    let tracks = tracks_from_components(values)?;
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

#[doc = include_str!("grid_track_lowering.doc/lower_grid_template_areas.md")]
pub(super) fn lower_grid_template_areas(values: &[ParsedValue]) -> Option<PropertyValue> {
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

#[doc = include_str!("grid_track_lowering.doc/tracks_from_components.md")]
fn tracks_from_components(values: &[ParsedValue]) -> Option<Vec<GridTrack>> {
    let mut out = Vec::new();
    for value in values {
        match value {
            // A comma has no meaning inside a track list; reject rather than drop.
            ParsedValue::Comma => return None,
            // `[name]` line-name groups carry no sizing; they are captured by
            // `track_list_from_components` on the `grid-template-*` path and simply
            // skipped here (the `grid-auto-*` / inner-`repeat` paths keep no names).
            ParsedValue::LineNames(_) => {}
            ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("repeat") => {
                out.extend(expand_repeat(args)?);
            }
            other => out.push(grid_track_scalar(other)?),
        }
    }
    Some(out)
}

#[doc = include_str!("grid_track_lowering.doc/grid_track_scalar.md")]
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
        ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("minmax") => {
            lower_minmax(args)
        }
        _ => None,
    }
}

#[doc = include_str!("grid_track_lowering.doc/lower_minmax.md")]
fn lower_minmax(args: &[ParsedValue]) -> Option<GridTrack> {
    let groups: Vec<&[ParsedValue]> = args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    let [min_group, max_group] = groups.as_slice() else {
        return None;
    };
    let min = grid_track_scalar(single_component(min_group)?)?;
    let max = grid_track_scalar(single_component(max_group)?)?;
    Some(GridTrack::minmax(min, max))
}

#[doc = include_str!("grid_track_lowering.doc/expand_repeat.md")]
fn expand_repeat(args: &[ParsedValue]) -> Option<Vec<GridTrack>> {
    let comma = args.iter().position(|v| matches!(v, ParsedValue::Comma))?;
    let (count_part, rest) = args.split_at(comma);
    let inner = tracks_from_components(&rest[1..])?;
    if inner.is_empty() {
        return None;
    }
    // `auto-fill` / `auto-fit`: keep the pattern as an auto-repeat resolved later.
    if let Some(fill) = auto_repeat_fill(count_part) {
        return Some(vec![GridTrack::AutoRepeat {
            fill,
            tracks: inner.into_boxed_slice(),
        }]);
    }
    let count = repeat_count(count_part)?;
    let mut out = Vec::with_capacity(inner.len().saturating_mul(count));
    for _ in 0..count {
        out.extend(inner.iter().cloned());
    }
    Some(out)
}

#[doc = include_str!("grid_track_lowering.doc/auto_repeat_fill.md")]
fn auto_repeat_fill(part: &[ParsedValue]) -> Option<bool> {
    match single_component(part)? {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto-fill") => Some(true),
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto-fit") => Some(false),
        _ => None,
    }
}

#[doc = include_str!("grid_track_lowering.doc/repeat_count.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn repeat_count(part: &[ParsedValue]) -> Option<usize> {
    match single_component(part)? {
        ParsedValue::Number(n) if *n >= 1.0 => Some((*n as usize).min(MAX_REPEAT)),
        _ => None,
    }
}

#[doc = include_str!("grid_track_lowering.doc/single_component.md")]
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
    fn a_bare_fit_content_track_is_still_unsupported() {
        // CSS Grid has no bare `fit-content` track — it is
        // `fit-content(<length-percentage>)`, whose argument this crate does not
        // model — so it stays an unsupported-feature diagnostic rather than being
        // quietly accepted as something else.
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
                fill: true,
                tracks: Box::from([GridTrack::Fixed(Length::px(100.0))]),
            }]
        );
    }

    #[test]
    fn auto_fit_repeat_lowers_to_auto_repeat() {
        assert_eq!(
            tracks("grid-template-columns: repeat(auto-fit, 100px 1fr)"),
            vec![GridTrack::AutoRepeat {
                fill: false,
                tracks: Box::from([GridTrack::Fixed(Length::px(100.0)), GridTrack::Fr(1.0)]),
            }]
        );
    }

    #[test]
    fn auto_fill_repeat_keeps_surrounding_tracks() {
        // A leading fixed track and a trailing track flank the auto-repeat.
        assert_eq!(
            tracks("grid-template-columns: 50px repeat(auto-fill, 100px) 20px"),
            vec![
                GridTrack::Fixed(Length::px(50.0)),
                GridTrack::AutoRepeat {
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

    // ---- grid-auto-columns / grid-auto-rows ----

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

    // ---- grid-column / grid-row line placement ----

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
