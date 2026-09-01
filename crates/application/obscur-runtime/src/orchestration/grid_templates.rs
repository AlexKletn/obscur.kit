#![doc = include_str!("grid_templates.doc/_module.md")]

use obscur_geometry::Size;
use obscur_identity::{FastMap, NodeId};
use obscur_layout::algorithm::{AutoRepeat, AutoRepeatMode, GridTemplate, TrackList, TrackSize};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{GridTrack, Length, LengthContext, PropertyId, PropertyValue};

use obscur_text::shaping::ParagraphShaper;

use super::ComputedStyles;
use super::length_resolution::{ch_width_for, length_context};

#[doc = include_str!("grid_templates.doc/GridTemplates.md")]
pub type GridTemplates = FastMap<NodeId, GridTemplate>;

#[doc = include_str!("grid_templates.doc/grid_templates_from.md")]
#[must_use]
pub fn grid_templates_from(
    computed: &ComputedStyles,
    viewport: Size,
    root_font_size: f32,
    paragraph: Option<&ParagraphShaper>,
) -> GridTemplates {
    computed
        .iter()
        .filter_map(|(&node, style)| {
            grid_template_for(style, viewport, root_font_size, paragraph)
                .map(|template| (node, template))
        })
        .collect()
}

#[doc = include_str!("grid_templates.doc/grid_template_for.md")]
#[must_use]
pub fn grid_template_for(
    computed: &ComputedStyle,
    viewport: Size,
    root_font_size: f32,
    paragraph: Option<&ParagraphShaper>,
) -> Option<GridTemplate> {
    let columns = grid_tracks_of(computed, PropertyId::GridTemplateColumns);
    let rows = grid_tracks_of(computed, PropertyId::GridTemplateRows);
    let auto_columns = grid_tracks_of(computed, PropertyId::GridAutoColumns);
    let auto_rows = grid_tracks_of(computed, PropertyId::GridAutoRows);
    if columns.is_none() && rows.is_none() && auto_columns.is_none() && auto_rows.is_none() {
        return None;
    }
    let ctx = length_context(
        computed,
        viewport,
        root_font_size,
        ch_width_for(computed, paragraph),
    );
    let map = |tracks: Option<&[GridTrack]>| track_list_of(tracks.unwrap_or(&[]), &ctx);
    // `grid-auto-columns` / `grid-auto-rows` are flat implicit-track sizing lists
    // (auto-fill / auto-fit is rejected in the CSS layer), so each `GridTrack` maps
    // straight to a `TrackSize` via the same conversion the template tracks use. An
    // absent list stays empty → the algorithm's default `Auto` implicit tracks.
    let auto_map = |tracks: Option<&[GridTrack]>| auto_track_sizes(tracks.unwrap_or(&[]), &ctx);
    Some(
        GridTemplate::from_track_lists(map(columns), map(rows))
            .with_auto_tracks(auto_map(auto_columns), auto_map(auto_rows)),
    )
}

#[doc = include_str!("grid_templates.doc/auto_track_sizes.md")]
fn auto_track_sizes(tracks: &[GridTrack], ctx: &LengthContext) -> Vec<TrackSize> {
    tracks.iter().map(|t| track_size_of(t, ctx)).collect()
}

#[doc = include_str!("grid_templates.doc/track_list_of.md")]
fn track_list_of(tracks: &[GridTrack], ctx: &LengthContext) -> TrackList {
    let mut leading = Vec::new();
    let mut auto_repeat = None;
    let mut trailing = Vec::new();
    for track in tracks {
        match track {
            GridTrack::AutoRepeat {
                fill,
                tracks: pattern,
            } if auto_repeat.is_none() => {
                let mode = if *fill {
                    AutoRepeatMode::AutoFill
                } else {
                    AutoRepeatMode::AutoFit
                };
                let pattern = pattern.iter().map(|t| track_size_of(t, ctx)).collect();
                auto_repeat = Some(AutoRepeat::new(mode, pattern));
            }
            other => {
                let size = track_size_of(other, ctx);
                if auto_repeat.is_none() {
                    leading.push(size);
                } else {
                    trailing.push(size);
                }
            }
        }
    }
    TrackList::new(leading, auto_repeat, trailing)
}

#[doc = include_str!("grid_templates.doc/grid_tracks_of.md")]
fn grid_tracks_of(computed: &ComputedStyle, property: PropertyId) -> Option<&[GridTrack]> {
    computed
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_tracks)
}

#[doc = include_str!("grid_templates.doc/track_size_of.md")]
fn track_size_of(track: &GridTrack, ctx: &LengthContext) -> TrackSize {
    match track {
        GridTrack::Fixed(length) => fixed_track_size(*length, ctx),
        GridTrack::Fr(fraction) => TrackSize::Fr(*fraction),
        // The content keywords carry no length either, and unlike a percentage
        // they have no basis to wait for: the algorithm answers them by measuring
        // the items placed in the track.
        GridTrack::MinContent => TrackSize::MinContent,
        GridTrack::MaxContent => TrackSize::MaxContent,
        GridTrack::MinMax(min, max) => {
            TrackSize::minmax(track_size_of(min, ctx), track_size_of(max, ctx))
        }
        // `Auto` is content-sized. A top-level auto-repeat is peeled off by
        // `track_list_of`; only a nested or duplicate one reaches here, and it has
        // no scalar size, so it degrades to the same content-sized `Auto` track (a
        // documented fallback).
        GridTrack::Auto | GridTrack::AutoRepeat { .. } => TrackSize::Auto,
    }
}

#[doc = include_str!("grid_templates.doc/fixed_track_size.md")]
fn fixed_track_size(length: Length, ctx: &LengthContext) -> TrackSize {
    match length {
        Length::Px(px) => TrackSize::Fixed(px.0),
        // A percentage's basis (the container content size) is unknown here, so
        // carry the fraction to the algorithm instead of degrading it.
        Length::Percent(pct) => TrackSize::Percent(pct / 100.0),
        // `auto` has no numeric value; it is a content-sized track.
        Length::Auto => TrackSize::Auto,
        relative => relative
            .resolve_px(ctx)
            .map_or(TrackSize::Auto, TrackSize::Fixed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Size = Size::new(800.0, 600.0);
    const ROOT_FONT: f32 = 16.0;

    #[test]
    fn absent_template_yields_none() {
        let computed = ComputedStyle::new();
        assert!(grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).is_none());
    }

    #[test]
    fn converts_fixed_fr_auto_tracks_to_pixels() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
                GridTrack::Auto,
            ]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        assert_eq!(
            template.columns().leading(),
            &[TrackSize::Fixed(100.0), TrackSize::Fr(1.0), TrackSize::Auto]
        );
        assert!(template.columns().auto_repeat().is_none());
        // No rows authored -> all-implicit (empty) row list.
        assert!(template.rows().is_empty());
    }

    #[test]
    fn resolves_relative_fixed_track_against_context() {
        let mut computed = ComputedStyle::new();
        // font-size 16 -> 2em = 32px; rows carry a rem track resolved against the
        // document root font size (20).
        computed.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(16.0)),
        );
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::Fixed(
                Length::Em(2.0),
            )]))
            .expect("concrete"),
        );
        computed.set(
            PropertyId::GridTemplateRows,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::Fixed(
                Length::Rem(2.0),
            )]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, 20.0, None).expect("template");
        assert_eq!(template.columns().leading(), &[TrackSize::Fixed(32.0)]);
        assert_eq!(template.rows().leading(), &[TrackSize::Fixed(40.0)]);
    }

    #[test]
    fn percent_track_maps_to_percent_fraction() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([
                GridTrack::Fixed(Length::Percent(30.0)),
                GridTrack::Fixed(Length::Percent(50.0)),
            ]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        // A bare percentage no longer degrades to `Auto`: it carries its `0..1`
        // fraction to the algorithm, which resolves it against the container size.
        assert_eq!(
            template.columns().leading(),
            &[TrackSize::Percent(0.30), TrackSize::Percent(0.50)]
        );
    }

    #[test]
    fn percent_inside_minmax_maps_to_percent_fraction() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::minmax(
                GridTrack::Fixed(Length::Percent(30.0)),
                GridTrack::Fr(1.0),
            )]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        assert_eq!(
            template.columns().leading(),
            &[TrackSize::minmax(
                TrackSize::Percent(0.30),
                TrackSize::Fr(1.0)
            )]
        );
    }

    #[test]
    fn content_keyword_tracks_carry_to_the_layout_track_list() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([
                GridTrack::MinContent,
                GridTrack::Fr(1.0),
                GridTrack::minmax(GridTrack::MinContent, GridTrack::MaxContent),
            ]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        // They do NOT degrade to `Auto`: the algorithm answers each by measuring
        // the items with the offer that keyword names.
        assert_eq!(
            template.columns().leading(),
            &[
                TrackSize::MinContent,
                TrackSize::Fr(1.0),
                TrackSize::minmax(TrackSize::MinContent, TrackSize::MaxContent),
            ]
        );
    }

    #[test]
    fn converts_minmax_recursively() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::minmax(
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
            )]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        assert_eq!(
            template.columns().leading(),
            &[TrackSize::minmax(
                TrackSize::Fixed(100.0),
                TrackSize::Fr(1.0)
            )]
        );
    }

    #[test]
    fn auto_repeat_track_maps_to_layout_auto_repeat() {
        let mut computed = ComputedStyle::new();
        // `50px repeat(auto-fill, 100px) 20px`: leading 50px, an auto-fill repeat,
        // and a trailing 20px track.
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([
                GridTrack::Fixed(Length::px(50.0)),
                GridTrack::AutoRepeat {
                    fill: true,
                    tracks: Box::from([GridTrack::Fixed(Length::px(100.0))]),
                },
                GridTrack::Fixed(Length::px(20.0)),
            ]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        let columns = template.columns();
        assert_eq!(columns.leading(), &[TrackSize::Fixed(50.0)]);
        assert_eq!(columns.trailing(), &[TrackSize::Fixed(20.0)]);
        let repeat = columns.auto_repeat().expect("auto-repeat present");
        assert_eq!(repeat.mode(), AutoRepeatMode::AutoFill);
        assert_eq!(repeat.tracks(), &[TrackSize::Fixed(100.0)]);
    }

    #[test]
    fn auto_fit_track_maps_to_auto_fit_mode() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::AutoRepeat {
                fill: false,
                tracks: Box::from([GridTrack::Fixed(Length::px(100.0))]),
            }]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        let repeat = template
            .columns()
            .auto_repeat()
            .expect("auto-repeat present");
        assert_eq!(repeat.mode(), AutoRepeatMode::AutoFit);
    }

    #[test]
    fn grid_auto_tracks_map_to_implicit_track_sizes() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::Fixed(
                Length::px(100.0),
            )]))
            .expect("concrete"),
        );
        computed.set(
            PropertyId::GridAutoRows,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::Fixed(
                Length::px(40.0),
            )]))
            .expect("concrete"),
        );
        computed.set(
            PropertyId::GridAutoColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([
                GridTrack::Fixed(Length::px(30.0)),
                GridTrack::Fr(1.0),
            ]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        assert_eq!(template.auto_rows(), &[TrackSize::Fixed(40.0)]);
        assert_eq!(
            template.auto_columns(),
            &[TrackSize::Fixed(30.0), TrackSize::Fr(1.0)]
        );
    }

    #[test]
    fn grid_auto_rows_alone_yields_template() {
        // A node with only `grid-auto-rows` (no explicit template) still produces a
        // template so its implicit rows are sized (not the default `None`).
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridAutoRows,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::Fixed(
                Length::px(40.0),
            )]))
            .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        assert!(template.columns().is_empty());
        assert_eq!(template.auto_rows(), &[TrackSize::Fixed(40.0)]);
    }

    #[test]
    fn absent_auto_tracks_stay_empty() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::Fr(1.0)]))
                .expect("concrete"),
        );
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT, None).expect("template");
        assert!(template.auto_columns().is_empty());
        assert!(template.auto_rows().is_empty());
    }

    #[test]
    fn grid_templates_from_only_maps_authored_nodes() {
        let mut computed = ComputedStyles::default();
        let mut with_template = ComputedStyle::new();
        with_template.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([GridTrack::Fr(1.0)]))
                .expect("concrete"),
        );
        computed.insert(NodeId::from_raw(1), with_template);
        computed.insert(NodeId::from_raw(2), ComputedStyle::new());

        let templates = grid_templates_from(&computed, VIEWPORT, ROOT_FONT, None);
        assert_eq!(templates.len(), 1);
        assert!(templates.contains_key(&NodeId::from_raw(1)));
    }
}
