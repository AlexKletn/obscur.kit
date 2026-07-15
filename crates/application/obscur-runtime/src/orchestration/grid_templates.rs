//! Bridges an authored CSS grid template into an `obscur-layout` [`GridTemplate`].
//!
//! `grid-template-columns` / `grid-template-rows` are computed on a node's
//! [`ComputedStyle`] as a style-local [`PropertyValue::GridTracks`] list (each a
//! [`GridTrack`]). Layout consumes its own vocabulary, so — exactly as
//! [`layout_style_from`](super::layout_coordinator) maps computed lengths into
//! [`Dimension`](obscur_layout::model::Dimension)s — this module converts each
//! [`GridTrack`] into an `obscur_layout` [`TrackSize`] and assembles the
//! [`GridTemplate`] a grid container lays out with (spec §4.1).
//!
//! A [`GridTrack::Fixed`] length is resolved to logical pixels through the same
//! [`LengthContext`] the box-model lengths use. A bare percentage inside a track
//! has no known basis at this point (the grid container's content size is not yet
//! known here), so it is carried symbolically as a [`TrackSize::Percent`]
//! fraction and resolved against the container content extent *inside* the layout
//! algorithm. A `repeat(auto-fill | auto-fit, …)` track — whose repetition count
//! also depends on the (not-yet-known) container size — is likewise carried
//! symbolically: it becomes a layout [`AutoRepeat`] inside the axis
//! [`TrackList`], expanded to concrete tracks by the grid algorithm. Line
//! placement / spanning remain out of scope for this step.

use std::collections::HashMap;

use obscur_geometry::Size;
use obscur_identity::NodeId;
use obscur_layout::algorithm::{AutoRepeat, AutoRepeatMode, GridTemplate, TrackList, TrackSize};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{GridTrack, Length, LengthContext, PropertyId, PropertyValue};

use obscur_text::shaping::ParagraphShaper;

use super::layout_coordinator::{ch_width_for, length_context};
use super::ComputedStyles;

/// The [`GridTemplate`] derived for every grid node that authored one, keyed by
/// [`NodeId`].
///
/// This is the grid twin of [`LayoutStyles`](super::LayoutStyles): a per-node
/// side-table the coordinator threads into the layout walk so a
/// [`LayoutDisplay::Grid`](obscur_layout::model::LayoutDisplay::Grid) node with an
/// authored template dispatches the real track-sizing algorithm. A node absent
/// from this map simply has no authored template and falls back to the default
/// single implicit column.
pub type GridTemplates = HashMap<NodeId, GridTemplate>;

/// Builds a [`GridTemplates`] side-table from every node's [`ComputedStyle`].
///
/// Only nodes that authored `grid-template-columns` and/or
/// `grid-template-rows` produce an entry; a node with neither is skipped, so the
/// map stays small. `viewport`, `root_font_size` and `paragraph` supply the
/// resolution context for relative / viewport units in a fixed track (`em`,
/// `rem`, `vw`, `ch`, …), exactly as in
/// [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from) — so
/// `grid-template-columns: repeat(3, 20ch)` sizes its tracks from the `0` glyph.
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

/// Converts a single node's computed grid template into an `obscur_layout`
/// [`GridTemplate`], or [`None`] when the node authored neither track list.
///
/// The column and row track lists are read from [`PropertyId::GridTemplateColumns`]
/// / [`PropertyId::GridTemplateRows`] (each a [`PropertyValue::GridTracks`]) and
/// each [`GridTrack`] is mapped to a [`TrackSize`] via [`track_size_of`]. An empty
/// `rows` list yields an all-implicit-row (auto) grid, the common columns-only
/// template.
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
    let auto_map =
        |tracks: Option<&[GridTrack]>| auto_track_sizes(tracks.unwrap_or(&[]), &ctx);
    Some(
        GridTemplate::from_track_lists(map(columns), map(rows))
            .with_auto_tracks(auto_map(auto_columns), auto_map(auto_rows)),
    )
}

/// Converts a flat `grid-auto-columns` / `grid-auto-rows` [`GridTrack`] list into
/// the implicit-track [`TrackSize`] vector, reusing [`track_size_of`].
///
/// These lists carry no auto-repeat (the CSS layer rejects `auto-fill` / `auto-fit`
/// here), so every element maps 1:1; a stray auto-repeat would degrade to a
/// content-sized [`TrackSize::Auto`] via [`track_size_of`] rather than panic.
fn auto_track_sizes(tracks: &[GridTrack], ctx: &LengthContext) -> Vec<TrackSize> {
    tracks.iter().map(|t| track_size_of(t, ctx)).collect()
}

/// Converts one axis's style [`GridTrack`] list into a layout [`TrackList`],
/// splitting it around its (first) `repeat(auto-fill | auto-fit, …)`.
///
/// Tracks before the auto-repeat become the leading run, the auto-repeat's pattern
/// becomes the [`AutoRepeat`], and tracks after it become the trailing run — the
/// shape [`TrackList`] and the grid algorithm expand against the container size.
/// A list with no auto-repeat yields a leading-only `TrackList` that expands
/// byte-identically to its explicit tracks. Only the first auto-repeat is honoured
/// (as in CSS, which allows one per axis); any later `AutoRepeat` degrades through
/// [`track_size_of`] to a single [`TrackSize::Auto`] track.
fn track_list_of(tracks: &[GridTrack], ctx: &LengthContext) -> TrackList {
    let mut leading = Vec::new();
    let mut auto_repeat = None;
    let mut trailing = Vec::new();
    for track in tracks {
        match track {
            GridTrack::AutoRepeat { fill, tracks: pattern } if auto_repeat.is_none() => {
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

/// Reads a `grid-template-*` property as its authored [`GridTrack`] slice, or
/// [`None`] when the property is absent or not a track list.
fn grid_tracks_of(computed: &ComputedStyle, property: PropertyId) -> Option<&[GridTrack]> {
    computed
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_tracks)
}

/// Maps one style [`GridTrack`] into an `obscur_layout` [`TrackSize`].
///
/// A [`GridTrack::Fixed`] length is resolved to logical pixels against `ctx`; a
/// bare percentage — its basis is the not-yet-known container content size —
/// becomes a [`TrackSize::Percent`] fraction the algorithm resolves later, and
/// `auto` (no numeric value) degrades to [`TrackSize::Auto`]. `fr`, `auto`, and
/// `minmax` map structurally, recursing on the `minmax` bounds.
fn track_size_of(track: &GridTrack, ctx: &LengthContext) -> TrackSize {
    match track {
        GridTrack::Fixed(length) => fixed_track_size(*length, ctx),
        GridTrack::Fr(fraction) => TrackSize::Fr(*fraction),
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

/// Resolves a fixed-track [`Length`] to a concrete [`TrackSize`].
///
/// A `px` (or resolvable relative unit) becomes a [`TrackSize::Fixed`]; a bare
/// percentage becomes a [`TrackSize::Percent`] fraction (the `0..100` percent is
/// converted to `0..1`) carried symbolically to the layout algorithm, where the
/// container content extent — its basis — is known; `auto` (no numeric value)
/// degrades to [`TrackSize::Auto`].
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
        computed.set(PropertyId::FontSize, ComputedValue::length(Length::px(16.0)));
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
            &[TrackSize::minmax(TrackSize::Percent(0.30), TrackSize::Fr(1.0))]
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
            &[TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0))]
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
        let repeat = template.columns().auto_repeat().expect("auto-repeat present");
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
        let mut computed = ComputedStyles::new();
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
