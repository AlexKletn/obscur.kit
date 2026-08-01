#![doc = include_str!("grid_track_sizing.doc/_module.md")]

use super::grid::TrackSize;

#[doc = include_str!("grid_track_sizing.doc/DISTRIBUTE_THRESHOLD.md")]
const DISTRIBUTE_THRESHOLD: f32 = 1e-6;

#[doc = include_str!("grid_track_sizing.doc/Kind.md")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    #[doc = include_str!("grid_track_sizing.doc/Auto.md")]
    Auto,
    #[doc = include_str!("grid_track_sizing.doc/Min.md")]
    Min,
    #[doc = include_str!("grid_track_sizing.doc/Max.md")]
    Max,
}

#[doc = include_str!("grid_track_sizing.doc/KINDS.md")]
pub(super) const KINDS: [Kind; 3] = [Kind::Auto, Kind::Min, Kind::Max];

#[doc = include_str!("grid_track_sizing.doc/MinFn.md")]
#[derive(Clone, Copy)]
enum MinFn {
    #[doc = include_str!("grid_track_sizing.doc/Fixed.md")]
    Fixed(f32),
    #[doc = include_str!("grid_track_sizing.doc/Intrinsic.md")]
    Intrinsic(Kind),
}

#[doc = include_str!("grid_track_sizing.doc/MaxFn.md")]
#[derive(Clone, Copy)]
enum MaxFn {
    #[doc = include_str!("grid_track_sizing.doc/Fixed_2.md")]
    Fixed(f32),
    #[doc = include_str!("grid_track_sizing.doc/Intrinsic_2.md")]
    Intrinsic(Kind),
    #[doc = include_str!("grid_track_sizing.doc/Flex.md")]
    Flex(f32),
}

impl MinFn {
    #[doc = include_str!("grid_track_sizing.doc/kind.md")]
    fn kind(self) -> Option<Kind> {
        match self {
            Self::Fixed(_) => None,
            Self::Intrinsic(kind) => Some(kind),
        }
    }
}

impl MaxFn {
    #[doc = include_str!("grid_track_sizing.doc/kind_2.md")]
    fn kind(self) -> Option<Kind> {
        match self {
            Self::Fixed(_) | Self::Flex(_) => None,
            Self::Intrinsic(kind) => Some(kind),
        }
    }
}

#[doc = include_str!("grid_track_sizing.doc/Contributions.md")]
#[derive(Clone, Copy, Default)]
pub(super) struct Contributions {
    auto: f32,
    min: f32,
    max: f32,
}

impl Contributions {
    #[doc = include_str!("grid_track_sizing.doc/of.md")]
    fn of(self, kind: Kind) -> f32 {
        match kind {
            Kind::Auto => self.auto,
            Kind::Min => self.min,
            Kind::Max => self.max,
        }
    }

    #[doc = include_str!("grid_track_sizing.doc/slot.md")]
    pub(super) fn slot(&mut self, kind: Kind) -> &mut f32 {
        match kind {
            Kind::Auto => &mut self.auto,
            Kind::Min => &mut self.min,
            Kind::Max => &mut self.max,
        }
    }
}

#[doc = include_str!("grid_track_sizing.doc/Track.md")]
struct Track {
    #[doc = include_str!("grid_track_sizing.doc/base.md")]
    base: f32,
    #[doc = include_str!("grid_track_sizing.doc/limit.md")]
    limit: f32,
    min_fn: MinFn,
    max_fn: MaxFn,
    #[doc = include_str!("grid_track_sizing.doc/increase.md")]
    increase: f32,
    #[doc = include_str!("grid_track_sizing.doc/planned_base.md")]
    planned_base: f32,
    #[doc = include_str!("grid_track_sizing.doc/planned_limit.md")]
    planned_limit: f32,
    #[doc = include_str!("grid_track_sizing.doc/infinitely_growable.md")]
    infinitely_growable: bool,
    #[doc = include_str!("grid_track_sizing.doc/max_touched.md")]
    max_touched: bool,
    #[doc = include_str!("grid_track_sizing.doc/auto_max.md")]
    auto_max: bool,
}

#[doc = include_str!("grid_track_sizing.doc/max_is_auto.md")]
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

#[doc = include_str!("grid_track_sizing.doc/min_fn_of.md")]
// `auto` / `min-content` / `fr` deliberately share the `Kind::Min` body but are
// kept as separate, self-documenting arms.
#[allow(clippy::match_same_arms)]
fn min_fn_of(track: &TrackSize, axis: f32) -> MinFn {
    match track {
        TrackSize::Fixed(px) => MinFn::Fixed(px.max(0.0)),
        TrackSize::Percent(f) => MinFn::Fixed((f * axis).max(0.0)),
        // `auto` as a MINIMUM is the item's automatic minimum size — accommodated
        // by the min-content contribution (§12.5 "intrinsic minimums"), NOT a
        // fill-available measure. Measuring it as min-content is what stops a block
        // item (whose `width: auto` would otherwise fill the offered width) from
        // inflating its track to the container's whole inline size.
        TrackSize::Auto => MinFn::Intrinsic(Kind::Min),
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

#[doc = include_str!("grid_track_sizing.doc/max_fn_of.md")]
// `auto` / `max-content` deliberately share the `Kind::Max` body but are kept as
// separate, self-documenting arms.
#[allow(clippy::match_same_arms)]
fn max_fn_of(track: &TrackSize, axis: f32) -> MaxFn {
    match track {
        TrackSize::Fixed(px) => MaxFn::Fixed(px.max(0.0)),
        TrackSize::Percent(f) => MaxFn::Fixed((f * axis).max(0.0)),
        // `auto` as a MAXIMUM is the item's max-content contribution (§12.5
        // "For max-content maximums" applies to `auto`/`max-content` maxes).
        TrackSize::Auto => MaxFn::Intrinsic(Kind::Max),
        TrackSize::MinContent => MaxFn::Intrinsic(Kind::Min),
        TrackSize::MaxContent => MaxFn::Intrinsic(Kind::Max),
        TrackSize::Fr(f) => MaxFn::Flex(f.max(0.0)),
        TrackSize::MinMax(_, max) => match max.as_ref() {
            TrackSize::MinMax(_, _) => MaxFn::Intrinsic(Kind::Auto),
            other => max_fn_of(other, axis),
        },
    }
}

#[doc = include_str!("grid_track_sizing.doc/track_takes.md")]
pub(super) fn track_takes(track: &TrackSize, kind: Kind) -> bool {
    let bound = |t: &TrackSize| match t {
        // `auto` is `minmax(min-content, max-content)`: its min is fed by the
        // min-content contribution and its max by the max-content one.
        TrackSize::Auto => kind == Kind::Min || kind == Kind::Max,
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

#[doc = include_str!("grid_track_sizing.doc/distribute_up_to_limits.md")]
fn distribute_up_to_limits(
    tracks: &mut [Track],
    lo: usize,
    hi: usize,
    mut space: f32,
    affected: impl Fn(&Track) -> bool,
    base_of: impl Fn(&Track) -> f32,
    limit_of: impl Fn(&Track) -> f32,
) -> f32 {
    while space > DISTRIBUTE_THRESHOLD {
        // The growable set: affected tracks not yet at their limit.
        let mut count = 0.0_f32;
        let mut min_headroom = f32::INFINITY;
        for t in &tracks[lo..hi] {
            if affected(t) && base_of(t) + t.increase < limit_of(t) {
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
            if affected(t) && base_of(t) + t.increase < limit_of(t) {
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

#[doc = include_str!("grid_track_sizing.doc/distribute_to_base.md")]
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
    let leftover =
        distribute_up_to_limits(tracks, start, hi, extra, affected, |t| t.base, |t| t.limit);
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

#[doc = include_str!("grid_track_sizing.doc/distribute_to_limit.md")]
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
    let growable: Vec<usize> = (start..hi)
        .filter(|&i| {
            affected(&tracks[i]) && (tracks[i].infinitely_growable || tracks[i].limit.is_infinite())
        })
        .collect();
    if growable.is_empty() {
        distribute_up_to_limits(
            tracks,
            start,
            hi,
            extra,
            affected,
            |t| {
                if t.limit.is_infinite() {
                    t.base
                } else {
                    t.limit
                }
            },
            |_| f32::INFINITY,
        );
    } else {
        let per = extra / growable.len() as f32;
        for &i in &growable {
            tracks[i].increase = per;
        }
    }
    for t in &mut tracks[start..hi] {
        t.planned_limit = t.planned_limit.max(t.increase);
        t.increase = 0.0;
    }
}

#[doc = include_str!("grid_track_sizing.doc/present_kinds.md")]
fn present_kinds(tracks: &[Track], of: impl Fn(&Track) -> Option<Kind>) -> Vec<Kind> {
    KINDS
        .into_iter()
        .filter(|&kind| tracks.iter().any(|t| of(t) == Some(kind)))
        .collect()
}

#[doc = include_str!("grid_track_sizing.doc/flush_base.md")]
fn flush_base(tracks: &mut [Track]) {
    for t in tracks {
        t.base += t.planned_base;
        t.planned_base = 0.0;
    }
}

#[doc = include_str!("grid_track_sizing.doc/flush_limit.md")]
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

#[doc = include_str!("grid_track_sizing.doc/resolve_intrinsic.md")]
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

#[doc = include_str!("grid_track_sizing.doc/maximise.md")]
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

#[doc = include_str!("grid_track_sizing.doc/find_size_of_fr.md")]
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

#[doc = include_str!("grid_track_sizing.doc/expand_flexible.md")]
fn expand_flexible(tracks: &mut [Track], avail: f32) {
    let used: f32 = tracks.iter().map(|t| t.base).sum();
    if avail - used <= 0.0 {
        return;
    }
    let fr = find_size_of_fr(tracks, avail);
    for t in tracks.iter_mut() {
        if let MaxFn::Flex(factor) = t.max_fn {
            t.base = t.base.max(factor * fr);
        }
    }
}

#[doc = include_str!("grid_track_sizing.doc/stretch_auto.md")]
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

#[doc = include_str!("grid_track_sizing.doc/resolve_tracks.md")]
pub(super) fn resolve_tracks(
    track_sizes: &[TrackSize],
    available: f32,
    gap: f32,
    definite: bool,
    stretch: bool,
    items: &[(usize, usize, Contributions)],
) -> Vec<f32> {
    let n = track_sizes.len();
    if n == 0 {
        return Vec::new();
    }
    let gap_span = gap * (n.saturating_sub(1) as f32);
    // The grid space the tracks share (gutters always consume their space).
    let avail = (available - gap_span).max(0.0);

    // §11.4 Initialize each track's base size and growth limit.
    let mut tracks: Vec<Track> = track_sizes
        .iter()
        .map(|ts| {
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
    expand_flexible(&mut tracks, avail); // §11.7
    if definite && stretch {
        stretch_auto(&mut tracks, avail); // §11.8
    }

    tracks.iter().map(|t| t.base.max(0.0)).collect()
}

#[doc = include_str!("grid_track_sizing.doc/implicit_track.md")]
pub(super) fn implicit_track(auto: &[TrackSize], idx: usize) -> TrackSize {
    match auto.len() {
        0 => TrackSize::Auto,
        len => auto[idx % len].clone(),
    }
}
