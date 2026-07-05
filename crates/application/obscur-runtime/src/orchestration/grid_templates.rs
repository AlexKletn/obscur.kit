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
//! algorithm. `auto-fill` / `auto-fit` (handled in the CSS layer) and line
//! placement / spanning remain out of scope for this step.

use std::collections::HashMap;

use obscur_geometry::Size;
use obscur_identity::NodeId;
use obscur_layout::algorithm::{GridTemplate, TrackSize};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{GridTrack, Length, LengthContext, PropertyId, PropertyValue};

use super::layout_coordinator::length_context;
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
/// map stays small. `viewport` and `root_font_size` supply the resolution context
/// for relative / viewport units in a fixed track (`em`, `rem`, `vw`, …), exactly
/// as in [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from).
#[must_use]
pub fn grid_templates_from(
    computed: &ComputedStyles,
    viewport: Size,
    root_font_size: f32,
) -> GridTemplates {
    computed
        .iter()
        .filter_map(|(&node, style)| {
            grid_template_for(style, viewport, root_font_size).map(|template| (node, template))
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
) -> Option<GridTemplate> {
    let columns = grid_tracks_of(computed, PropertyId::GridTemplateColumns);
    let rows = grid_tracks_of(computed, PropertyId::GridTemplateRows);
    if columns.is_none() && rows.is_none() {
        return None;
    }
    let ctx = length_context(computed, viewport, root_font_size);
    let map = |tracks: Option<&[GridTrack]>| {
        tracks
            .unwrap_or(&[])
            .iter()
            .map(|track| track_size_of(track, &ctx))
            .collect::<Vec<_>>()
    };
    Some(GridTemplate::new(map(columns), map(rows)))
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
        GridTrack::Auto => TrackSize::Auto,
        GridTrack::MinMax(min, max) => {
            TrackSize::minmax(track_size_of(min, ctx), track_size_of(max, ctx))
        }
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
        assert!(grid_template_for(&computed, VIEWPORT, ROOT_FONT).is_none());
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
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT).expect("template");
        assert_eq!(
            template.columns(),
            &[TrackSize::Fixed(100.0), TrackSize::Fr(1.0), TrackSize::Auto]
        );
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
        let template = grid_template_for(&computed, VIEWPORT, 20.0).expect("template");
        assert_eq!(template.columns(), &[TrackSize::Fixed(32.0)]);
        assert_eq!(template.rows(), &[TrackSize::Fixed(40.0)]);
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
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT).expect("template");
        // A bare percentage no longer degrades to `Auto`: it carries its `0..1`
        // fraction to the algorithm, which resolves it against the container size.
        assert_eq!(
            template.columns(),
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
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT).expect("template");
        assert_eq!(
            template.columns(),
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
        let template = grid_template_for(&computed, VIEWPORT, ROOT_FONT).expect("template");
        assert_eq!(
            template.columns(),
            &[TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0))]
        );
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

        let templates = grid_templates_from(&computed, VIEWPORT, ROOT_FONT);
        assert_eq!(templates.len(), 1);
        assert!(templates.contains_key(&NodeId::from_raw(1)));
    }
}
