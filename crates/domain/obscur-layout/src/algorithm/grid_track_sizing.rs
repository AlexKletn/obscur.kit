#![doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/_module.md")]

use std::ops::RangeInclusive;

use crate::model::Dimension;

use super::grid::TrackSize;
pub(super) use super::intrinsic_track_kind::{KINDS, Kind};
use super::max_track_fn::MaxFn;
use super::min_track_fn::MinFn;
pub(super) use super::track_contributions::Contributions;
use super::track_sizing_state::Track;

/// Small epsilon so space distribution stops at ~0 instead of looping on f32
/// rounding error (spec's distribution loops have no natural fixpoint at 0).
const DISTRIBUTE_THRESHOLD: f32 = 1e-6;

/// Whether a track's *max* sizing function is `auto` (§11.8's stretch target).
/// A `max-content` max is content-sized but is **not** stretched.
fn max_is_auto(ts: &TrackSize) -> bool {
    match ts {
        TrackSize::Auto => true,
        // `minmax(_, auto)`, and an invalid nested `minmax` max (treated as `auto`
        // by [`max_fn_of`]), both stretch.
        TrackSize::MinMax(_, max) => {
            matches!(max.as_ref(), TrackSize::Auto | TrackSize::MinMax(..))
        }
        _ => false,
    }
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/min_fn_of.md")]
// `auto` / `min-content` / `fr` deliberately share the `Kind::Min` body but are
// kept as separate, self-documenting arms.
#[allow(clippy::match_same_arms)]
fn min_fn_of(track: &TrackSize, axis: f32) -> MinFn {
    match track {
        TrackSize::Fixed(px) => MinFn::Fixed(px.max(0.0)),
        TrackSize::Percent(_) if !axis.is_finite() => MinFn::Intrinsic(Kind::Min),
        TrackSize::Percent(f) => MinFn::Fixed((f * axis).max(0.0)),
        // `auto` as a MINIMUM is the item's automatic minimum size — accommodated
        // by the min-content contribution (§12.5 "intrinsic minimums"), NOT a
        // fill-available measure. Measuring it as min-content is what stops a block
        // item (whose `width: auto` would otherwise fill the offered width) from
        // inflating its track to the container's whole inline size.
        TrackSize::Auto | TrackSize::FitContent(_) | TrackSize::FitContentMath(_) => {
            MinFn::Intrinsic(Kind::Min)
        }
        TrackSize::MinContent => MinFn::Intrinsic(Kind::Min),
        TrackSize::MaxContent => MinFn::Intrinsic(Kind::Max),
        TrackSize::Fr(_) => MinFn::Intrinsic(Kind::Min),
        TrackSize::MinMax(min, _) => match min.as_ref() {
            // An `fr` is not a valid `minmax` minimum — floor at 0.
            TrackSize::Fr(_) => MinFn::Fixed(0.0),
            other => min_fn_of(other, axis),
        },
    }
}

/// The `max` sizing function of a track (§11.4). A nested `minmax` as the `max`
/// bound is not meaningful; it is treated as `auto` (content-sized).
// `auto` / `max-content` deliberately share the `Kind::Max` body but are kept as
// separate, self-documenting arms.
#[allow(clippy::match_same_arms)]
fn max_fn_of(track: &TrackSize, axis: f32) -> MaxFn {
    match track {
        TrackSize::Fixed(px) => MaxFn::Fixed(px.max(0.0)),
        TrackSize::Percent(_) if !axis.is_finite() => MaxFn::Intrinsic(Kind::Max),
        TrackSize::Percent(f) => MaxFn::Fixed((f * axis).max(0.0)),
        // `auto` as a MAXIMUM is the item's max-content contribution (§12.5
        // "For max-content maximums" applies to `auto`/`max-content` maxes).
        TrackSize::Auto => MaxFn::Intrinsic(Kind::Max),
        TrackSize::MinContent => MaxFn::Intrinsic(Kind::Min),
        TrackSize::MaxContent | TrackSize::FitContent(_) | TrackSize::FitContentMath(_) => {
            MaxFn::Intrinsic(Kind::Max)
        }
        TrackSize::Fr(f) => MaxFn::Flex(f.max(0.0)),
        TrackSize::MinMax(_, max) => match max.as_ref() {
            TrackSize::MinMax(_, _) => MaxFn::Intrinsic(Kind::Auto),
            other => max_fn_of(other, axis),
        },
    }
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/track_takes.md")]
pub(super) fn track_takes(track: &TrackSize, kind: Kind, available: f32) -> bool {
    let bound = |t: &TrackSize| match t {
        // `auto` is `minmax(min-content, max-content)`: its min is fed by the
        // min-content contribution and its max by the max-content one.
        TrackSize::Percent(_) if !available.is_finite() => kind != Kind::Auto,
        TrackSize::Auto | TrackSize::FitContent(_) | TrackSize::FitContentMath(_) => {
            kind == Kind::Min || kind == Kind::Max
        }
        TrackSize::MinContent => kind == Kind::Min,
        TrackSize::MaxContent => kind == Kind::Max,
        _ => false,
    };
    match track {
        TrackSize::MinMax(min, max) => bound(min) || bound(max),
        // A standalone `fr` is `minmax(auto, fr)`: its base is fed by the item's
        // min-content contribution (see [`min_fn_of`]), so it takes `Kind::Min`.
        TrackSize::Fr(_) => kind == Kind::Min,
        other => bound(other),
    }
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/distribute_up_to_limits.md")]
fn distribute_up_to_limits(
    tracks: &mut [Track],
    lo: usize,
    hi: usize,
    mut space: f32,
    affected: impl Fn(&Track) -> bool,
    base_of: impl Fn(&Track) -> f32,
    limit_of: impl Fn(&Track) -> f32,
) -> f32 {
    let is_growable = |t: &Track| affected(t) && base_of(t) + t.increase < limit_of(t);
    while space > DISTRIBUTE_THRESHOLD {
        let mut count = 0.0_f32;
        let mut min_headroom = f32::INFINITY;
        for t in &tracks[lo..hi] {
            if is_growable(t) {
                count += 1.0;
                min_headroom = min_headroom.min(limit_of(t) - (base_of(t) + t.increase));
            }
        }
        if count == 0.0 {
            break;
        }
        // Raise every growable track by the even share, capped so the tightest
        // track freezes exactly at its limit this round.
        let per = (space / count).min(min_headroom);
        for t in &mut tracks[lo..hi] {
            if is_growable(t) {
                t.increase += per;
                space -= per;
            }
        }
        // `per == space/count` exhausts the space; `per == min_headroom` freezes
        // ≥1 track and loops. A pure-headroom round with 0 leftover also exits.
        if per <= 0.0 {
            break;
        }
    }
    space
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/distribute_to_base.md")]
fn distribute_to_base(
    tracks: &mut [Track],
    start: usize,
    span: usize,
    contribution: f32,
    kind: Kind,
) {
    let hi = (start + span).min(tracks.len());
    let affected = |t: &Track| t.min_fn.kind() == Some(kind);
    if !tracks[start..hi].iter().any(affected) {
        return;
    }
    for t in &mut tracks[start..hi] {
        t.increase = 0.0;
    }
    let sum: f32 = tracks[start..hi].iter().map(|t| t.base).sum();
    let extra = (contribution - sum).max(0.0);
    if extra <= 0.0 {
        return;
    }
    let leftover = distribute_up_to_limits(
        tracks,
        start,
        hi,
        extra,
        affected,
        |t| t.base,
        |t| t.limit.min(t.fit_limit),
    );
    // §11.5.1 "distribute space beyond limits": if affected tracks are frozen,
    // give the remainder to those with an intrinsic max (or, failing that, to
    // all affected tracks) growing without limit.
    if leftover > DISTRIBUTE_THRESHOLD {
        let beyond = |t: &Track| affected(t) && t.max_fn.kind().is_some();
        if tracks[start..hi].iter().any(beyond) {
            distribute_up_to_limits(
                tracks,
                start,
                hi,
                leftover,
                beyond,
                |t| t.base,
                |_| f32::INFINITY,
            );
        } else {
            distribute_up_to_limits(
                tracks,
                start,
                hi,
                leftover,
                affected,
                |t| t.base,
                |_| f32::INFINITY,
            );
        }
    }
    for t in &mut tracks[start..hi] {
        t.planned_base = t.planned_base.max(t.increase);
        t.increase = 0.0;
    }
}

/// §11.5 sub-step (d) — grow the *growth limit* of the tracks spanned by one item
/// whose **max** sizing function is `kind`, to fit that kind's contribution.
/// Space goes only to tracks whose limit is still infinite (or newly
/// `infinitely_growable`); if none, it is distributed beyond limits to all
/// affected tracks.
fn distribute_to_limit(
    tracks: &mut [Track],
    start: usize,
    span: usize,
    contribution: f32,
    kind: Kind,
) {
    let hi = (start + span).min(tracks.len());
    let affected = |t: &Track| t.max_fn.kind() == Some(kind);
    if !tracks[start..hi].iter().any(affected) {
        return;
    }
    for t in &mut tracks[start..hi] {
        t.increase = 0.0;
        // Record that this item's max distribution targeted the track, so a
        // content-sized track with an infinite growth limit that ends this batch
        // with no increase is still committed to a finite growth limit (§12.5).
        if affected(t) {
            t.max_touched = true;
        }
    }
    let sum: f32 = tracks[start..hi]
        .iter()
        .map(|t| {
            if t.limit.is_infinite() {
                t.base
            } else {
                t.limit
            }
        })
        .sum();
    let extra = (contribution - sum).max(0.0);
    if extra <= 0.0 {
        return;
    }
    let growth_base = |t: &Track| {
        if t.limit.is_infinite() {
            t.base
        } else {
            t.limit
        }
    };
    let preferred = |t: &Track| t.infinitely_growable || t.limit.is_infinite();
    let leftover = distribute_up_to_limits(
        tracks,
        start,
        hi,
        extra,
        |t| affected(t) && preferred(t),
        growth_base,
        |t| t.fit_limit,
    );
    distribute_up_to_limits(tracks, start, hi, leftover, affected, growth_base, |t| {
        t.fit_limit
    });
    for t in &mut tracks[start..hi] {
        t.planned_limit = t.planned_limit.max(t.increase);
        t.increase = 0.0;
    }
}

/// The intrinsic [`Kind`]s some track is sized by, in [`KINDS`] order, read
/// through one of the two sizing functions.
fn present_kinds(tracks: &[Track], of: impl Fn(&Track) -> Option<Kind>) -> Vec<Kind> {
    KINDS
        .into_iter()
        .filter(|&kind| tracks.iter().any(|t| of(t) == Some(kind)))
        .collect()
}

/// Flush planned base increases into `base` (§11.5, between sub-steps).
fn flush_base(tracks: &mut [Track]) {
    for t in tracks {
        t.base += t.planned_base;
        t.planned_base = 0.0;
    }
}

/// Flush planned growth-limit increases into `limit` (§11.5.1); a limit that
/// turns from ∞ to finite becomes `base + planned` and is marked
/// `infinitely_growable` for the following growth-limit round.
fn flush_limit(tracks: &mut [Track]) {
    for t in tracks {
        if t.planned_limit > 0.0 {
            t.limit = if t.limit.is_infinite() {
                t.base + t.planned_limit
            } else {
                t.limit + t.planned_limit
            };
            t.infinitely_growable = true;
        } else if t.max_touched && t.limit.is_infinite() {
            // §12.5 span-1 simplification: a content-sized track whose growth limit
            // was targeted by a max distribution but needed no increase (its base
            // already covers the contribution) commits its infinite growth limit to
            // that finite value now — a hard cap for later spanning items, not an
            // infinitely-growable track.
            t.limit = t.base;
            t.infinitely_growable = false;
        } else {
            t.infinitely_growable = false;
        }
        t.planned_limit = 0.0;
        t.max_touched = false;
    }
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/resolve_intrinsic.md")]
fn resolve_intrinsic(tracks: &mut [Track], items: &[(usize, usize, Contributions)]) {
    // Only the kinds some track is actually sized by. A pass for an absent kind
    // would place no space, but it would not be a no-op: `flush_limit` clears the
    // §11.5.1 infinitely-growable flag, which the NEXT pass reads. Skipping the
    // absent kinds keeps a grid of plain `auto` tracks running exactly the one
    // pass it always ran.
    let min_kinds = present_kinds(tracks, |t| t.min_fn.kind());
    let max_kinds = present_kinds(tracks, |t| t.max_fn.kind());

    let mut order: Vec<usize> = (0..items.len()).collect();
    // Ascending span; span-1 items sized first (spec batches by span count).
    order.sort_by_key(|&i| items[i].1);
    let mut i = 0;
    while i < order.len() {
        let span = items[order[i]].1;
        let mut j = i;
        while j < order.len() && items[order[j]].1 == span {
            j += 1;
        }
        // Sub-step (a): intrinsic minimums → base sizes.
        for &kind in &min_kinds {
            for &k in &order[i..j] {
                let (start, sp, contributions) = items[k];
                if contributions.of(kind) > 0.0 {
                    distribute_to_base(tracks, start, sp, contributions.of(kind), kind);
                }
            }
            flush_base(tracks);
        }
        // Sub-step (d): intrinsic maximums → growth limits.
        for &kind in &max_kinds {
            for &k in &order[i..j] {
                let (start, sp, contributions) = items[k];
                if contributions.of(kind) > 0.0 {
                    distribute_to_limit(tracks, start, sp, contributions.of(kind), kind);
                }
            }
            flush_limit(tracks);
        }
        i = j;
    }
    // Final: raise any growth limit below its base; give any still-∞ growth
    // limit (empty tracks, flexible tracks) its base — this is what stops
    // §11.6 maximise from inflating flexible tracks.
    for t in tracks {
        if t.limit < t.base {
            t.limit = t.base;
        }
        if t.limit.is_infinite() {
            t.limit = t.base;
        }
    }
}

/// §11.6 — *Maximise Tracks*. Distributes the free space (`avail − Σ base`) to
/// every track's base up to its growth limit. (The spec's "infinite free space"
/// branch — for an indefinite container — is subsumed here: with a finite but
/// ample `avail`, every track reaches its growth limit anyway, and `auto`/`fr`
/// tracks have `limit == base` so they are untouched.)
fn maximise(tracks: &mut [Track], avail: f32) {
    let used: f32 = tracks.iter().map(|t| t.base).sum();
    let free = avail - used;
    if free <= 0.0 {
        return;
    }
    for t in tracks.iter_mut() {
        t.increase = 0.0;
    }
    distribute_up_to_limits(
        tracks,
        0,
        tracks.len(),
        free,
        |_| true,
        |t| t.base,
        |t| t.limit,
    );
    for t in tracks.iter_mut() {
        t.base += t.increase;
        t.increase = 0.0;
    }
}

/// §11.7.1 — *Find the Size of an fr*. The largest `fr` such that
/// `fr · factor ≥ base` for every flexible track; any flexible track failing
/// that is demoted to inflexible and the search restarts (it converges).
fn find_size_of_fr(tracks: &[Track], space: f32) -> f32 {
    if space <= 0.0 {
        return 0.0;
    }
    let mut hypo = f32::INFINITY;
    loop {
        let mut used = 0.0_f32;
        let mut factor_sum = 0.0_f32;
        for t in tracks {
            match t.max_fn {
                // A flexible track only counts while `factor · fr ≥ base`;
                // otherwise it is (this iteration) treated as inflexible.
                MaxFn::Flex(f) if f * hypo >= t.base => factor_sum += f,
                _ => used += t.base,
            }
        }
        let leftover = space - used;
        let prev = hypo;
        hypo = leftover / factor_sum.max(1.0);
        let valid = tracks.iter().all(|t| match t.max_fn {
            MaxFn::Flex(f) => f * hypo >= t.base || f * prev < t.base,
            _ => true,
        });
        if valid {
            break;
        }
    }
    hypo
}

/// Whether an item crossing this track supplies a max-content flex contribution.
pub(super) fn is_flexible(track: &TrackSize) -> bool {
    matches!(max_fn_of(track, 0.0), MaxFn::Flex(_))
}

/// Calculates the flex fraction from track floors and item contributions.
fn intrinsic_flex_fraction(
    tracks: &[Track],
    gap: f32,
    items: &[(usize, usize, Contributions)],
) -> f32 {
    let mut fraction = 0.0_f32;
    for track in tracks {
        if let MaxFn::Flex(factor) = track.max_fn {
            fraction = fraction.max(track.base / factor.max(1.0));
        }
    }
    for &(start, span, contributions) in items {
        let max_content = contributions.of(Kind::Max);
        let crossed = &tracks[start..start + span];
        if max_content.is_finite() && crossed.iter().any(|t| matches!(t.max_fn, MaxFn::Flex(_))) {
            let space = max_content - gap * span.saturating_sub(1) as f32;
            fraction = fraction.max(find_size_of_fr(crossed, space));
        }
    }
    fraction
}

/// Expands flexible tracks to at least their share of the used flex fraction.
/// A finite extent supplies available space. An unbounded extent uses the track
/// floors and the max-content contributions of items crossing flexible tracks.
/// When that intrinsic size falls outside the authored bounds, recomputes the
/// fraction with the constrained extent as the available space.
fn expand_flexible(
    tracks: &mut [Track],
    avail: f32,
    gap: f32,
    track_bounds: RangeInclusive<f32>,
    items: &[(usize, usize, Contributions)],
) {
    let fr = if avail.is_finite() {
        let used: f32 = tracks.iter().map(|t| t.base).sum();
        if avail <= used {
            return;
        }
        find_size_of_fr(tracks, avail)
    } else {
        let intrinsic = intrinsic_flex_fraction(tracks, gap, items);
        let extent = tracks
            .iter()
            .map(|t| match t.max_fn {
                MaxFn::Flex(factor) => t.base.max(factor * intrinsic),
                _ => t.base,
            })
            .sum::<f32>();
        let bounded = extent.clamp(*track_bounds.start(), *track_bounds.end());
        if bounded == extent {
            intrinsic
        } else {
            find_size_of_fr(tracks, bounded)
        }
    };
    for t in tracks.iter_mut() {
        if let MaxFn::Flex(factor) = t.max_fn {
            t.base = t.base.max(factor * fr);
        }
    }
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/stretch_auto.md")]
fn stretch_auto(tracks: &mut [Track], avail: f32) {
    let stretchable = |t: &Track| t.auto_max;
    let count = tracks.iter().filter(|t| stretchable(t)).count();
    if count == 0 {
        return;
    }
    let used: f32 = tracks.iter().map(|t| t.base).sum();
    let free = avail - used;
    if free <= 0.0 {
        return;
    }
    let per = free / count as f32;
    for t in tracks.iter_mut() {
        if stretchable(t) {
            t.base += per;
        }
    }
}

/// Whether the track needs a content extent to resolve a percentage.
pub(super) fn uses_percentage(track: &TrackSize) -> bool {
    match track {
        TrackSize::Percent(_)
        | TrackSize::FitContent(Dimension::Percent(_) | Dimension::Affine { .. }) => true,
        TrackSize::FitContentMath(limit) => limit.uses_percentage(),
        TrackSize::MinMax(min, max) => uses_percentage(min) || uses_percentage(max),
        _ => false,
    }
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/resolve_tracks.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_tracks(
    track_sizes: &[TrackSize],
    available: f32,
    gap: f32,
    definite: bool,
    stretch: bool,
    content_bounds: RangeInclusive<f32>,
    collapsed: &[bool],
    items: &[(usize, usize, Contributions)],
) -> Vec<f32> {
    let n = track_sizes.len();
    if n == 0 {
        return Vec::new();
    }
    let active = n - collapsed.iter().filter(|&&empty| empty).count();
    let gap_span = gap * active.saturating_sub(1) as f32;
    // The grid space the tracks share (gutters always consume their space).
    let avail = (available - gap_span).max(0.0);

    // §11.4 Initialize each track's base size and growth limit.
    let mut tracks: Vec<Track> = track_sizes
        .iter()
        .zip(collapsed)
        .map(|(track, &empty)| {
            let ts = if empty { &TrackSize::Fixed(0.0) } else { track };
            let min_fn = min_fn_of(ts, available);
            let max_fn = max_fn_of(ts, available);
            let base = match min_fn {
                MinFn::Fixed(v) => v,
                MinFn::Intrinsic(_) => 0.0,
            };
            let mut limit = match max_fn {
                MaxFn::Fixed(v) => v,
                MaxFn::Intrinsic(_) | MaxFn::Flex(_) => f32::INFINITY,
            };
            if limit < base {
                limit = base;
            }
            Track {
                base,
                limit,
                min_fn,
                max_fn,
                fit_limit: match ts {
                    TrackSize::FitContent(limit) => {
                        limit.resolve(available).unwrap_or(f32::INFINITY).max(0.0)
                    }
                    TrackSize::FitContentMath(limit) => {
                        limit.resolve(available).unwrap_or(f32::INFINITY).max(0.0)
                    }
                    _ => f32::INFINITY,
                },
                increase: 0.0,
                planned_base: 0.0,
                planned_limit: 0.0,
                infinitely_growable: false,
                max_touched: false,
                auto_max: max_is_auto(ts),
            }
        })
        .collect();

    resolve_intrinsic(&mut tracks, items); // §11.5
    maximise(&mut tracks, avail); // §11.6
    // An empty auto-sized axis has no item contributions for its flex tracks.
    // Its layout constraint is an upper bound and supplies no flex space.
    if !avail.is_finite() || definite || !items.is_empty() {
        let to_track_space = |extent: f32| (extent - gap_span).max(0.0);
        let track_bounds =
            to_track_space(*content_bounds.start())..=to_track_space(*content_bounds.end());
        expand_flexible(&mut tracks, avail, gap, track_bounds, items); // §11.7
    }
    if definite && stretch && avail.is_finite() {
        stretch_auto(&mut tracks, avail); // §11.8
    }

    tracks.iter().map(|t| t.base.max(0.0)).collect()
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/implicit_track.md")]
pub(super) fn implicit_track(auto: &[TrackSize], idx: usize) -> TrackSize {
    match auto.len() {
        0 => TrackSize::Auto,
        len => auto[idx % len].clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spanning_max_content_contribution_can_grow_past_a_capped_neighbor() {
        let tracks = [TrackSize::FitContent(Dimension::Px(40.0)), TrackSize::Auto];
        let items = [(
            0,
            2,
            Contributions {
                auto: 0.0,
                min: 0.0,
                max: 200.0,
            },
        )];
        let sizes = resolve_tracks(
            &tracks,
            300.0,
            0.0,
            true,
            false,
            0.0..=300.0,
            &[false; 2],
            &items,
        );
        assert_eq!(sizes, [40.0, 160.0]);
    }
}
