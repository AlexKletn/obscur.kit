//! `GridLayout`: a real CSS-grid track-sizing formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{AlignItems, JustifyContent, LayoutBox, LayoutStyle};

use super::grid::{GridTemplate, TrackSize};
use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput, clamp_content};

/// CSS-grid layout with real per-track sizing (spec §4.1, §4.2).
///
/// Children flow **row-major** into a grid described by a [`GridTemplate`]:
/// child `0` fills column `0` of row `0`, child `1` column `1`, and so on,
/// wrapping to a new row once every column in the current row is filled. Each
/// column and row is sized independently by its [`TrackSize`] — a fixed pixel
/// length, a flexible `fr` fraction, a content-sized `auto`, or a
/// `minmax(min, max)` clamp.
///
/// # Configuration
///
/// The grid carries a [`GridTemplate`] (the column and row track lists) plus a
/// horizontal `column_gap` and vertical `row_gap`. The uniform constructor
/// [`GridLayout::new`] builds an *N equal columns* grid by using `N` `1fr`
/// columns and all-implicit auto rows, so a plain `new(n)` still lays out `n`
/// equal-width columns exactly as before.
///
/// # Algorithm
///
/// 1. **Content area** — the container's own explicit `width`/`height` (resolved
///    against `constraints.max`) gives the content width/height; an `Auto` axis
///    falls back to `constraints.max` minus `padding` (each clamped ≥ 0). Tracks,
///    `fr` distribution, and percentages all resolve against this extent.
/// 2. **Column track sizing** — with `n = columns.len()` and
///    `available = content_width - (n - 1) * column_gap`, each column resolves to:
///    `Fixed(px)` → `px`; `Percent(f)` → `f * content_width` (its basis is the
///    grid container's content extent, known only here); `Auto` → the largest
///    measured content width among the items auto-placed in it (measured under
///    loose constraints); `MinContent` / `MaxContent` → the largest min-content /
///    max-content contribution among its items, each measured by offering the item
///    no inline space / unbounded inline space (CSS Sizing 3 §4.1) — unlike `Auto`
///    these ignore the space the container has and do not stretch into it;
///    `Fr(f)` → a share `f / Σfr` of the space left after the
///    fixed + percentage + auto tracks; and
///    `MinMax(min, max)` → the `max` function resolved, then floored at the `min`
///    function (so `minmax(px, fr)` grows flexibly from `px` up). `fr` space is
///    distributed exactly like [`FlexLayout`](super::FlexLayout) shares free
///    space among grow factors.
/// 3. **Row track sizing** — the same resolution over the explicit `rows` using
///    `content_height` and `row_gap`. Rows past the explicit list are *implicit*
///    and sized by the template's `grid-auto-rows`
///    ([`auto_rows`](GridTemplate::auto_rows)) — cycled by the n-th implicit row
///    using index `n % len` — defaulting (empty list) to `Auto`, as tall as the
///    tallest item placed in them.
/// 4. **Placement** — each child carries a [`GridPlacement`](super::GridPlacement)
///    (its `grid-column` / `grid-row` line pair). Placement runs in two phases:
///    *Phase A* honours every item with a definite column and/or row line — a
///    fully-definite item lands at its cell, a half-definite item locks the
///    definite axis and auto-flows the other — marking each `col-span × row-span`
///    block occupied. *Phase B* then places the remaining fully-auto items
///    row-major by **first-fit**: each drops into the first free cell (skipping
///    occupied cells, wrapping a too-wide span to the next row) that fits its
///    span. Definite lines past the template add implicit columns and rows past
///    the template are implicit — both sized by `grid-auto-columns` /
///    `grid-auto-rows` (cycled), defaulting to `Auto`. With every child left at the default
///    all-`Auto` placement this reduces exactly to `(i % n_cols, i / n_cols)`.
/// 5. **Sizing** — an item spanning `s` columns is `Σ(spanned column widths) +
///    (s - 1) * column_gap` wide (analogously for a row span with `row_gap`); a
///    spanning item attributes its content size to its first spanned track. It is
///    measured with its width fixed to that span; a single content-sized (`Auto`
///    / implicit) row keeps its own measured height, otherwise it fills its
///    spanned row tracks. It is placed at
///    `(padding.left + Σ(prior column widths) + start_col * column_gap,
///      padding.top  + Σ(prior row heights)  + start_row * row_gap)`.
/// 6. **Border box** — an explicit container axis sizes to that content extent
///    plus `padding`; an `Auto` axis sizes to the bounding box of the placed
///    children plus `padding`. The result is clamped into `constraints`.
///
/// # Out of scope (later steps)
///
/// This does real track sizing plus line-based placement and spanning (including
/// negative, from-end grid lines resolved against the track count), but is still a
/// subset: no *dense* auto-placement packing (placement is first-fit, never
/// backfilling earlier holes), no named grid lines, and no
/// alignment of an item within its track. Half-definite items always auto-flow
/// the missing axis. Those belong to later work; the CSS parsing and runtime
/// wiring of placement is a later step.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridLayout {
    /// The explicit column/row track template.
    template: GridTemplate,
    /// Horizontal gap between adjacent columns, in logical pixels.
    column_gap: f32,
    /// Vertical gap between adjacent rows, in logical pixels.
    row_gap: f32,
}

impl GridLayout {
    /// Creates a uniform grid of `columns` equal-width columns and no gaps.
    ///
    /// `columns` is clamped to a minimum of `1`, so a zero column count still
    /// produces a valid single-column grid. Each column is a `1fr` track, so the
    /// columns share the content width equally — the historical uniform-grid
    /// behaviour — and every row is implicit (auto, content-sized).
    #[must_use]
    pub fn new(columns: usize) -> Self {
        let n = columns.max(1);
        Self {
            template: GridTemplate::columns_only(vec![TrackSize::Fr(1.0); n]),
            column_gap: 0.0,
            row_gap: 0.0,
        }
    }

    /// Creates a grid from an explicit [`GridTemplate`] and the two gaps.
    #[must_use]
    pub fn with_template(template: GridTemplate, column_gap: f32, row_gap: f32) -> Self {
        Self {
            template,
            column_gap,
            row_gap,
        }
    }

    /// Returns a copy with the horizontal `column_gap` replaced.
    #[must_use]
    pub fn with_column_gap(mut self, column_gap: f32) -> Self {
        self.column_gap = column_gap;
        self
    }

    /// Returns a copy with the vertical `row_gap` replaced.
    #[must_use]
    pub fn with_row_gap(mut self, row_gap: f32) -> Self {
        self.row_gap = row_gap;
        self
    }

    /// The grid's track template.
    #[must_use]
    pub const fn template(&self) -> &GridTemplate {
        &self.template
    }

    /// The number of columns (always at least `1`).
    ///
    /// This reports the *explicit* column count (the explicit tracks plus one copy
    /// of any auto-repeat pattern); the real per-content column count — which
    /// depends on how many auto-repeat copies fit the container — is computed
    /// during [`layout`](LayoutAlgorithm::layout).
    #[must_use]
    pub fn columns(&self) -> usize {
        self.template.columns().explicit_len().max(1)
    }

    /// The horizontal gap between columns, in logical pixels.
    #[must_use]
    pub const fn column_gap(&self) -> f32 {
        self.column_gap
    }

    /// The vertical gap between rows, in logical pixels.
    #[must_use]
    pub const fn row_gap(&self) -> f32 {
        self.row_gap
    }
}

// ---------------------------------------------------------------------------
// Canonical CSS Grid track-sizing algorithm (drafts.csswg.org/css-grid-2 §11).
//
// Every track carries TWO numbers — a growing floor `base` and a ceiling
// `limit` (its "growth limit", possibly ∞) — plus its resolved min/max sizing
// functions. The six staged steps below (§11.4 init, §11.5 resolve-intrinsic
// span distribution, §11.5.1 distribute-up-to-limits, §11.6 maximise, §11.7
// find-size-of-fr with restart, §11.8 stretch) replace the previous one-value
// `(base, fr, floor)` single-pass model. Reimplemented from the spec; the
// `taffy` crate was read for cross-reference only (not copied).
//
// # Content contributions, and what `auto` approximates
//
// §11.5 distributes an item's contribution to the tracks it spans, and WHICH
// contribution depends on the sizing function of each spanned track. Three are
// modelled here, one per intrinsic track type ([`Contributions`]):
//
// * a `min-content` track takes the item's min-content contribution — measured
//   under an offer of no space;
// * a `max-content` track takes its max-content contribution — measured under an
//   unbounded offer;
// * an `auto` track takes a single measure of the item under the container's own
//   loose offer. This is an APPROXIMATION, and the one place this file departs
//   from the spec: `auto` is formally `minmax(min-content, max-content)`, whose
//   base should grow to the min-content contribution and whose growth limit to
//   the max-content one. The single loose measure lands between the two — it is
//   what the item wants in the space this container has — and it is what every
//   `auto` track in this engine has always been sized by. Splitting it is a
//   behaviour change to the overwhelmingly common track type, so it stays a
//   documented approximation; the two keyword tracks, which have no legacy
//   behaviour to preserve, probe for real.
// ---------------------------------------------------------------------------

/// Small epsilon so space distribution stops at ~0 instead of looping on f32
/// rounding error (spec's distribution loops have no natural fixpoint at 0).
const DISTRIBUTE_THRESHOLD: f32 = 1e-6;

/// Which content contribution an intrinsic sizing function is fed by (§11.5).
///
/// One kind per intrinsic track type, and the reason this is an enum rather than
/// a bool: a track sized by `min-content` and one sized by `max-content` are both
/// "intrinsic", but they must be grown by *different numbers*, so the item has to
/// supply — and each sub-step has to pick — the right one.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// An `auto` track: the item's size under the container's own loose offer
    /// (the documented approximation — see the module note above).
    Auto,
    /// A `min-content` track: the item measured under an offer of no space.
    Min,
    /// A `max-content` track: the item measured under an unbounded offer.
    Max,
}

/// Every intrinsic kind, in the order §11.5 works through them: the smallest
/// contribution first, so a track sized by a larger one is never grown twice.
const KINDS: [Kind; 3] = [Kind::Auto, Kind::Min, Kind::Max];

/// A track's resolved *min* sizing function (§11.4). A `Percent`/`Fixed` min is
/// a definite length; `auto`, `min-content`, `max-content` (and a standalone /
/// `minmax` `fr` min) are intrinsic.
#[derive(Clone, Copy)]
enum MinFn {
    /// A definite length (fixed px or a resolved percentage).
    Fixed(f32),
    /// An intrinsic minimum — grows to fit the [`Kind`]'s content contribution.
    Intrinsic(Kind),
}

/// A track's resolved *max* sizing function (§11.4).
#[derive(Clone, Copy)]
enum MaxFn {
    /// A definite length (fixed px or a resolved percentage): the growth limit.
    Fixed(f32),
    /// An intrinsic maximum — an infinite initial growth limit, grown to fit the
    /// [`Kind`]'s content contribution.
    Intrinsic(Kind),
    /// A flexible (`fr`) maximum with the given flex factor.
    Flex(f32),
}

impl MinFn {
    /// The contribution kind this min sizing function is fed by, or [`None`] when
    /// it is a definite length that takes no contribution at all.
    fn kind(self) -> Option<Kind> {
        match self {
            Self::Fixed(_) => None,
            Self::Intrinsic(kind) => Some(kind),
        }
    }
}

impl MaxFn {
    /// The contribution kind this max sizing function is fed by; [`None`] for a
    /// definite length and for `fr` (whose growth limit is settled by §11.7, not
    /// by content).
    fn kind(self) -> Option<Kind> {
        match self {
            Self::Fixed(_) | Self::Flex(_) => None,
            Self::Intrinsic(kind) => Some(kind),
        }
    }
}

/// One item's content contribution for each intrinsic [`Kind`] it crosses.
///
/// A kind the item crosses no track of stays `0` and is never measured: an item
/// in a grid of plain `auto` tracks costs exactly the one measure it always did.
#[derive(Clone, Copy, Default)]
struct Contributions {
    auto: f32,
    min: f32,
    max: f32,
}

impl Contributions {
    /// This item's contribution for `kind`.
    fn of(self, kind: Kind) -> f32 {
        match kind {
            Kind::Auto => self.auto,
            Kind::Min => self.min,
            Kind::Max => self.max,
        }
    }

    /// The contribution slot for `kind`, to fill in from a measure.
    fn slot(&mut self, kind: Kind) -> &mut f32 {
        match kind {
            Kind::Auto => &mut self.auto,
            Kind::Min => &mut self.min,
            Kind::Max => &mut self.max,
        }
    }
}

/// One track's two-value sizing state during the §11 pipeline.
struct Track {
    /// The growing floor (§11 "base size").
    base: f32,
    /// The ceiling (§11 "growth limit"), `f32::INFINITY` while unbounded.
    limit: f32,
    min_fn: MinFn,
    max_fn: MaxFn,
    /// Scratch: this item's increase within one distribution round.
    increase: f32,
    /// Scratch: the largest per-item base increase planned across a sub-step.
    planned_base: f32,
    /// Scratch: the largest per-item growth-limit increase planned across a
    /// sub-step.
    planned_limit: f32,
    /// Whether this track's growth limit turned finite in the current
    /// growth-limit round (§11.5.1 "infinitely growable" flag).
    infinitely_growable: bool,
    /// Scratch: whether an item's growth-limit (max) distribution targeted this
    /// track in the current batch. A content-sized track that is targeted but
    /// receives no increase (its contribution is already covered by its base) must
    /// still have its infinite growth limit committed to a finite value — the §12.5
    /// span-1 "set the growth limit to the maximum of the items' max-content
    /// contributions" simplification — so a later spanning item sees it as a hard
    /// cap rather than as still-infinitely-growable (the Peter Salas example).
    max_touched: bool,
    /// Whether this track's *max* sizing function is literally `auto` (as opposed
    /// to `max-content`, which shares the same [`Kind::Max`] content contribution).
    /// §11.8 *Stretch auto Tracks* grows only `auto`-max tracks, so this preserves
    /// the distinction the growth-limit [`Kind`] no longer carries.
    auto_max: bool,
}

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

/// The `min` sizing function of a track (§11.4). `axis` is the content extent a
/// `Percent` bound resolves against.
///
/// Per §7.2.3 a standalone `fr` is formally `minmax(auto, fr)` — an `auto`
/// (content-based) minimum. Its base therefore grows to the items' **minimum
/// contribution**, which §12.5 accommodates with the min-content contribution
/// (measured under an offer of no space, [`Kind::Min`]) — not the loose/max-content
/// measure, so a reflowable item still shrinks to its longest word rather than
/// inflating the flexible track to its preferred size. This is what makes an `fr`
/// track blown past its fair share by content restart the fr algorithm (§12.7.1);
/// a track with no content min keeps a base of `0` and a plain `1fr` split. An
/// `fr` used as a `minmax` *min* is invalid and acts as `0`.
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

/// The `max` sizing function of a track (§11.4). A nested `minmax` as the `max`
/// bound is not meaningful; it is treated as `auto` (content-sized).
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

/// Whether a track takes the `kind` content contribution — i.e. an item spanning
/// it must be measured that way to feed §11.5.
///
/// True when either of the track's two sizing functions is that kind: its min
/// grows the base, its max grows the growth limit, and either one alone is reason
/// enough to measure. A pure fixed/percentage/`fr` track takes no contribution at
/// all (a standalone `fr`'s min is floored at `0`, see [`min_fn_of`]).
fn track_takes(track: &TrackSize, kind: Kind) -> bool {
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

/// §11.5.1 core — *Distribute extra space up to limits*. Grows the affected
/// tracks in `tracks[lo..hi]` equally, freezing each as it reaches `limit_of`,
/// by repeatedly raising all still-growable tracks by the smallest per-track
/// headroom (or the even share, whichever is smaller). Accumulates into each
/// track's `increase`; returns the space that could not be placed (all affected
/// tracks hit their limit). Reused by §11.6 maximise.
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

/// §11.5 sub-step (a) — grow the *base* of the tracks spanned by one item whose
/// **min** sizing function is `kind`, to fit that kind's contribution, then
/// record the per-track increase as this sub-step's planned increase (flushed by
/// [`flush_base`] so same-batch items don't double-count).
///
/// Tracks of another kind in the same span are left alone: they have their own
/// sub-step, with their own contribution. They do count toward the `sum` the
/// contribution is measured against, per §11.5.1 — the space to place is what the
/// item needs beyond *every* track it spans.
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

/// §11.5 — *Resolve Intrinsic Track Sizes*. Processes items in ascending span
/// batches (span-1 first). Within each batch, sub-step (a) grows intrinsic-min
/// bases and sub-step (d) grows intrinsic-max growth limits, each via
/// [`distribute_up_to_limits`] over only the item's spanned tracks, flushing
/// planned increases between sub-steps. Items are
/// `(start, span, contributions)`.
///
/// Each sub-step runs once per intrinsic [`Kind`], because the spec's sub-steps
/// are exactly that: "content-based minimums" grows the min-content tracks by the
/// min-content contribution, "max-content minimums" grows the max-content tracks
/// by the max-content one. A track has one min and one max sizing function, so
/// the passes touch disjoint sets.
///
/// Their order is not fully inert, though: besides the `sum` each later pass
/// measures its extra space against, `flush_limit` clears the §11.5.1
/// infinitely-growable flag that the next pass reads (see the body). For a grid
/// whose tracks are sized by one kind — the common case — that is unobservable.
/// For a mixed grid it is a real ordering dependency, and no spec-exactness is
/// claimed for one.
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

/// §11.7 — *Expand Flexible Tracks*. When there is free space, size each
/// flexible track's base to `max(base, factor · fr)`.
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

/// §11.8 — *Stretch auto Tracks*. Splits any remaining positive free space
/// equally among tracks whose *max* sizing function is `auto`. The caller only
/// invokes this when the container axis is definite and its content alignment is
/// `stretch`/`normal` (the default), matching the spec's preconditions.
///
/// `auto` only: a `min-content` / `max-content` track is sized by its content and
/// nothing else, which is the one place the three visibly part company. §11.8 is
/// written against `auto` for exactly that reason.
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

/// Resolves a list of tracks to concrete pixel sizes along one axis via the
/// canonical CSS-grid track-sizing algorithm (§11).
///
/// `available` is the content extent along the axis and `gap` the inter-track
/// gap; the *grid space* the tracks share is `available − gap · (n − 1)`.
/// `definite` is whether the container's size along this axis is authored (not
/// `auto`) and `stretch` whether its content alignment routes to §11.8 (the
/// `stretch`/`normal` default). `items` are the grid items as
/// `(start_track, span, contributions)`; only items crossing an intrinsic track
/// need a real contribution, and only of the [`Kind`]s they cross (others pass
/// `0`).
///
/// Simple cases — fixed tracks, a single `1fr` split with no floors, and
/// non-spanning items — resolve to the same sizes as the previous single-pass
/// model; fr-restart, span distribution, maximise, and stretch change only the
/// cases they were added to fix.
fn resolve_tracks(
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

/// Running start offsets for a track list: `offsets[i] = origin + Σ_{k<i}
/// (sizes[k] + gap)`. Length is `sizes.len()`.
///
/// A `collapsed[i]` track (an empty `auto-fit` track) contributes neither its size
/// nor a following gap, so the gaps on either side of it merge into one and the
/// occupied tracks pack together (CSS §4.1). With no collapsed track this is
/// byte-identical to the plain running-offset accumulation.
fn track_offsets(sizes: &[f32], gap: f32, origin: f32, collapsed: &[bool]) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(sizes.len());
    let mut cursor = origin;
    for (i, &size) in sizes.iter().enumerate() {
        if collapsed.get(i).copied().unwrap_or(false) {
            // Collapsed to zero width; parks at the cursor without advancing it.
            offsets.push(cursor);
            continue;
        }
        offsets.push(cursor);
        cursor += size + gap;
    }
    offsets
}

/// The `(leading, extra_gap)` a content-distribution mode contributes when the
/// tracks along one axis leave leftover free space (spec §10.4, `align-content` /
/// `justify-content` on a grid container).
///
/// `n` non-collapsed tracks occupy `Σ sizes + gap * (n − 1)` of the axis; the
/// `leftover = content_extent − that` (when positive) is distributed:
/// * `FlexStart` — nothing (the historical packing; both `0.0`).
/// * `FlexEnd` — a `leading` of the full leftover, no extra gap.
/// * `Center` — a `leading` of half the leftover, no extra gap.
/// * `SpaceBetween` — no leading, an extra `leftover/(n−1)` between tracks (n≥2;
///   n<2 falls back to `FlexStart`).
/// * `SpaceAround` — a `leftover/(2n)` half-gap at each end plus `leftover/n`
///   between adjacent tracks.
/// * `SpaceEvenly` — an equal `leftover/(n+1)` gap at the ends and between tracks.
///
/// A non-positive leftover, an empty axis, or the `FlexStart` default all yield
/// `(0.0, 0.0)`, so a default grid is byte-identical to the un-distributed
/// packing.
fn content_distribution(n: usize, leftover: f32, mode: JustifyContent) -> (f32, f32) {
    if n == 0 || leftover <= 0.0 {
        return (0.0, 0.0);
    }
    match mode {
        // `stretch` grows the tracks themselves to consume the leftover (handled by
        // the track-stretch step), so it leaves no gaps to distribute here.
        JustifyContent::FlexStart | JustifyContent::Stretch => (0.0, 0.0),
        JustifyContent::FlexEnd => (leftover, 0.0),
        JustifyContent::Center => (leftover / 2.0, 0.0),
        JustifyContent::SpaceBetween => {
            if n >= 2 {
                (0.0, leftover / (n - 1) as f32)
            } else {
                (0.0, 0.0)
            }
        }
        JustifyContent::SpaceAround => {
            let unit = leftover / n as f32;
            (unit / 2.0, unit)
        }
        JustifyContent::SpaceEvenly => {
            let unit = leftover / (n + 1) as f32;
            (unit, unit)
        }
    }
}

/// Running start offsets for a track list with leftover free space distributed
/// per a content-distribution `mode` (`justify-content` on columns /
/// `align-content` on rows).
///
/// This generalises [`track_offsets`]: it computes the leftover
/// (`content_extent − (Σ non-collapsed sizes + gap * (n − 1))`) and, when it is
/// positive, shifts the first track by a `leading` offset and inserts an
/// `extra_gap` between adjacent tracks per [`content_distribution`]. The track
/// **sizes** are unchanged — only their **positions** move — so items placed in a
/// track move with it. With the [`FlexStart`](JustifyContent::FlexStart) default
/// (or no leftover) `leading` and `extra_gap` are both zero and this is
/// byte-identical to [`track_offsets`].
fn distributed_track_offsets(
    sizes: &[f32],
    gap: f32,
    origin: f32,
    collapsed: &[bool],
    content_extent: f32,
    mode: JustifyContent,
) -> Vec<f32> {
    // Non-collapsed track count and total size, for the leftover computation. A
    // collapsed (empty `auto-fit`) track contributes neither size nor gap.
    let mut n = 0_usize;
    let mut sum = 0.0_f32;
    for (i, &size) in sizes.iter().enumerate() {
        if collapsed.get(i).copied().unwrap_or(false) {
            continue;
        }
        n += 1;
        sum += size;
    }
    let leftover = content_extent - (sum + gap * (n.saturating_sub(1) as f32));
    let (leading, extra_gap) = content_distribution(n, leftover, mode);
    // The un-distributed default path stays byte-identical to `track_offsets`.
    if leading == 0.0 && extra_gap == 0.0 {
        return track_offsets(sizes, gap, origin, collapsed);
    }
    let mut offsets = Vec::with_capacity(sizes.len());
    let mut cursor = origin + leading;
    for (i, &size) in sizes.iter().enumerate() {
        if collapsed.get(i).copied().unwrap_or(false) {
            offsets.push(cursor);
            continue;
        }
        offsets.push(cursor);
        cursor += size + gap + extra_gap;
    }
    offsets
}

/// Marks which tracks of an `auto-fit` repeat region collapse: a track inside the
/// `[start, end)` generated range that no placed item covers. Returns an
/// all-`false` vector (nothing collapses) when there is no `auto-fit` region — so
/// an `auto-fill` or plain template is untouched.
fn collapsed_tracks(
    len: usize,
    auto_fit: Option<(usize, usize)>,
    anchors: &[usize],
    spans: &[usize],
) -> Vec<bool> {
    let mut collapsed = vec![false; len];
    let Some((start, end)) = auto_fit else {
        return collapsed;
    };
    let mut used = vec![false; len];
    for (&anchor, &span) in anchors.iter().zip(spans.iter()) {
        for slot in used.iter_mut().take((anchor + span).min(len)).skip(anchor) {
            *slot = true;
        }
    }
    for (track, slot) in collapsed
        .iter_mut()
        .enumerate()
        .take(end.min(len))
        .skip(start)
    {
        if !used[track] {
            *slot = true;
        }
    }
    collapsed
}

/// The sizing function for the `idx`-th *implicit* track on an axis, from the
/// axis's `grid-auto-columns` / `grid-auto-rows` list.
///
/// The list is **cycled** (CSS §7.5): the n-th implicit track uses index
/// `n % len`. An **empty** list means the default content-sized [`TrackSize::Auto`]
/// — byte-identical to the historical behaviour where every implicit track was
/// `Auto`. An `fr` in an implicit track resolves the same way as an explicit `fr`
/// (via [`resolve_tracks`]); because an implicit track has no free-space basis of
/// its own, on an indefinite axis its leftover is zero and it collapses to its
/// floor (its `min`, else `0`) — documented, not special-cased.
fn implicit_track(auto: &[TrackSize], idx: usize) -> TrackSize {
    match auto.len() {
        0 => TrackSize::Auto,
        len => auto[idx % len].clone(),
    }
}

/// The pixel extent a `span`-track cell covers: the sum of the spanned track
/// sizes plus the `span - 1` interior gaps between them.
fn span_extent(sizes: &[f32], start: usize, span: usize, gap: f32) -> f32 {
    let end = (start + span).min(sizes.len());
    let base: f32 = sizes
        .get(start..end)
        .map_or(0.0, |s| s.iter().copied().sum());
    base + gap * (span.saturating_sub(1) as f32)
}

/// Whether a `rspan × cspan` block anchored at `(row, col)` is entirely free in
/// the occupancy grid. Cells in rows past the grid's current height are free.
fn block_free(
    occ: &[Vec<bool>],
    row: usize,
    rspan: usize,
    col: usize,
    cspan: usize,
    n_cols: usize,
) -> bool {
    if col + cspan > n_cols {
        return false;
    }
    for line in occ.iter().skip(row).take(rspan) {
        for &cell in line.iter().skip(col).take(cspan) {
            if cell {
                return false;
            }
        }
    }
    true
}

/// Marks a `rspan × cspan` block occupied, growing `occ` to reach the block.
///
/// Rows are at least `n_cols` wide but grow to `col + cspan` so a *column-flow*
/// item placed past the explicit column count still marks its cells (implicit
/// columns). For row flow `col + cspan ≤ n_cols` always, so the width is exactly
/// `n_cols` and this is byte-identical to the previous fixed-width behaviour.
fn mark_block(
    occ: &mut Vec<Vec<bool>>,
    row: usize,
    rspan: usize,
    col: usize,
    cspan: usize,
    n_cols: usize,
) {
    let width = n_cols.max(col + cspan);
    while occ.len() < row + rspan {
        occ.push(vec![false; width]);
    }
    for line in occ.iter_mut() {
        if line.len() < width {
            line.resize(width, false);
        }
    }
    for line in occ.iter_mut().skip(row).take(rspan) {
        for cell in line.iter_mut().skip(col).take(cspan) {
            *cell = true;
        }
    }
}

/// First row (scanning up from 0) where a column-locked block fits.
fn first_row(occ: &[Vec<bool>], col: usize, cspan: usize, rspan: usize, n_cols: usize) -> usize {
    let mut row = 0;
    while !block_free(occ, row, rspan, col, cspan, n_cols) {
        row += 1;
    }
    row
}

/// First column in a fixed row where a row-locked block fits (`0` if the row is
/// already full — a degenerate overlap kept deterministic).
fn first_col(occ: &[Vec<bool>], row: usize, cspan: usize, rspan: usize, n_cols: usize) -> usize {
    if cspan > n_cols {
        return 0;
    }
    (0..=(n_cols - cspan))
        .find(|&col| block_free(occ, row, rspan, col, cspan, n_cols))
        .unwrap_or(0)
}

/// First free `(col, row)` in row-major order for a fully-auto block (first-fit).
///
/// This scans from the grid origin every call — the *dense* row-major search
/// (`grid-auto-flow: row dense`), which backfills earlier holes.
fn first_fit(occ: &[Vec<bool>], cspan: usize, rspan: usize, n_cols: usize) -> (usize, usize) {
    if cspan > n_cols {
        return (0, 0);
    }
    let mut row = 0;
    loop {
        if let Some(col) =
            (0..=(n_cols - cspan)).find(|&col| block_free(occ, row, rspan, col, cspan, n_cols))
        {
            return (col, row);
        }
        row += 1;
    }
}

/// Whether a `rspan × cspan` block anchored at `(row, col)` is entirely free,
/// treating any cell past the current occupancy extent (in either dimension) as
/// free. Unlike [`block_free`] this does **not** bound the column axis, so it
/// suits column-flow placement into implicit (growing) columns.
fn block_is_free(occ: &[Vec<bool>], row: usize, rspan: usize, col: usize, cspan: usize) -> bool {
    for line in occ.iter().skip(row).take(rspan) {
        for c in col..col + cspan {
            if line.get(c).copied().unwrap_or(false) {
                return false;
            }
        }
    }
    true
}

/// First free `(col, row)` in **row-major** order at or after the cursor
/// `(start_row, start_col)` — the *sparse* row-major search
/// (`grid-auto-flow: row`).
///
/// The current row is scanned from `start_col` (never earlier — the cursor only
/// moves forward, so a hole left behind is not backfilled); each later row is
/// scanned from column `0`. A span wider than the row wraps to the next row.
fn first_fit_from(
    occ: &[Vec<bool>],
    cspan: usize,
    rspan: usize,
    n_cols: usize,
    start_row: usize,
    start_col: usize,
) -> (usize, usize) {
    if cspan > n_cols {
        return (0, start_row);
    }
    let hi = n_cols - cspan;
    let mut row = start_row;
    let mut col_lo = start_col;
    loop {
        if col_lo <= hi
            && let Some(col) =
                (col_lo..=hi).find(|&col| block_free(occ, row, rspan, col, cspan, n_cols))
        {
            return (col, row);
        }
        row += 1;
        col_lo = 0;
    }
}

/// First free `(col, row)` in **column-major** order at or after the cursor
/// `(start_col, start_row)` — the column-flow search (`grid-auto-flow: column`).
///
/// Columns grow implicitly; `n_rows` is the fixed row-wrap bound. The current
/// column is scanned from `start_row`; each later column is scanned from row `0`.
/// Passing `(0, 0)` yields the dense column search (search from the origin every
/// call); passing the running cursor yields the sparse one.
fn first_fit_col_major(
    occ: &[Vec<bool>],
    cspan: usize,
    rspan: usize,
    n_rows: usize,
    start_col: usize,
    start_row: usize,
) -> (usize, usize) {
    if rspan > n_rows {
        return (start_col, 0);
    }
    let hi = n_rows - rspan;
    let mut col = start_col;
    let mut row_lo = start_row;
    loop {
        if row_lo <= hi
            && let Some(row) = (row_lo..=hi).find(|&row| block_is_free(occ, row, rspan, col, cspan))
        {
            return (col, row);
        }
        col += 1;
        row_lo = 0;
    }
}

/// How a grid item is positioned within its cell on one axis, after resolving
/// the container default (`justify-items` / `align-items`) against any per-item
/// override (`justify-self` / `align-self`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum CellAlign {
    /// Fill the cell extent (the default; measured with a fixed size).
    Stretch,
    /// Place the item at the cell start.
    Start,
    /// Place the item at the cell end (`cell − item`).
    End,
    /// Center the item within the cell (`(cell − item) / 2`).
    Center,
}

/// Classifies an effective [`AlignItems`] into a grid [`CellAlign`]. The flexbox
/// spellings (`FlexStart` / `FlexEnd`) fold onto the grid `Start` / `End`
/// physical alignments so an authored `align-items: start` (which the runtime
/// maps to [`AlignItems::FlexStart`]) behaves as a grid start.
const fn cell_align(align: AlignItems) -> CellAlign {
    match align {
        AlignItems::Stretch => CellAlign::Stretch,
        AlignItems::Center => CellAlign::Center,
        AlignItems::FlexStart | AlignItems::Start => CellAlign::Start,
        AlignItems::FlexEnd | AlignItems::End => CellAlign::End,
    }
}

/// The offset of an `item`-sized box within a `cell`-sized extent for `align`.
/// `Stretch` and `Start` sit at the cell start; `End` at `cell − item`; `Center`
/// halves the slack.
fn align_offset(align: CellAlign, cell: f32, item: f32) -> f32 {
    match align {
        CellAlign::Stretch | CellAlign::Start => 0.0,
        CellAlign::End => cell - item,
        CellAlign::Center => (cell - item) / 2.0,
    }
}

impl LayoutAlgorithm for GridLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let padding = style.padding;
        let pad_x = padding.left + padding.right;
        let pad_y = padding.top + padding.bottom;

        // Step 1: content area the grid sizes its tracks against. An explicit
        // container `width` / `height` is honoured (resolved against the available
        // max, so a fixed px or a percentage of it both work) so column/row tracks,
        // `fr` distribution, and percentages all resolve against the authored size;
        // an `Auto` dimension falls back to the available content extent, which is
        // byte-identical to the previous fill-available behaviour.
        // `box-sizing` governs how an explicit container size maps to the content
        // extent the tracks size against: `content-box` (the default) takes it
        // as-is, `border-box` subtracts the axis padding (floored at `0`). The
        // `Auto` branch already subtracts padding, so both modes are consistent
        // with the flex container — an explicitly-sized grid stays at its authored
        // (border-box) size with no phantom inflation.
        let content_width = style
            .width
            .resolve(constraints.max.width)
            .map_or(constraints.max.width - pad_x, |w| {
                style.box_sizing.content_extent(w, pad_x)
            })
            .max(0.0);
        let content_height = style
            .height
            .resolve(constraints.max.height)
            .map_or(constraints.max.height - pad_y, |h| {
                style.box_sizing.content_extent(h, pad_y)
            })
            .max(0.0);

        // Expand each axis's track list into a concrete track vector against the
        // content extent: a `repeat(auto-fill | auto-fit, …)`'s repetition count
        // depends on how many copies fit that extent (spec §4.1). For a template
        // WITHOUT an auto-repeat this is byte-identical to its explicit track
        // vector, so every non-auto-repeat template lays out exactly as before.
        // `col_autofit` / `row_autofit` carry the generated `auto-fit` track range
        // (if any) so empty ones collapse after placement.
        let (columns_expanded, col_autofit) = self
            .template
            .columns()
            .expand(content_width, self.column_gap);
        let (rows_expanded, row_autofit) =
            self.template.rows().expand(content_height, self.row_gap);

        // An empty column template still lays out one flexible column so items
        // always have somewhere to go.
        let fallback_cols = [TrackSize::Fr(1.0)];
        let columns: &[TrackSize] = if columns_expanded.is_empty() {
            &fallback_cols
        } else {
            &columns_expanded
        };
        let count = children.len();

        // Resolve each child's line-based placement into a definite/auto anchor
        // plus its column/row span (spec §4.13, a subset). A default placement is
        // all-`Auto` → `(None, 1)` on both axes: pure row-major auto-placement.
        let templ_cols_len = columns.len();
        let templ_rows_len = rows_expanded.len();
        let mut col_anchor = Vec::with_capacity(count);
        let mut col_span = Vec::with_capacity(count);
        let mut row_anchor = Vec::with_capacity(count);
        let mut row_span = Vec::with_capacity(count);
        for child in children {
            let placement = child.style().grid_placement();
            let (ca, cs) = placement.resolve_column(templ_cols_len);
            let (ra, rs) = placement.resolve_row(templ_rows_len);
            col_anchor.push(ca);
            col_span.push(cs.max(1));
            row_anchor.push(ra);
            row_span.push(rs.max(1));
        }

        // Column count: the explicit tracks, grown to cover any definite column
        // line that overflows the template and any span wider than it (implicit
        // auto columns). With all-`Auto` placement (every span 1, no definite
        // line) this is exactly the template width — the historical grid.
        let mut n_cols = columns.len().max(1);
        for i in 0..count {
            if let Some(c) = col_anchor[i] {
                n_cols = n_cols.max(c + col_span[i]);
            }
            n_cols = n_cols.max(col_span[i]);
        }

        // Row-wrap bound for **column** auto-flow: the explicit rows, grown to
        // cover any definite row line / row span (the row analogue of `n_cols`).
        // Only consulted when the container flows column-major.
        let mut n_rows_flow = templ_rows_len.max(1);
        for i in 0..count {
            if let Some(r) = row_anchor[i] {
                n_rows_flow = n_rows_flow.max(r + row_span[i]);
            }
            n_rows_flow = n_rows_flow.max(row_span[i]);
        }

        // Two-phase placement into an occupancy grid. Phase A — items with a
        // definite column and/or row: honour the definite axis, auto-flow any
        // missing axis. Phase B — fully-auto items, placed per the container's
        // `grid-auto-flow`: row- vs column-major (the iteration axis) and sparse
        // vs dense (a forward-only cursor vs a from-the-origin search that
        // backfills earlier holes). With the default `Row` (sparse) flow and
        // all-`Auto` placement Phase A is empty and Phase B fills cell `i` in
        // row-major order — byte-identical to the previous auto-placement.
        let flow = style.grid_auto_flow;
        let mut occ: Vec<Vec<bool>> = Vec::new();
        let mut place_col = vec![0_usize; count];
        let mut place_row = vec![0_usize; count];
        for i in 0..count {
            let (ca, ra) = (col_anchor[i], row_anchor[i]);
            if ca.is_none() && ra.is_none() {
                continue; // fully auto → Phase B
            }
            let (cspan, rspan) = (col_span[i], row_span[i]);
            let (c0, r0) = match (ca, ra) {
                (Some(c), Some(r)) => (c, r),
                (Some(c), None) => (c, first_row(&occ, c, cspan, rspan, n_cols)),
                (None, Some(r)) => (first_col(&occ, r, cspan, rspan, n_cols), r),
                (None, None) => unreachable!(),
            };
            mark_block(&mut occ, r0, rspan, c0, cspan, n_cols);
            place_col[i] = c0;
            place_row[i] = r0;
        }
        // Phase B: the auto-placement cursor. `is_dense` searches from the origin
        // every item (backfilling holes); the sparse cursor only moves forward.
        let (mut cur_major, mut cur_minor) = (0_usize, 0_usize);
        for i in 0..count {
            if col_anchor[i].is_some() || row_anchor[i].is_some() {
                continue; // placed in Phase A
            }
            let (cspan, rspan) = (col_span[i], row_span[i]);
            let (c0, r0) = if flow.is_column() {
                // Column-major: `cur_major` is the column, `cur_minor` the row.
                let (sc, sr) = if flow.is_dense() {
                    (0, 0)
                } else {
                    (cur_major, cur_minor)
                };
                first_fit_col_major(&occ, cspan, rspan, n_rows_flow, sc, sr)
            } else if flow.is_dense() {
                first_fit(&occ, cspan, rspan, n_cols)
            } else {
                // Row-major sparse: `cur_major` is the row, `cur_minor` the column.
                first_fit_from(&occ, cspan, rspan, n_cols, cur_major, cur_minor)
            };
            mark_block(&mut occ, r0, rspan, c0, cspan, n_cols);
            place_col[i] = c0;
            place_row[i] = r0;
            if !flow.is_dense() {
                (cur_major, cur_minor) = if flow.is_column() { (c0, r0) } else { (r0, c0) };
            }
        }
        // Column-flow items may land past the explicit column count (implicit
        // columns); grow `n_cols` to cover them. For row flow every placed column
        // is already within `n_cols`, so this is a no-op (byte-identical).
        for i in 0..count {
            n_cols = n_cols.max(place_col[i] + col_span[i]);
        }

        // Total rows the grid spans (implicit auto rows past the template).
        let n_rows = (0..count)
            .map(|i| place_row[i] + row_span[i])
            .max()
            .unwrap_or(0);

        // Effective column tracks: the explicit template, padded with implicit
        // tracks sized by `grid-auto-columns` (cycled; empty → `Auto`).
        let auto_cols = self.template.auto_columns();
        let explicit_cols = columns.len();
        let cols_effective: Vec<TrackSize> = (0..n_cols)
            .map(|c| {
                columns
                    .get(c)
                    .cloned()
                    .unwrap_or_else(|| implicit_track(auto_cols, c - explicit_cols))
            })
            .collect();

        // Step 2: column track sizing via the canonical §11 pipeline. Each item
        // crossing an intrinsic (`auto`/`fr`/intrinsic-`minmax`) column supplies
        // a content-width contribution measured under a loose constraint; the
        // algorithm distributes each spanning item's contribution across ALL its
        // spanned tracks (not just the first). `justify-content: stretch`/`normal`
        // (the default) routes leftover space to §11.8 stretch of `auto` columns.
        let loose = BoxConstraints::loose(Size::new(content_width, content_height));
        let col_definite = !style.width.is_auto();
        let col_stretch = matches!(
            style.justify_content,
            JustifyContent::FlexStart | JustifyContent::Stretch
        );
        let mut col_items: Vec<(usize, usize, Contributions)> = Vec::with_capacity(count);
        for i in 0..count {
            let (start, span) = (place_col[i], col_span[i]);
            let spanned = &cols_effective[start..(start + span).min(cols_effective.len())];
            // One measure per intrinsic kind this item's span actually crosses.
            // The `min-content` / `max-content` probes offer no space / unbounded
            // space along the INLINE axis, exactly as `intrinsic_inline_size` does
            // for a flow box: the item answers with the width it needs rather than
            // the width it was given (`BoxConstraints::constrain_allowing_overflow`).
            let mut contributions = Contributions::default();
            for kind in KINDS {
                if !spanned.iter().any(|t| track_takes(t, kind)) {
                    continue;
                }
                let offer = match kind {
                    Kind::Auto => loose,
                    Kind::Min => BoxConstraints::loose(Size::new(0.0, content_height)),
                    Kind::Max => BoxConstraints::loose(Size::new(f32::INFINITY, content_height)),
                };
                *contributions.slot(kind) = measurer.measure_child(i, offer).width;
            }
            col_items.push((start, span, contributions));
        }
        let mut col_sizes = resolve_tracks(
            &cols_effective,
            content_width,
            self.column_gap,
            col_definite,
            col_stretch,
            &col_items,
        );
        // `auto-fit`: a generated column no item landed in collapses to zero width
        // (its surrounding gaps collapse in `track_offsets`).
        let col_collapsed = collapsed_tracks(n_cols, col_autofit, &place_col, &col_span);
        for (size, &collapsed) in col_sizes.iter_mut().zip(col_collapsed.iter()) {
            if collapsed {
                *size = 0.0;
            }
        }

        // Measure each item with its width fixed to its spanned columns (spanned
        // track widths + interior column gaps), height loose; this feeds both
        // content-sized row heights and the placed item boxes.
        let mut item_size = vec![Size::ZERO; count];
        for i in 0..count {
            let w = span_extent(&col_sizes, place_col[i], col_span[i], self.column_gap);
            item_size[i] = measurer.measure_child(
                i,
                BoxConstraints {
                    min: Size::new(w, 0.0),
                    max: Size::new(w, content_height),
                    percent_basis: None,
                },
            );
        }

        // Step 3: row track sizing via the same §11 pipeline. Explicit rows come
        // from the template; rows past it are implicit, sized by `grid-auto-rows`
        // (cycled; empty → auto). Each item crossing an intrinsic row contributes
        // its measured height (with its width already fixed to its column span);
        // the algorithm distributes a row-spanning item's height across its
        // spanned rows. `align-content: stretch`/`normal` routes to §11.8.
        let auto_rows = self.template.auto_rows();
        let explicit_rows = rows_expanded.len();
        let rows_effective: Vec<TrackSize> = (0..n_rows)
            .map(|r| {
                rows_expanded
                    .get(r)
                    .cloned()
                    .unwrap_or_else(|| implicit_track(auto_rows, r - explicit_rows))
            })
            .collect();
        let row_definite = !style.height.is_auto();
        let row_stretch = matches!(
            style.align_content,
            JustifyContent::FlexStart | JustifyContent::Stretch
        );
        let mut row_items: Vec<(usize, usize, Contributions)> = Vec::with_capacity(count);
        for i in 0..count {
            let (start, span) = (place_row[i], row_span[i]);
            let spanned = &rows_effective[start..(start + span).min(rows_effective.len())];
            // In the BLOCK axis all three contributions are the same number, and it
            // needs no probe: the item's width is already fixed to its column span,
            // and at a fixed width a box does not reflow vertically — its
            // min-content, max-content and natural heights coincide. So a
            // `min-content` row is fed the height already measured above, and costs
            // no extra measure at all (the inline axis is where they diverge).
            let mut contributions = Contributions::default();
            for kind in KINDS {
                if spanned.iter().any(|t| track_takes(t, kind)) {
                    *contributions.slot(kind) = item_size[i].height;
                }
            }
            row_items.push((start, span, contributions));
        }
        let mut row_sizes = resolve_tracks(
            &rows_effective,
            content_height,
            self.row_gap,
            row_definite,
            row_stretch,
            &row_items,
        );
        // `auto-fit` on the row axis: collapse the generated rows with no item.
        let row_collapsed = collapsed_tracks(n_rows, row_autofit, &place_row, &row_span);
        for (size, &collapsed) in row_sizes.iter_mut().zip(row_collapsed.iter()) {
            if collapsed {
                *size = 0.0;
            }
        }

        // Step 4 & 5: place each item into its (spanned) track cell. Leftover free
        // space on each axis (when the tracks do not fill the content extent) is
        // distributed per `justify-content` (columns / inline axis) and
        // `align-content` (rows / block axis); the default `FlexStart` on both
        // packs the tracks at the start — byte-identical to before.
        let col_offsets = distributed_track_offsets(
            &col_sizes,
            self.column_gap,
            padding.left,
            &col_collapsed,
            content_width,
            style.justify_content,
        );
        let row_offsets = distributed_track_offsets(
            &row_sizes,
            self.row_gap,
            padding.top,
            &row_collapsed,
            content_height,
            style.align_content,
        );

        let mut child_boxes = Vec::with_capacity(count);
        // Bounding box of placed children; starts at the padding origin so an
        // empty grid sizes to its padding only.
        let mut max_right = padding.left;
        let mut max_bottom = padding.top;
        for (i, size) in item_size.iter().enumerate() {
            let (c0, cspan) = (place_col[i], col_span[i]);
            let (r0, rspan) = (place_row[i], row_span[i]);
            let cell_x = col_offsets[c0];
            let cell_y = row_offsets[r0];
            let cell_w = span_extent(&col_sizes, c0, cspan, self.column_gap);
            // A single content-sized row keeps the item's own measured height;
            // otherwise the cell fills its spanned row tracks (+ interior gaps).
            let cell_h = if rspan == 1 && matches!(rows_effective[r0], TrackSize::Auto) {
                size.height
            } else {
                span_extent(&row_sizes, r0, rspan, self.row_gap)
            };

            // Resolve this item's effective alignment on each axis: a per-item
            // `justify-self` / `align-self` (if not `auto`) overrides the
            // container's `justify-items` / `align-items` default. The inline axis
            // is the column (x) axis; the block axis is the row (y) axis.
            let cs = children[i].style();
            let inline = cell_align(cs.justify_self.resolve(style.justify_items));
            let block = cell_align(cs.align_self.resolve(style.align_items));

            let (x, y, w, h) = if inline == CellAlign::Stretch && block == CellAlign::Stretch {
                // Default: the item fills its cell — byte-identical to before.
                (cell_x, cell_y, cell_w, cell_h)
            } else {
                // At least one axis aligns the item within its cell. Re-measure
                // the natural size: an axis that still stretches keeps a size
                // fixed to the cell extent, an aligned axis is loose up to it.
                let natural = measurer.measure_child(
                    i,
                    BoxConstraints {
                        min: Size::new(
                            if inline == CellAlign::Stretch {
                                cell_w
                            } else {
                                0.0
                            },
                            if block == CellAlign::Stretch {
                                cell_h
                            } else {
                                0.0
                            },
                        ),
                        max: Size::new(cell_w, cell_h),
                        percent_basis: None,
                    },
                );
                let iw = if inline == CellAlign::Stretch {
                    cell_w
                } else {
                    natural.width
                };
                let ih = if block == CellAlign::Stretch {
                    cell_h
                } else {
                    natural.height
                };
                (
                    cell_x + align_offset(inline, cell_w, iw),
                    cell_y + align_offset(block, cell_h, ih),
                    iw,
                    ih,
                )
            };
            child_boxes.push(LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h))));
            // The grid's own extent is defined by its tracks (the cell), not by
            // where an aligned item sits inside it, so the bounding box grows by
            // the cell — byte-identical to the fill case.
            max_right = max_right.max(cell_x + cell_w);
            max_bottom = max_bottom.max(cell_y + cell_h);
        }

        // Step 6: border box, clamped. When the container sets an explicit size the
        // border box is that content extent plus padding (so the container *is* the
        // authored size, even if its tracks do not fill it); an `Auto` axis sizes to
        // the children's bounding box plus padding, exactly as before.
        //
        // `max_right`/`max_bottom` are accumulated from the content origin (they
        // start at `padding.left`/`padding.top`), so subtracting that origin back
        // off yields the CONTENT extent — the axis `clamp_content` operates on.
        let own_content_width = if style.width.is_auto() {
            max_right - padding.left
        } else {
            content_width
        };
        let own_content_height = if style.height.is_auto() {
            max_bottom - padding.top
        } else {
            content_height
        };

        // Clamp the container's OWN box into its authored min/max, exactly as flow
        // and flex do (spec §4.2). Without this an authored `min-height` /
        // `max-width` on a `display: grid` element is silently ignored — the same
        // defect flow and flex carried before `clamp_content` was factored out.
        let own_w = clamp_content(
            own_content_width,
            style.min_width,
            style.max_width,
            constraints.max.width,
            pad_x,
            style.box_sizing,
        );
        let own_h = clamp_content(
            own_content_height,
            style.min_height,
            style.max_height,
            constraints.max.height,
            pad_y,
            style.box_sizing,
        );
        let border_box = Size::new(own_w + pad_x, own_h + pad_y);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::{GridLayout, GridTemplate, TrackSize};
    use crate::algorithm::{
        AutoRepeat, AutoRepeatMode, FixedChildMeasurer, GridAutoFlow, GridLine, GridPlacement,
        LayoutAlgorithm, LayoutChild, TrackList,
    };
    use crate::model::{Dimension, JustifyContent, LayoutStyle};
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    fn placed(raw: u64, placement: GridPlacement) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(raw),
            LayoutStyle::default().with_grid_placement(placement),
        )
    }

    fn grid(
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
        col_gap: f32,
        row_gap: f32,
    ) -> GridLayout {
        GridLayout::with_template(GridTemplate::new(columns, rows), col_gap, row_gap)
    }

    // ---- §4.2 own-box min/max clamp (parity with flow + flex) ---------------

    #[test]
    fn min_height_floors_a_short_grid() {
        // One 20-tall row, but `min-height: 200` floors the container. Grid used
        // to honour only the parent constraints, so an authored `min-height` on a
        // `display: grid` element was silently dropped — flow and flex clamped.
        let style = LayoutStyle {
            min_height: Dimension::Px(200.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 20.0)]);
        let out = grid(vec![TrackSize::Fixed(100.0)], vec![], 0.0, 0.0).layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 200.0);
    }

    #[test]
    fn max_height_caps_a_tall_grid() {
        let style = LayoutStyle {
            max_height: Dimension::Px(50.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 300.0)]);
        let out = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(300.0)],
            0.0,
            0.0,
        )
        .layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 50.0);
    }

    #[test]
    fn max_width_caps_an_explicitly_sized_grid() {
        // The clamp applies to an authored width too, not just the `Auto` axis.
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            max_width: Dimension::Px(120.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 20.0)]);
        let out = grid(vec![TrackSize::Fixed(100.0)], vec![], 0.0, 0.0).layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().width, 120.0);
    }

    #[test]
    fn the_clamp_is_measured_from_the_content_box_under_padding() {
        // The clamp operates on the CONTENT extent, then padding is added back.
        // `min-height: 200` under `content-box` sizing means 200 of *content*, so
        // the border box is 200 + 10 + 10 — the same rule flow applies. This is
        // what makes the `max_bottom - padding.top` round-trip load-bearing.
        let style = LayoutStyle {
            min_height: Dimension::Px(200.0),
            padding: Insets::uniform(10.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 20.0)]);
        let out = grid(vec![TrackSize::Fixed(100.0)], vec![], 0.0, 0.0).layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 220.0);
    }

    #[test]
    fn auto_min_max_leaves_the_grid_byte_identical() {
        // The clamp must be a no-op when nothing is authored: an `Auto`-height
        // grid still sizes to its tracks' bounding box plus padding.
        let style = LayoutStyle {
            padding: Insets::uniform(10.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 20.0)]);
        let out = grid(vec![TrackSize::Fixed(100.0)], vec![], 0.0, 0.0).layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 40.0);
    }

    // (a) `100px 1fr 2fr` across a 400px content width, no gap.
    #[test]
    fn fixed_and_fr_columns_share_leftover() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(10.0, 20.0),
            Size::new(10.0, 20.0),
            Size::new(10.0, 20.0),
        ]);
        let g = grid(
            vec![
                TrackSize::Fixed(100.0),
                TrackSize::Fr(1.0),
                TrackSize::Fr(2.0),
            ],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // 1fr = (400-100)/3 = 100, 2fr = 200.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 100.0);
        assert_eq!(b[1].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].size().width, 100.0);
        assert_eq!(b[2].origin(), Point::new(200.0, 0.0));
        assert_eq!(b[2].size().width, 200.0);
        assert_eq!(out.size(), Size::new(400.0, 20.0));
    }

    // (b) `1fr 1fr` → two equal 50% columns.
    #[test]
    fn equal_fr_columns_split_evenly() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 150.0);
        assert_eq!(b[1].origin().x, 150.0);
        assert_eq!(b[1].size().width, 150.0);
    }

    // (c) a 10px column gap shifts columns and reduces the fr space.
    #[test]
    fn column_gap_reduces_fr_space() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fixed(100.0), TrackSize::Fr(1.0)],
            vec![],
            10.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // available = 400 - 10 (gap) = 390; fr = 390 - 100 = 290.
        assert_eq!(b[0].size().width, 100.0);
        assert_eq!(b[1].origin().x, 110.0); // 100 + gap 10
        assert_eq!(b[1].size().width, 290.0);
    }

    // (d) auto-placement: 5 items in 2 columns → 3 rows; item 4 at (col0,row2).
    #[test]
    fn row_major_auto_placement() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3), child(4), child(5)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 5]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // 5 items / 2 cols = 3 rows (rows tall 10 each).
        assert_eq!(b[3].origin(), Point::new(100.0, 10.0)); // col1,row1
        assert_eq!(b[4].origin(), Point::new(0.0, 20.0)); // col0,row2
        assert_eq!(out.size(), Size::new(200.0, 30.0));
    }

    // (e) implicit auto rows are sized to the tallest item in the row.
    #[test]
    fn implicit_rows_size_to_tallest_item() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3), child(4)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(0.0, 20.0),
            Size::new(0.0, 35.0), // tallest in row 0
            Size::new(0.0, 15.0),
            Size::new(0.0, 5.0),
        ]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Row 0 tallest = 35 → row 1 starts at y = 35.
        assert_eq!(b[2].origin(), Point::new(0.0, 35.0));
        assert_eq!(b[3].origin(), Point::new(100.0, 35.0));
        // Border box height = row0(35) + row1(15) = 50.
        assert_eq!(out.size(), Size::new(200.0, 50.0));
    }

    /// Builds a grid whose implicit tracks are sized by `grid-auto-columns` /
    /// `grid-auto-rows` (each cycled; empty → `auto`).
    fn grid_auto(
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
        auto_columns: Vec<TrackSize>,
        auto_rows: Vec<TrackSize>,
    ) -> GridLayout {
        GridLayout::with_template(
            GridTemplate::new(columns, rows).with_auto_tracks(auto_columns, auto_rows),
            0.0,
            0.0,
        )
    }

    // (auto-rows) explicit `100px 100px` columns, no explicit rows, and
    // `grid-auto-rows: 40px`: all rows are implicit and are 40px tall (NOT sized to
    // the content), so the two wrapped items land at y = 40.
    #[test]
    fn grid_auto_rows_sizes_implicit_rows() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3), child(4)];
        // Content heights (10) differ from the 40px track; the track wins.
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 4]);
        let g = grid_auto(
            vec![TrackSize::Fixed(100.0), TrackSize::Fixed(100.0)],
            vec![],
            vec![],
            vec![TrackSize::Fixed(40.0)],
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Row 0 items at y=0; wrapped row-1 items at y=40 (not the 10px content).
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[2].origin(), Point::new(0.0, 40.0));
        assert_eq!(b[3].origin(), Point::new(100.0, 40.0));
        assert_eq!(b[2].size().height, 40.0);
        // Two 40px implicit rows → border box height 80.
        assert_eq!(out.size(), Size::new(200.0, 80.0));
    }

    // (auto-columns) an item placed past the explicit columns (`grid-column: 4` in a
    // 2-column grid) opens implicit column 3, sized by `grid-auto-columns: 50px`.
    #[test]
    fn grid_auto_columns_sizes_implicit_column() {
        let style = LayoutStyle::default();
        let placement = GridPlacement::column(GridLine::Line(4), GridLine::Auto);
        let children = [placed(1, placement)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0)]);
        let g = grid_auto(
            vec![TrackSize::Fixed(100.0), TrackSize::Fixed(100.0)],
            vec![],
            vec![TrackSize::Fixed(50.0)],
            vec![],
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(1000.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Columns: 100,100 explicit then implicit 50 (col 2) + 50 (col 3). The item
        // sits in col 3 → x = 100+100+50 = 250, width 50.
        assert_eq!(b[0].origin().x, 250.0);
        assert_eq!(b[0].size().width, 50.0);
    }

    // (auto-rows cycling) `grid-auto-rows: 30px 50px` cycles across implicit rows:
    // row0=30, row1=50, row2=30 → single-column items stack at y 0/30/80.
    #[test]
    fn grid_auto_rows_cycles_two_sizes() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 3]);
        let g = grid_auto(
            vec![TrackSize::Fr(1.0)],
            vec![],
            vec![],
            vec![TrackSize::Fixed(30.0), TrackSize::Fixed(50.0)],
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().y, 0.0);
        assert_eq!(b[1].origin().y, 30.0); // after the 30px row 0
        assert_eq!(b[2].origin().y, 80.0); // after 30 + 50
        // Border box height = 30 + 50 + 30 = 110.
        assert_eq!(out.size().height, 110.0);
    }

    // (f) an `Auto` column sizes to its items' content width.
    #[test]
    fn auto_column_sizes_to_content() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(80.0, 20.0), // col 0 (auto) content
            Size::new(30.0, 20.0), // col 1 (fr)
        ]);
        let g = grid(vec![TrackSize::Auto, TrackSize::Fr(1.0)], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Auto column = 80; fr column = 300 - 80 = 220.
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 80.0);
        assert_eq!(b[1].origin().x, 80.0);
        assert_eq!(b[1].size().width, 220.0);
    }

    // (g) `minmax(100px, 1fr)` never resolves below 100.
    #[test]
    fn minmax_floors_fr_column() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        // Two minmax(100px,1fr) columns in a 150px width: 2*100 = 200 > 150, so
        // each is floored at 100 (no room for fr growth).
        let g = grid(
            vec![
                TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0)),
                TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0)),
            ],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(150.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 100.0);
        assert_eq!(b[1].size().width, 100.0);
        assert_eq!(b[1].origin().x, 100.0);
    }

    // (p1) `30% 1fr` on a 400px content width: 30% = 120, fr takes the 280 remainder.
    #[test]
    fn percent_and_fr_columns_share_leftover() {
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Percent(0.30), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // 30% of 400 = 120; the 1fr track takes the 280 remainder. (f32 percent
        // arithmetic rounds 0.30 * 400 to ~120.00001, so compare with tolerance.)
        assert_eq!(b[0].origin().x, 0.0);
        assert!((b[0].size().width - 120.0).abs() < 1e-3);
        assert!((b[1].origin().x - 120.0).abs() < 1e-3);
        assert!((b[1].size().width - 280.0).abs() < 1e-3);
    }

    // (p2) two `50%` columns fill the content width exactly: 200 / 200 on 400px.
    #[test]
    fn two_percent_columns_split_content_width() {
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Percent(0.50), TrackSize::Percent(0.50)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 200.0);
        assert_eq!(b[1].origin().x, 200.0);
        assert_eq!(b[1].size().width, 200.0);
    }

    // (p3) a `Percent` inside `minmax(30%, 1fr)` acts as the floor: with a 300px
    // fixed track ahead of it on a 400px width there is no room to grow, so the
    // minmax track floors at 30% of 400 = 120 (not below).
    #[test]
    fn minmax_percent_floor_holds() {
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![
                TrackSize::Fixed(300.0),
                TrackSize::minmax(TrackSize::Percent(0.30), TrackSize::Fr(1.0)),
            ],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 300.0);
        // No leftover for fr growth (300 + 120 > 400), so the minmax floors at 120
        // (f32 percent arithmetic rounds 0.30 * 400 to ~120.00001).
        assert_eq!(b[1].origin().x, 300.0);
        assert!((b[1].size().width - 120.0).abs() < 1e-3);
    }

    // (p4) percentage ROW tracks resolve against the container content height.
    #[test]
    fn percent_rows_resolve_against_content_height() {
        let style = LayoutStyle {
            height: Dimension::Px(200.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0)],
            vec![TrackSize::Percent(0.25), TrackSize::Fr(1.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 500.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Row 0 = 25% of 200 = 50; the second child starts on row 1 at y = 50.
        assert_eq!(b[1].origin().y, 50.0);
    }

    // (h) regression: `GridLayout::new(1)` lays a single equal column.
    #[test]
    fn new_one_is_single_equal_column() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 20.0), Size::new(50.0, 30.0)]);
        let out = GridLayout::new(1).layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(120.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // One column filling the full content width; items stack vertically.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 120.0);
        assert_eq!(b[1].origin(), Point::new(0.0, 20.0));
        assert_eq!(GridLayout::new(1).columns(), 1);
        assert_eq!(GridLayout::new(0).columns(), 1);
    }

    // ---- Regressions carried over from the uniform-grid implementation. ----

    #[test]
    fn three_children_fill_two_columns_over_two_rows() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(100.0, 20.0),
            Size::new(100.0, 30.0),
            Size::new(100.0, 10.0),
        ]);
        let out = GridLayout::new(2).layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size(), Size::new(100.0, 20.0));
        assert_eq!(b[1].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].size(), Size::new(100.0, 30.0));
        assert_eq!(b[2].origin(), Point::new(0.0, 30.0));
        assert_eq!(b[2].size(), Size::new(100.0, 10.0));
        assert_eq!(out.size(), Size::new(200.0, 40.0));
    }

    #[test]
    fn gaps_offset_columns_and_rows() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(0.0, 20.0),
            Size::new(0.0, 20.0),
            Size::new(0.0, 10.0),
        ]);
        let g = GridLayout::new(2).with_column_gap(10.0).with_row_gap(5.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(210.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[1].origin(), Point::new(110.0, 0.0));
        assert_eq!(b[2].origin(), Point::new(0.0, 25.0));
        assert_eq!(b[0].size(), Size::new(100.0, 20.0));
    }

    #[test]
    fn zero_columns_clamps_to_one() {
        let g = GridLayout::new(0);
        assert_eq!(g.columns(), 1);
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 20.0), Size::new(50.0, 30.0)]);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(50.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(0.0, 20.0));
    }

    #[test]
    fn padding_offsets_grid_and_grows_box() {
        let style = LayoutStyle {
            padding: Insets::uniform(5.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(90.0, 40.0)]);
        let out = GridLayout::new(1).layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(5.0, 5.0));
        assert_eq!(out.size(), Size::new(100.0, 50.0));
    }

    // Explicit container width bounds the fr tracks: `width:300px` with `1fr 1fr`
    // under a 500px available extent → two 150px columns, not 250.
    #[test]
    fn explicit_width_bounds_fr_columns() {
        let style = LayoutStyle {
            width: Dimension::Px(300.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // fr tracks resolve against the explicit 300, not the 500 available.
        assert_eq!(b[0].size().width, 150.0);
        assert_eq!(b[1].origin().x, 150.0);
        assert_eq!(b[1].size().width, 150.0);
        // The container is its authored 300px wide.
        assert_eq!(out.size().width, 300.0);
    }

    // Explicit container height bounds the fr rows: `height:200px` with two `1fr`
    // rows → each row 100 tall, and the container is 200 tall.
    #[test]
    fn explicit_height_bounds_fr_rows() {
        let style = LayoutStyle {
            height: Dimension::Px(200.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0)],
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 500.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Rows share the explicit 200 → 100 each; second child in the second row.
        assert_eq!(b[1].origin().y, 100.0);
        assert_eq!(out.size().height, 200.0);
    }

    // Regression: with no explicit width, tracks still fill the available extent.
    #[test]
    fn auto_width_fills_available_columns() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Auto width → fr tracks fill the 500 available → two 250px columns.
        assert_eq!(b[0].size().width, 250.0);
        assert_eq!(b[1].size().width, 250.0);
        assert_eq!(out.size().width, 500.0);
    }

    // ---- Line-based placement + spanning. ----

    fn three_col_100() -> GridLayout {
        grid(
            vec![
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(100.0),
            ],
            vec![],
            0.0,
            0.0,
        )
    }

    // (a) `grid-column: 1 / 3` spans cols 0-1: width 200 at x=0.
    #[test]
    fn column_line_to_line_spans_two_tracks() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Line(1), GridLine::Line(3)),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 200.0);
    }

    // (b) `grid-column: span 2` auto-placed occupies cols 0-1 (width 200 at x=0).
    #[test]
    fn column_span_auto_placed_occupies_two_cells() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Auto, GridLine::Span(2)),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 200.0);
    }

    // (c) an explicit item at column line 2 + an auto item: the auto item flows
    // around the occupied cell, landing in column 0.
    #[test]
    fn auto_item_flows_around_explicit_cell() {
        let style = LayoutStyle::default();
        let children = [
            placed(1, GridPlacement::column(GridLine::Line(2), GridLine::Auto)),
            child(2),
        ];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 20.0), Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Explicit item on column 1 (x=100), auto item flows to column 0 (x=0).
        assert_eq!(b[0].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(0.0, 0.0));
    }

    // (d) `grid-row: 2` puts an item on row index 1. Row 0 is a fixed 50px track
    // so the row boundary is a concrete coordinate.
    #[test]
    fn explicit_row_places_on_second_row() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::row(GridLine::Line(2), GridLine::Auto),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 30.0)]);
        let g = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(50.0)], // row 0 fixed; row 1 implicit auto
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Row 0 (50px) is empty; the item sits on row 1 at y = 50.
        assert_eq!(b[0].origin(), Point::new(0.0, 50.0));
    }

    // (e) a span that does not fit the remaining row wraps to the next row.
    #[test]
    fn span_that_does_not_fit_wraps_to_next_row() {
        let style = LayoutStyle::default();
        let children = [
            child(1), // auto → (col0,row0)
            placed(2, GridPlacement::column(GridLine::Auto, GridLine::Span(2))),
        ];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 15.0)]);
        let g = grid(
            vec![TrackSize::Fixed(100.0), TrackSize::Fixed(100.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // child0 fills (0,0); the span-2 item cannot fit beside it, so it wraps
        // to row 1 (y = row0 height = 10), spanning the full 200px width.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(0.0, 10.0));
        assert_eq!(b[1].size().width, 200.0);
    }

    // (f) regression: all-`Auto` placement reproduces the row-major result.
    #[test]
    fn all_auto_placement_is_row_major() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3), child(4), child(5)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 5]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[3].origin(), Point::new(100.0, 10.0)); // col1,row1
        assert_eq!(b[4].origin(), Point::new(0.0, 20.0)); // col0,row2
        assert_eq!(out.size(), Size::new(200.0, 30.0));
    }

    // (g) a column gap is applied across a span's interior.
    #[test]
    fn column_gap_applies_across_span_interior() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Line(1), GridLine::Line(3)),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let g = grid(
            vec![
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(100.0),
            ],
            vec![],
            10.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(320.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Spans cols 0-1: 100 + 100 + one interior 10px gap = 210, at x=0.
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 210.0);
    }

    // (neg-a) `grid-column: 1 / -1` spans every column: width 300 at x=0 (3×100).
    #[test]
    fn negative_end_line_spans_all_columns() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Line(1), GridLine::Line(-1)),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 300.0);
    }

    // (neg-b) `grid-column: -1` lands on the last column: x=200, width 100.
    #[test]
    fn negative_start_line_places_in_last_column() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Line(-1), GridLine::Auto),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 200.0);
        assert_eq!(b[0].size().width, 100.0);
    }

    // (neg-c) a negative row line: `grid-row: -1` places on the last explicit row.
    #[test]
    fn negative_row_line_places_on_last_row() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::row(GridLine::Line(-1), GridLine::Auto),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 30.0)]);
        // Two explicit 50px rows; -1 is the last row (index 1) → y = 50.
        let g = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().y, 50.0);
    }

    // ---- `repeat(auto-fill | auto-fit, …)` computed from the container size. ----

    /// A columns-only grid whose single axis is one `auto-fill`/`auto-fit` repeat.
    fn auto_repeat_grid(mode: AutoRepeatMode, pattern: Vec<TrackSize>, col_gap: f32) -> GridLayout {
        let columns = TrackList::new(Vec::new(), Some(AutoRepeat::new(mode, pattern)), Vec::new());
        GridLayout::with_template(
            GridTemplate::from_track_lists(columns, TrackList::default()),
            col_gap,
            0.0,
        )
    }

    fn width_style(px: f32) -> LayoutStyle {
        LayoutStyle {
            width: Dimension::Px(px),
            ..LayoutStyle::default()
        }
    }

    // (a) `repeat(auto-fill, 100px)` on a 350px container, no gap: 3 columns fit
    // (3*100 = 300 <= 350; a 4th overflows), placed at x = 0 / 100 / 200.
    #[test]
    fn auto_fill_fits_three_columns() {
        let style = width_style(350.0);
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 3]);
        let g = auto_repeat_grid(AutoRepeatMode::AutoFill, vec![TrackSize::Fixed(100.0)], 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 100.0);
        assert_eq!(b[1].origin().x, 100.0);
        assert_eq!(b[2].origin().x, 200.0);
    }

    // (a') the same count with a 10px gap: floor((350 + 10) / (100 + 10)) = 3,
    // columns at x = 0 / 110 / 220.
    #[test]
    fn auto_fill_count_accounts_for_gap() {
        let style = width_style(350.0);
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 3]);
        let g = auto_repeat_grid(
            AutoRepeatMode::AutoFill,
            vec![TrackSize::Fixed(100.0)],
            10.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[1].origin().x, 110.0);
        assert_eq!(b[2].origin().x, 220.0);
    }

    // (b) `auto-fit`: with fewer items than tracks, an empty interior track
    // collapses to 0 and its gaps collapse — an item on the last line packs left.
    // Container 350px, `repeat(auto-fit, 100px)` -> 3 tracks. Child A auto (col0),
    // child B on grid-column line 3 (col2); col1 is empty and collapses, so B sits
    // at x = 100 (not 200).
    #[test]
    fn auto_fit_collapses_empty_interior_track() {
        let style = width_style(350.0);
        let children = [
            child(1),
            placed(2, GridPlacement::column(GridLine::Line(3), GridLine::Auto)),
        ];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = auto_repeat_grid(AutoRepeatMode::AutoFit, vec![TrackSize::Fixed(100.0)], 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0); // A in col0
        assert_eq!(b[1].origin().x, 100.0); // B: col1 collapsed, so col2 starts at 100
    }

    // (b') `auto-fill` keeps the same empty interior track (no collapse): B on the
    // last line sits at x = 200 (col1 occupies its 100px).
    #[test]
    fn auto_fill_keeps_empty_interior_track() {
        let style = width_style(350.0);
        let children = [
            child(1),
            placed(2, GridPlacement::column(GridLine::Line(3), GridLine::Auto)),
        ];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = auto_repeat_grid(AutoRepeatMode::AutoFill, vec![TrackSize::Fixed(100.0)], 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[1].origin().x, 200.0); // col1 kept its 100px
    }

    // (c) a leading fixed 50px track ahead of `repeat(auto-fill, 100px)` on a 350px
    // container: 3 more 100px tracks fit the remaining 300px, so tracks are
    // [50, 100, 100, 100] and 4 children land at x = 0 / 50 / 150 / 250.
    #[test]
    fn auto_fill_count_after_leading_track() {
        let style = width_style(350.0);
        let children = [child(1), child(2), child(3), child(4)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 4]);
        let columns = TrackList::new(
            vec![TrackSize::Fixed(50.0)],
            Some(AutoRepeat::new(
                AutoRepeatMode::AutoFill,
                vec![TrackSize::Fixed(100.0)],
            )),
            Vec::new(),
        );
        let g = GridLayout::with_template(
            GridTemplate::from_track_lists(columns, TrackList::default()),
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 50.0);
        assert_eq!(b[1].origin().x, 50.0);
        assert_eq!(b[2].origin().x, 150.0);
        assert_eq!(b[3].origin().x, 250.0);
    }

    #[test]
    fn empty_grid_sizes_to_padding() {
        let style = LayoutStyle {
            padding: Insets::uniform(4.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::default();
        let out = GridLayout::new(3).layout(
            &style,
            &[],
            BoxConstraints::loose(Size::new(60.0, 60.0)),
            &mut measurer,
        );
        assert!(out.is_empty());
        assert_eq!(out.size(), Size::new(8.0, 8.0));
    }

    // ---- `grid-auto-flow`: dense packing + column flow. ----

    /// Three span-then-1x1 items in a 3-column grid leave a hole at `(row0, col2)`:
    /// two span-2 items land on row 0 and row 1, then a 1x1 item follows. `dense`
    /// backfills the hole (row 0); the default sparse flow does not (row 1). The
    /// two flows must place that item at *different* coordinates.
    fn hole_children() -> [LayoutChild; 3] {
        [
            placed(1, GridPlacement::column(GridLine::Auto, GridLine::Span(2))),
            placed(2, GridPlacement::column(GridLine::Auto, GridLine::Span(2))),
            child(3),
        ]
    }

    #[test]
    fn dense_backfills_earlier_hole_but_sparse_does_not() {
        let cols = || {
            grid(
                vec![
                    TrackSize::Fixed(100.0),
                    TrackSize::Fixed(100.0),
                    TrackSize::Fixed(100.0),
                ],
                vec![],
                0.0,
                0.0,
            )
        };
        let sizes = || {
            FixedChildMeasurer::new(vec![
                Size::new(0.0, 10.0),
                Size::new(0.0, 10.0),
                Size::new(0.0, 10.0),
            ])
        };
        let constraints = BoxConstraints::loose(Size::new(300.0, 1000.0));

        // Sparse (default `Row`): the 1x1 item does NOT backfill — it follows the
        // forward cursor onto row 1, col 2.
        let sparse = LayoutStyle::default();
        let mut m1 = sizes();
        let out_sparse = cols().layout(&sparse, &hole_children(), constraints, &mut m1);
        let bs = out_sparse.child_boxes();
        assert_eq!(bs[2].origin(), Point::new(200.0, 10.0));

        // Dense (`RowDense`): the 1x1 item backfills the hole at row 0, col 2.
        let dense = LayoutStyle::default().with_grid_auto_flow(GridAutoFlow::RowDense);
        let mut m2 = sizes();
        let out_dense = cols().layout(&dense, &hole_children(), constraints, &mut m2);
        let bd = out_dense.child_boxes();
        assert_eq!(bd[2].origin(), Point::new(200.0, 0.0));

        // The two flows differ (the whole point of `dense`).
        assert_ne!(bs[2].origin(), bd[2].origin());
    }

    /// `column` flow fills *down* the rows of column 0, then across to column 1:
    /// with two fixed 20px rows, four 1x1 items land at
    /// `(c0,r0) (c0,r1) (c1,r0) (c1,r1)`.
    #[test]
    fn column_flow_places_down_then_across() {
        let style = LayoutStyle::default().with_grid_auto_flow(GridAutoFlow::Column);
        let children = [child(1), child(2), child(3), child(4)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0); 4]);
        let g = grid(
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
            vec![TrackSize::Fixed(20.0), TrackSize::Fixed(20.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0)); // col0, row0
        assert_eq!(b[1].origin(), Point::new(0.0, 20.0)); // col0, row1 (down)
        assert_eq!(b[2].origin(), Point::new(50.0, 0.0)); // col1, row0 (across)
        assert_eq!(b[3].origin(), Point::new(50.0, 20.0)); // col1, row1
    }

    /// Regression: the default (`Row`, sparse) flow is unchanged — the same
    /// row-major result as before `grid-auto-flow` existed.
    #[test]
    fn default_row_sparse_is_unchanged() {
        let style = LayoutStyle::default();
        assert_eq!(style.grid_auto_flow(), GridAutoFlow::Row);
        let children = [child(1), child(2), child(3), child(4), child(5)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 5]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[3].origin(), Point::new(100.0, 10.0)); // col1,row1
        assert_eq!(b[4].origin(), Point::new(0.0, 20.0)); // col0,row2
        assert_eq!(out.size(), Size::new(200.0, 30.0));
    }

    // ---- Grid item alignment within a cell (`justify-self` / `align-self`). ----

    use crate::model::{AlignItems, AlignSelf};

    /// A single `40×30` child in one `100×100` fixed cell, aligned per-item.
    fn aligned_in_cell(justify: AlignSelf, align: AlignSelf) -> (f32, f32, f32, f32) {
        aligned_in_cell_with(LayoutStyle::default(), justify, align)
    }

    /// As [`aligned_in_cell`], but the container `style` supplies the
    /// `justify-items` / `align-items` defaults.
    fn aligned_in_cell_with(
        style: LayoutStyle,
        justify: AlignSelf,
        align: AlignSelf,
    ) -> (f32, f32, f32, f32) {
        let child = LayoutChild::new(
            NodeId::from_raw(1),
            LayoutStyle::default()
                .with_justify_self(justify)
                .with_align_self(align),
        );
        // The measurer reports the item's natural size (40×30) regardless of
        // constraints; the fixed 100×100 tracks are the cell.
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(40.0, 30.0)]);
        let g = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(100.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &[child],
            BoxConstraints::loose(Size::new(100.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes()[0];
        (b.origin().x, b.origin().y, b.size().width, b.size().height)
    }

    // `justify-self:start` + `align-self:start` → the item sits at the cell
    // origin at its natural 40×30 size (no stretch to the 100×100 cell).
    #[test]
    fn self_start_places_at_cell_origin_natural_size() {
        assert_eq!(
            aligned_in_cell(AlignSelf::Start, AlignSelf::Start),
            (0.0, 0.0, 40.0, 30.0)
        );
    }

    // `center` on both axes → offset ((100-40)/2, (100-30)/2) = (30, 35).
    #[test]
    fn self_center_offsets_by_half_the_slack() {
        assert_eq!(
            aligned_in_cell(AlignSelf::Center, AlignSelf::Center),
            (30.0, 35.0, 40.0, 30.0)
        );
    }

    // `end` on both axes → offset (100-40, 100-30) = (60, 70).
    #[test]
    fn self_end_places_at_cell_end() {
        assert_eq!(
            aligned_in_cell(AlignSelf::End, AlignSelf::End),
            (60.0, 70.0, 40.0, 30.0)
        );
    }

    // `stretch` (the default) → the item fills the 100×100 cell, byte-identical
    // to the historical grid behaviour.
    #[test]
    fn self_stretch_fills_the_cell() {
        assert_eq!(
            aligned_in_cell(AlignSelf::Stretch, AlignSelf::Stretch),
            (0.0, 0.0, 100.0, 100.0)
        );
        // The `auto`/`auto` default (no per-item override, container all-stretch)
        // is likewise a full-cell fill.
        assert_eq!(
            aligned_in_cell(AlignSelf::Auto, AlignSelf::Auto),
            (0.0, 0.0, 100.0, 100.0)
        );
    }

    // The two axes are independent: `justify-self:end` (inline/x) with
    // `align-self:stretch` (block/y) → x at the cell end, y filling the cell.
    #[test]
    fn self_axes_are_independent() {
        assert_eq!(
            aligned_in_cell(AlignSelf::End, AlignSelf::Stretch),
            (60.0, 0.0, 40.0, 100.0)
        );
    }

    // A container `justify-items:center` default applies to an item whose
    // `justify-self` is `auto`; the block axis stays at the container's
    // `align-items` (stretch) default.
    #[test]
    fn container_justify_items_applies_when_self_is_auto() {
        let container = LayoutStyle::default().with_justify_items(AlignItems::Center);
        assert_eq!(
            aligned_in_cell_with(container, AlignSelf::Auto, AlignSelf::Auto),
            (30.0, 0.0, 40.0, 100.0)
        );
    }

    // A per-item `justify-self` overrides the container `justify-items` default.
    #[test]
    fn self_overrides_container_items_default() {
        let container = LayoutStyle::default().with_justify_items(AlignItems::Center);
        // `justify-self:start` beats the container `center`.
        assert_eq!(
            aligned_in_cell_with(container, AlignSelf::Start, AlignSelf::Auto),
            (0.0, 0.0, 40.0, 100.0)
        );
    }

    // ---- content distribution: `justify-content` (columns) / `align-content`
    // (rows) distribute the leftover free space when the tracks are smaller than
    // the container content extent. Two 50px tracks in a 200px container leave a
    // 100px leftover.

    /// Lays out two `50px` columns in a 200px-wide container with `justify` and
    /// returns the two children's `x` offsets.
    fn distribute_cols(justify: JustifyContent) -> (f32, f32) {
        let style = LayoutStyle {
            width: Dimension::Px(200.0),
            justify_content: justify,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 2]);
        let g = grid(
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        (b[0].origin().x, b[1].origin().x)
    }

    /// Lays out two `50px` rows (single column) in a 200px-tall container with
    /// `align` and returns the two children's `y` offsets.
    fn distribute_rows(align: JustifyContent) -> (f32, f32) {
        let style = LayoutStyle {
            height: Dimension::Px(200.0),
            align_content: align,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 2]);
        let g = grid(
            vec![TrackSize::Fr(1.0)],
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 200.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        (b[0].origin().y, b[1].origin().y)
    }

    #[test]
    fn justify_content_start_is_unshifted_regression() {
        // The default packs the columns at the start — byte-identical to before.
        assert_eq!(distribute_cols(JustifyContent::FlexStart), (0.0, 50.0));
    }

    #[test]
    fn justify_content_end_shifts_by_full_leftover() {
        assert_eq!(distribute_cols(JustifyContent::FlexEnd), (100.0, 150.0));
    }

    #[test]
    fn justify_content_center_shifts_by_half_leftover() {
        assert_eq!(distribute_cols(JustifyContent::Center), (50.0, 100.0));
    }

    #[test]
    fn justify_content_space_between_puts_leftover_between_tracks() {
        // n = 2 → the whole 100px leftover becomes the gap between the tracks.
        assert_eq!(distribute_cols(JustifyContent::SpaceBetween), (0.0, 150.0));
    }

    #[test]
    fn justify_content_space_around_half_gaps_at_ends() {
        // leftover/(2n) = 25 at the start, leftover/n = 50 between the tracks.
        assert_eq!(distribute_cols(JustifyContent::SpaceAround), (25.0, 125.0));
    }

    #[test]
    fn justify_content_space_evenly_equal_gaps_including_ends() {
        // leftover/(n+1) = 100/3 ≈ 33.33 at each of the three gaps.
        let (x0, x1) = distribute_cols(JustifyContent::SpaceEvenly);
        assert!((x0 - 100.0 / 3.0).abs() < 1e-3, "x0 = {x0}");
        assert!(
            (x1 - (100.0 / 3.0 + 50.0 + 100.0 / 3.0)).abs() < 1e-3,
            "x1 = {x1}"
        );
    }

    #[test]
    fn align_content_distributes_rows_on_the_block_axis() {
        // The same math on the row (block) axis via `align-content`.
        assert_eq!(distribute_rows(JustifyContent::FlexStart), (0.0, 50.0));
        assert_eq!(distribute_rows(JustifyContent::FlexEnd), (100.0, 150.0));
        assert_eq!(distribute_rows(JustifyContent::Center), (50.0, 100.0));
        assert_eq!(distribute_rows(JustifyContent::SpaceBetween), (0.0, 150.0));
        assert_eq!(distribute_rows(JustifyContent::SpaceAround), (25.0, 125.0));
        let (y0, y1) = distribute_rows(JustifyContent::SpaceEvenly);
        assert!((y0 - 100.0 / 3.0).abs() < 1e-3, "y0 = {y0}");
        assert!(
            (y1 - (100.0 / 3.0 + 50.0 + 100.0 / 3.0)).abs() < 1e-3,
            "y1 = {y1}"
        );
    }

    #[test]
    fn content_distribution_axes_are_independent() {
        // `justify-content: end` on columns must not disturb the row (block) axis,
        // which keeps its `align-content: start` default.
        let style = LayoutStyle {
            width: Dimension::Px(200.0),
            height: Dimension::Px(200.0),
            justify_content: JustifyContent::FlexEnd,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 2]);
        // 50px 50px columns, one 50px row → both items in row 0.
        let g = grid(
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
            vec![TrackSize::Fixed(50.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 200.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Columns shifted to the end (x = 100 / 150); rows unshifted (y = 0).
        assert_eq!(b[0].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(150.0, 0.0));
    }

    // ---- Canonical §11 track-sizing pipeline: the four divergence fixes plus a
    // regression that the simple even-split path is unchanged. ----

    // (fr-restart, §11.7) `minmax(200px,1fr) 1fr` in a 300px container. Track 0's
    // fr share (150) is below its 200px floor, so it pins at 200 and its UNUSED
    // fr share flows to track 1, which takes ALL the remaining 100 (not a 50/50
    // split). The old single-pass fr gave track 1 only 50.
    #[test]
    fn fr_restart_gives_unused_share_to_other_flex_track() {
        let style = LayoutStyle {
            width: Dimension::Px(300.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![
                TrackSize::minmax(TrackSize::Fixed(200.0), TrackSize::Fr(1.0)),
                TrackSize::Fr(1.0),
            ],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 200.0);
        assert_eq!(b[1].origin().x, 200.0);
        assert_eq!(b[1].size().width, 100.0);
    }

    // (span distribution, §11.5) `auto auto` with a colspan-2 item 200px wide:
    // its content is shared ACROSS both auto tracks (100 each), not attributed to
    // the first spanned track alone. A following 1×1 item in column 0 fills that
    // 100px cell — proving column 0 is 100 (the old code sized it 200).
    #[test]
    fn spanning_item_shares_content_across_auto_tracks() {
        let style = LayoutStyle::default();
        let children = [
            placed(1, GridPlacement::column(GridLine::Auto, GridLine::Span(2))),
            child(2),
        ];
        // Item 0 is 200px wide (crosses both auto cols); item 1 is zero-width.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(200.0, 20.0), Size::new(0.0, 10.0)]);
        let g = grid(vec![TrackSize::Auto, TrackSize::Auto], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(1000.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // The span-2 item covers both 100px tracks → 200 wide at x = 0.
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 200.0);
        // The 1×1 item lands in column 0 (row 1) and fills its cell: width 100
        // confirms the 200px content was split 100/100, not dumped on column 0.
        assert_eq!(b[1].origin().x, 0.0);
        assert_eq!(b[1].size().width, 100.0);
    }

    // (maximise, §11.6) `minmax(100px,300px)` alone in a 500px container grows to
    // its 300px growth limit (free space is distributed up to the limit). The old
    // model left it at its 100px floor.
    #[test]
    fn maximise_grows_minmax_to_its_growth_limit() {
        let style = LayoutStyle {
            width: Dimension::Px(500.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        let g = grid(
            vec![TrackSize::minmax(
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(300.0),
            )],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 300.0);
    }

    // (stretch, §11.8) `auto auto` in a definite 400px container with small
    // content: with the default (`normal`/`stretch`) content alignment the two
    // auto tracks each stretch to fill 200px. The old model packed them at the
    // start at their content size.
    #[test]
    fn stretch_expands_auto_tracks_to_fill_definite_container() {
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(30.0, 10.0), Size::new(30.0, 10.0)]);
        let g = grid(vec![TrackSize::Auto, TrackSize::Auto], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 200.0);
        assert_eq!(b[1].origin().x, 200.0);
        assert_eq!(b[1].size().width, 200.0);
    }

    // (regression) a plain `1fr 1fr` even split is unchanged by the new pipeline:
    // two equal halves of the 300px content width.
    #[test]
    fn even_fr_split_is_unchanged_by_pipeline() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 150.0);
        assert_eq!(b[1].origin().x, 150.0);
        assert_eq!(b[1].size().width, 150.0);
    }

    // ---- min-content / max-content tracks (audit Tier-1 §4-D) --------------

    /// An item that answers the offer like a paragraph: it fills the inline space
    /// it is given, never below `min` (its longest unbreakable word) and never
    /// above `max` (its whole content on one line). [`FixedChildMeasurer`] cannot
    /// catch a probe that is not probing — it returns the same size whatever it is
    /// offered — so these tests need a measurer with an opinion.
    struct Paragraph {
        min: f32,
        max: f32,
    }

    impl crate::algorithm::ChildMeasurer for Paragraph {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            Size::new(constraints.max.width.clamp(self.min, self.max), 10.0)
        }
    }

    /// The single column's width under `track`, in a `line`-wide grid holding one
    /// paragraph item (min-content 30, max-content 200).
    fn one_track_width(track: TrackSize, line: f32) -> f32 {
        let out = grid(vec![track], vec![], 0.0, 0.0).layout(
            &LayoutStyle::default(),
            &[child(1)],
            BoxConstraints::loose(Size::new(line, 1000.0)),
            &mut Paragraph {
                min: 30.0,
                max: 200.0,
            },
        );
        out.child_boxes()[0].size().width
    }

    #[test]
    fn a_min_content_column_is_the_widest_unbreakable_run() {
        // The probe offers the item no inline space, so it reports the 30 it cannot
        // break below — NOT the 100 the container has to give.
        assert_eq!(one_track_width(TrackSize::MinContent, 100.0), 30.0);
    }

    #[test]
    fn a_max_content_column_is_the_content_on_one_line() {
        // Unbounded offer → the item's whole content on one line (200), which
        // OVERFLOWS the 100px container: `max-content` is a question about the
        // content, and the container's answer is not it.
        assert_eq!(one_track_width(TrackSize::MaxContent, 100.0), 200.0);
    }

    #[test]
    fn min_content_max_content_and_auto_columns_all_disagree() {
        // If any two of these agreed, the probes would not be probing. The 100px
        // container is the case that separates them: `auto` takes the space it has
        // (the documented single-measure approximation), while the two keywords
        // each report a different property of the content and ignore the container.
        let min = one_track_width(TrackSize::MinContent, 100.0);
        let max = one_track_width(TrackSize::MaxContent, 100.0);
        let auto = one_track_width(TrackSize::Auto, 100.0);
        assert_eq!((min, max, auto), (30.0, 200.0, 100.0));
    }

    #[test]
    fn a_min_content_column_beside_an_fr_gives_the_fr_the_rest() {
        // The pair that makes the keyword worth having: a content-hugging label
        // column beside a flexible one. The label takes its 30, the `1fr` takes
        // the other 270.
        let out = grid(
            vec![TrackSize::MinContent, TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        )
        .layout(
            &LayoutStyle::default(),
            &[child(1), child(2)],
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut Paragraph {
                min: 30.0,
                max: 200.0,
            },
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 30.0);
        assert_eq!(b[1].origin().x, 30.0);
        assert_eq!(b[1].size().width, 270.0);
    }

    #[test]
    fn a_minmax_min_content_floor_is_the_min_content_size_not_the_auto_one() {
        // `minmax(min-content, 1fr) 100px` in a 200px grid. The floor is the item's
        // MIN-content size (30), which the `1fr` then covers — so the flexible
        // track gets its 100 and the grid fits. Were the floor `auto`'s single
        // loose measure (200, the space the container has), the base would exceed
        // the `fr` share and the grid would overflow to 200 + 100. The `minmax`
        // recursion is what has to carry the keyword down to the min bound.
        let out = grid(
            vec![
                TrackSize::minmax(TrackSize::MinContent, TrackSize::Fr(1.0)),
                TrackSize::Fixed(100.0),
            ],
            vec![],
            0.0,
            0.0,
        )
        .layout(
            &LayoutStyle::default(),
            &[child(1), child(2)],
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut Paragraph {
                min: 30.0,
                max: 200.0,
            },
        );
        let b = out.child_boxes();
        assert_eq!(
            b[0].size().width,
            100.0,
            "1fr of the 100 left after the 100px"
        );
        assert_eq!(b[1].origin().x, 100.0, "the fixed track is not pushed out");
    }

    #[test]
    fn a_content_keyword_row_takes_the_items_height() {
        // The block axis: with its width fixed to the column, the item's three
        // intrinsic heights coincide, so a `min-content` row is its measured
        // height — 10 here — and not the 1000 the container offers.
        let out = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::MinContent],
            0.0,
            0.0,
        )
        .layout(
            &LayoutStyle::default(),
            &[child(1)],
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut Paragraph {
                min: 30.0,
                max: 200.0,
            },
        );
        assert_eq!(out.child_boxes()[0].size().height, 10.0);
    }
}
