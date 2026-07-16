//! `FlexLayout`: single-line flex layout for the `Flex` formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{AlignItems, Dimension, JustifyContent, LayoutBox, LayoutStyle, Margin};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput, clamp_content};

/// The item's main-axis min / max dimensions for the container's flow direction:
/// `width` bounds along a row, `height` bounds along a column.
fn main_bounds(style: &LayoutStyle, is_row: bool) -> (Dimension, Dimension) {
    if is_row {
        (style.min_width, style.max_width)
    } else {
        (style.min_height, style.max_height)
    }
}

/// Clamps a main-axis size into a pre-resolved `[min_floor, max_val]` range
/// (spec §4.6).
///
/// `min_floor` and `max_val` are the item's main-axis bounds already resolved to
/// pixels against the container main extent (with `max_val` set to
/// [`f32::INFINITY`] when the item has no `max`). It applies `max` first, then the
/// `min` floor — so `min` wins on conflict, matching CSS `min-width` overriding
/// `max-width` — and never returns a negative size. `min_floor` carries the §4.5
/// content-based automatic minimum on the `auto` path and the explicit `min`
/// otherwise, so an `auto` min with no content floor (`min_floor == 0`) and no
/// `max` leaves the value unchanged.
fn clamp_floor(value: f32, min_floor: f32, max_val: f32) -> f32 {
    let mut clamped = value;
    if max_val.is_finite() {
        clamped = clamped.min(max_val);
    }
    clamped.max(min_floor).max(0.0)
}

/// The item's **content-based minimum main size** — its max-content size along the
/// main axis, measured by laying the item out with an *unbounded* main axis and
/// the container's cross extent (CSS Flexbox Level 1 §4.5 / §9.2,
/// <https://drafts.csswg.org/css-flexbox-1/#min-size-auto>).
///
/// For content whose main-axis extent does not reflow (a column's stacked block
/// height, an atomic / single-word inline) this equals the min-content size, so it
/// is the exact automatic minimum. For reflowable content (a multi-word inline in
/// a row) it is an upper bound of the true min-content — a deliberately safe
/// over-approximation, since the engine cannot measure a text leaf's min-content
/// through the constraint-clamping measure port.
///
/// The result may be [`f32::INFINITY`] for a fill-available container whose
/// max-content is unbounded; the caller resolves that to a finite floor.
fn content_max_main(
    measurer: &mut dyn ChildMeasurer,
    index: usize,
    is_row: bool,
    container_cross: f32,
) -> f32 {
    probe_main(measurer, index, is_row, f32::INFINITY, container_cross)
}

/// The main-axis size item `index` reports when offered `main_offer` of main-axis
/// space and the container's cross extent.
///
/// The one measuring primitive behind every intrinsic answer flex needs: `0` is
/// the `min-content` offer (nothing wraps that can be broken), [`f32::INFINITY`]
/// the `max-content` one (nothing wraps that does not have to). The offer is
/// **loose** — `min: ZERO` — so an item that cannot fit reports the size it needs
/// rather than the space it was given, which is the whole point of the probe (see
/// [`BoxConstraints::constrain_allowing_overflow`]).
///
/// The answer is a **border**-box extent, like every measured size, so no
/// `box-sizing` mapping applies to it.
fn probe_main(
    measurer: &mut dyn ChildMeasurer,
    index: usize,
    is_row: bool,
    main_offer: f32,
    container_cross: f32,
) -> f32 {
    let max = if is_row {
        Size::new(main_offer, container_cross)
    } else {
        Size::new(container_cross, main_offer)
    };
    let measured = measurer.measure_child(
        index,
        BoxConstraints {
            min: Size::ZERO,
            max,
        },
    );
    if is_row {
        measured.width
    } else {
        measured.height
    }
}

/// The item's main-axis size for an **intrinsic** `flex-basis` / `width` /
/// `height` keyword, or [`None`] when `dim` is not one (CSS Sizing 3 §4.1).
///
/// The flex twin of [`intrinsic_inline_size`](super::intrinsic_inline_size), and
/// the same three answers — but flex asks them of ONE item rather than of a run of
/// stacked children, and asks along whichever axis is the main one, so the probe
/// is a measure of that item instead of a pass over children:
///
/// * `min-content` — the item offered no main-axis space;
/// * `max-content` — the item offered unbounded main-axis space;
/// * `fit-content` — `min(max-content, max(min-content, stretch))`, where
///   `stretch` is the container's main extent (CSS 2.1 §10.3.5's shrink-to-fit).
///
/// Without this the keywords fell through [`Dimension::resolve`]'s [`None`] into
/// the measured-natural-size fallback — i.e. `flex-basis: max-content` silently
/// meant `flex-basis: auto`.
///
/// # Cost
///
/// One extra measure per intrinsic item (`fit-content` two), and only for an item
/// that authored a keyword: [`Dimension::resolve`]-shaped values return here
/// immediately without measuring.
fn intrinsic_main(
    measurer: &mut dyn ChildMeasurer,
    index: usize,
    dim: Dimension,
    is_row: bool,
    container_main: f32,
    container_cross: f32,
) -> Option<f32> {
    match dim {
        Dimension::MinContent => Some(probe_main(measurer, index, is_row, 0.0, container_cross)),
        Dimension::MaxContent => Some(content_max_main(measurer, index, is_row, container_cross)),
        Dimension::FitContent => {
            let min = probe_main(measurer, index, is_row, 0.0, container_cross);
            let max = content_max_main(measurer, index, is_row, container_cross);
            Some(max.min(container_main.max(min)))
        }
        Dimension::Auto | Dimension::Px(_) | Dimension::Percent(_) => None,
    }
}

/// The item's total border + padding along the given axis (`horizontal` for the
/// row main / column cross axis, else vertical).
///
/// Used to expand an explicit `content-box` size into the border-box extent the
/// item actually occupies, so a `width: 0` / `height: 0` item with a border keeps
/// its border widths instead of collapsing to a zero-extent box.
fn axis_bp(style: &LayoutStyle, horizontal: bool) -> f32 {
    if horizontal {
        style.border.left + style.border.right + style.padding.left + style.padding.right
    } else {
        style.border.top + style.border.bottom + style.padding.top + style.padding.bottom
    }
}

/// The item's **cross**-axis min / max dimensions for the container's flow
/// direction: `height` bounds across a row, `width` bounds across a column — the
/// mirror of [`main_bounds`].
fn cross_bounds(style: &LayoutStyle, is_row: bool) -> (Dimension, Dimension) {
    if is_row {
        (style.min_height, style.max_height)
    } else {
        (style.min_width, style.max_width)
    }
}

/// Clamps an item's cross-axis **border** extent into its authored cross-axis
/// min/max (CSS Flexbox Level 1 §9.4 step 11 "clamp by min/max cross size").
///
/// The main axis has always applied its bounds (via [`clamp_floor`]); the cross
/// axis dropped them entirely, so an item's `max-height` in a row — or a stretch
/// that should stop at `max-height` — was ignored. Bounds are resolved against
/// the container cross extent and mapped through the item's `box_sizing` into the
/// border extent this value is expressed in, matching how the explicit cross size
/// is resolved in [`cross_item_size`]. Max applies before min, so min wins on
/// conflict (per CSS).
///
/// An **intrinsic** cross bound (`max-height: max-content` on a row item) is
/// still dropped: `resolve` answers [`None`] for a keyword, so it imposes no
/// limit here. Only the MAIN axis measures its keywords (see [`intrinsic_main`]).
fn clamp_cross(value: f32, style: &LayoutStyle, is_row: bool, container_cross: f32) -> f32 {
    let (min, max) = cross_bounds(style, is_row);
    let bp = axis_bp(style, !is_row);
    let mut clamped = value;
    if let Some(px) = max.resolve(container_cross) {
        clamped = clamped.min(style.box_sizing.border_extent(px, bp));
    }
    if let Some(px) = min.resolve(container_cross) {
        clamped = clamped.max(style.box_sizing.border_extent(px, bp));
    }
    clamped.max(0.0)
}

/// The **effective** cross-axis alignment for one item: its own `align-self`,
/// falling back to the container's `align-items` when `auto` (the initial value).
///
/// Flex previously read the container's `align-items` for every item, so a
/// per-item `align-self` was silently dropped — while grid honoured it. This is
/// the same `AlignSelf::resolve` grid uses, so both contexts now agree.
fn item_align(item: &LayoutStyle, container: &LayoutStyle) -> AlignItems {
    item.align_self.resolve(container.align_items)
}

/// Single-line (no-wrap) flex layout (spec §4.1, §4.2, §4.6).
///
/// Lays items out along a main axis chosen by `style.flex_direction`, resolves
/// their main sizes by distributing free space via `flex_grow` / `flex_shrink`,
/// positions them with `justify_content` along the main axis, and aligns them
/// with `align_items` along the cross axis.
///
/// # Algorithm (Phase-1 subset)
///
/// 1. The main axis is horizontal for [`Row`]/[`RowReverse`] and vertical for
///    [`Column`]/[`ColumnReverse`]. Container content extents honour the
///    container's own explicit `width` / `height` (resolved against
///    `constraints.max`); an `Auto` extent falls back to `constraints.max` minus
///    border + padding. Both are clamped to be non-negative. Free space, `fr`
///    growth, percentages, and `justify_content` all resolve against this extent.
/// 2. Each item's **base main size** is `flex_basis` (resolved against the
///    container main extent); else its main-axis size (`width` for a row,
///    `height` for a column); else its measured natural main size (measured with
///    loose constraints up to the container content box). Never negative. An
///    **intrinsic** keyword on either source (`flex-basis: min-content`,
///    `width: fit-content`) is answered by measuring the item under a zero /
///    unbounded main-axis offer — see [`intrinsic_main`].
///
///    Two neighbours of that path still degrade an intrinsic keyword to `auto`:
///    an item's **cross**-axis size and bounds (step 5 / [`clamp_cross`]), and
///    the **container's own** `width` / `height` and `min-`/`max-` bounds in
///    step 1 — `display: flex; width: min-content` fills its parent. Both want a
///    min-content size *of a flex line*, which nothing here computes: the item
///    probes above answer for one item, not for the line they share.
/// 3. `free = container_main - (sum of base sizes) - gap * (count - 1)`.
/// 4. If `free > 0` and `sum(flex_grow) > 0`, each item grows by
///    `free * grow_i / sum(grow)`. If `free < 0` and the scaled shrink factor
///    `sum(flex_shrink_i * base_i) > 0`, each item shrinks proportionally to
///    `flex_shrink_i * base_i`, clamped so no final main size is negative.
///    Otherwise base sizes are kept and the slack is handled by justification.
/// 5. Cross size per item is the container cross extent when
///    `align_items == Stretch` (unless the item sets an explicit cross-axis size,
///    which wins over the stretch), else the item's measured cross size (measured
///    with its final main size fixed and a loose cross axis).
/// 6. `justify_content` turns the leftover main-axis space into a leading offset
///    and an inter-item spacing (`FlexStart`, `FlexEnd`, `Center`,
///    `SpaceBetween`, `SpaceAround`, `SpaceEvenly`); when nothing is left over
///    the leading offset and spacing are zero.
/// 7. Each item is positioned along the main axis (leading + accumulated sizes +
///    gaps + justify spacing) and along the cross axis per `align_items`
///    (`FlexStart` → 0, `FlexEnd` → `container_cross - item_cross`, `Center` →
///    half, `Stretch` → 0 filling the cross axis). Reverse directions place items
///    in reversed order along the main axis.
/// 8. Main/cross coordinates map back to x/y and are offset into the content box.
///    The node border-box size is the container content size plus border and
///    padding, then clamped into `constraints`.
///
/// [`Row`]: crate::model::FlexDirection::Row
/// [`RowReverse`]: crate::model::FlexDirection::RowReverse
/// [`Column`]: crate::model::FlexDirection::Column
/// [`ColumnReverse`]: crate::model::FlexDirection::ColumnReverse
///
/// # Out of scope for Phase 1
///
/// This is deliberately a **subset**: no wrapping / multi-line flex, no `order`
/// reordering, no baseline alignment (`AlignItems::Baseline` is not modelled),
/// and no `align-self` per-item cross override. These belong to later phases.
///
/// Per-item **main-axis min/max clamping** (`min-width`/`max-width` in a row,
/// the height bounds in a column) and **margins** (which consume main-axis space
/// and offset each item) *are* honoured; cross-axis min/max remains later work.
///
/// # `auto` margins take the free space before `justify-content` sees it
///
/// CSS Flexbox §8.1: when a line has positive free space and any main-axis margin
/// on it is `auto`, that space is split equally among the `auto` margins and
/// `justify-content` gets none. The ordering *is* the feature — it is why
/// `margin-left: auto` pushes an item to the end of a row no matter what
/// `justify-content` says, and why `justify-content` appears to stop working the
/// moment an `auto` margin is added. See [`place_line`].
///
/// **Cross-axis `auto` margins are not honoured**: §8.1 gives them the same job in
/// the cross axis, where they would absorb the line's leftover cross space and
/// override `align-self`. Only the main axis distributes here, so a cross-axis
/// `auto` margin reserves nothing and alignment decides alone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlexLayout;

impl LayoutAlgorithm for FlexLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let is_row = style.flex_direction.is_row();
        let is_reverse = style.flex_direction.is_reverse();
        let gap = style.gap;

        let border = style.border;
        let padding = style.padding;
        let bp_left = border.left + padding.left;
        let bp_top = border.top + padding.top;
        let bp_x = bp_left + border.right + padding.right;
        let bp_y = bp_top + border.bottom + padding.bottom;

        // Step 1: container content extents, split into main / cross. An explicit
        // `width` / `height` on the container is honoured (resolved against the
        // available max, so a fixed px or a percentage of it both work); an `Auto`
        // dimension falls back to the available content extent, so the common
        // fill-available case is byte-identical to before.
        //
        // `box-sizing` governs how an explicit size maps to the content extent:
        // `content-box` (the default) takes it as-is, `border-box` subtracts the
        // axis border + padding (floored at `0`). The `Auto` branch already
        // subtracts border + padding, so both modes are consistent — no phantom
        // inflation and no clamp mismatch when an explicitly-sized container is
        // arranged at its authored (border-box) size by its parent.
        let content_width = style
            .width
            .resolve(constraints.max.width)
            .map_or(constraints.max.width - bp_x, |w| {
                style.box_sizing.content_extent(w, bp_x)
            })
            .max(0.0);
        let content_height = style
            .height
            .resolve(constraints.max.height)
            .map_or(constraints.max.height - bp_y, |h| {
                style.box_sizing.content_extent(h, bp_y)
            })
            .max(0.0);
        let (container_main, container_cross) = if is_row {
            (content_width, content_height)
        } else {
            (content_height, content_width)
        };

        // Whether the block (height) extent is content-sized rather than
        // fill-available. In horizontal writing mode `height: auto` hugs its
        // content, but only when the incoming height constraint is *indefinite*
        // (a loose measure, `min.height != max.height`, or an unbounded axis). A
        // *definite* (tight) height from the parent — the constraint every
        // `arrange` pass supplies — fills as before, so placement is unchanged and
        // only the measured size a parent observes shrinks to fit.
        //
        // For a column this shrinks the MAIN axis to the stacked children (the
        // previous "fill the whole available height" was the root of nested
        // flex-column items over-reporting their height); for a row it shrinks the
        // CROSS axis to the tallest item. Width (the inline axis) always fills.
        let height_is_auto = style.height.resolve(constraints.max.height).is_none();
        let height_indefinite = height_is_auto
            && (!constraints.max.height.is_finite()
                || (constraints.max.height - constraints.min.height).abs() > f32::EPSILON);

        let count = children.len();

        // Per-item margins split into main-leading / main-trailing (which consume
        // main-axis space and offset items) and a cross-leading offset. The
        // physical edge that leads in cursor order depends on direction and
        // reversal; a zero-margin item contributes nothing on every edge, so this
        // is a no-op for the common case.
        let mut main_lead = Vec::with_capacity(count);
        let mut main_trail = Vec::with_capacity(count);
        let mut cross_lead = Vec::with_capacity(count);
        for child in children {
            // Edges stay [`Margin`] rather than collapsing to lengths here, because
            // an `auto` edge has two answers downstream: it *reserves* nothing
            // (§8.1) but it *absorbs* free space (see `place_line`). Flattening it
            // to `0` now would throw the second one away.
            //
            // Percentages, on the other hand, resolve right here: CSS 2.1 §8.3
            // resolves every margin edge against the containing block's INLINE size
            // — this container's content width — regardless of which way the flex
            // axis runs. It is known, it cannot change, and resolving it once leaves
            // only `Px` and `Auto` for the rest of the algorithm to think about.
            let fix = |edge: Margin| match edge {
                Margin::Percent(_) => Margin::Px(edge.resolve(content_width)),
                px_or_auto => px_or_auto,
            };
            let m = child.style.margin;
            let (lead, trail) = if is_row {
                if is_reverse {
                    (m.right, m.left)
                } else {
                    (m.left, m.right)
                }
            } else if is_reverse {
                (m.bottom, m.top)
            } else {
                (m.top, m.bottom)
            };
            main_lead.push(fix(lead));
            main_trail.push(fix(trail));
            cross_lead.push(fix(if is_row { m.top } else { m.left }));
        }

        // Step 2: per item, the **flex base size** (`flex_basis`/`width`/measured,
        // never negative) and the **hypothetical main size** (the flex base clamped
        // into the item's main-axis min/max range). The freeze loop (§9.7) needs
        // both: the unclamped base is what grow/shrink distribute from, while the
        // clamped hypothetical drives line-breaking and the freeze-inflexible test.
        let mut flex_base = Vec::with_capacity(count);
        let mut base_main = Vec::with_capacity(count);
        // Per-item main-axis min floor / max ceiling, resolved to pixels. `main_max`
        // is `f32::INFINITY` when the item has no `max`. `main_min` carries the
        // §4.5 content-based automatic minimum when the item's `min` is `auto` /
        // unset, and the explicit `min` otherwise. Both the `base_main` clamp below
        // and the §9.7 freeze loop consume these so the automatic minimum acts as
        // the true shrink floor.
        let mut main_min = Vec::with_capacity(count);
        let mut main_max = Vec::with_capacity(count);
        // Items whose §4.5 automatic minimum is a `content.min(fb)` floor (row main
        // axis, or any definite basis): that floor is `<= fb`, so it never raises the
        // item's base size and can only bite when the item is actually shrunk below
        // it. Its `content_max_main` measure is therefore DEFERRED to the per-line
        // shrink gate in `place_line` (resolved only for lines under real shrink
        // pressure). `false` means the item's floor was already resolved eagerly.
        let mut deferred_floor = vec![false; count];
        for (index, child) in children.iter().enumerate() {
            let main_dim = if is_row {
                child.style.width
            } else {
                child.style.height
            };
            // An explicit main size (from `flex-basis` or `width`/`height`) is
            // authored against the item's box-sizing, so it is expanded to the
            // border-box extent the item occupies (`content-box` adds the item's
            // main-axis border + padding). The measured fallback already reports a
            // border box, so it is used as-is.
            //
            // Each source is asked for an INTRINSIC answer before an arithmetic
            // one, because `Dimension::resolve` cannot tell the two apart: it
            // answers `None` for `min-content` exactly as it does for `auto`, so
            // an unasked keyword falls through to the next source and, in the end,
            // to the natural-size fallback — `flex-basis: max-content` meaning
            // `flex-basis: auto`. `intrinsic_main` measures nothing for a
            // non-keyword, so the common item costs what it always did.
            let explicit_main = |explicit: f32| {
                child
                    .style
                    .box_sizing
                    .border_extent(explicit, axis_bp(&child.style, is_row))
            };
            let basis = child.style.flex_basis;
            let base = if let Some(px) = intrinsic_main(
                &mut *measurer,
                index,
                basis,
                is_row,
                container_main,
                container_cross,
            ) {
                px
            } else if let Some(explicit) = basis.resolve(container_main) {
                explicit_main(explicit)
            } else if let Some(px) = intrinsic_main(
                &mut *measurer,
                index,
                main_dim,
                is_row,
                container_main,
                container_cross,
            ) {
                px
            } else if let Some(explicit) = main_dim.resolve(container_main) {
                explicit_main(explicit)
            } else {
                let measured = measurer.measure_child(
                    index,
                    BoxConstraints::loose(Size::new(content_width, content_height)),
                );
                if is_row {
                    measured.width
                } else {
                    measured.height
                }
            };
            let (min_main, max_main) = main_bounds(&child.style, is_row);
            let fb = base.max(0.0);
            let max_val = max_main.resolve(container_main).unwrap_or(f32::INFINITY);
            // §4.5 automatic minimum size: an `auto` (or unset) main-axis `min` does
            // not floor at 0 — its used value is the item's content-based minimum
            // size. An EXPLICIT `min` is used verbatim (byte-identical to before).
            // The extra `content_max_main` measure is computed LAZILY: it runs only
            // when it can actually change the result, so the common frame takes zero
            // extra measures.
            let min_floor = if let Some(explicit) = min_main.resolve(container_main) {
                explicit
            } else {
                let has_definite = child
                    .style
                    .flex_basis
                    .resolve(container_main)
                    .or_else(|| main_dim.resolve(container_main))
                    .is_some();
                if !is_row && !has_definite {
                    // Block (column) main axis with an auto basis: the block does not
                    // reflow, so max-content == min-content and the content extent is
                    // the exact automatic minimum. It differs from `fb` ONLY when `fb`
                    // was clamped by a finite, undersized container main (the
                    // card-collapse case) — then the natural size may exceed the
                    // clamped `fb`, so the floor must be measured. When the container
                    // main is unbounded, or `fb` sits below it, `natural == fb`: use
                    // `fb` as the floor with NO extra measure.
                    if container_main.is_finite() && fb >= container_main {
                        let content =
                            content_max_main(&mut *measurer, index, is_row, container_cross);
                        content.min(max_val).max(0.0)
                    } else {
                        fb.min(max_val).max(0.0)
                    }
                } else {
                    // Inline (row) main axis, or a definite basis: the floor is
                    // `content.min(fb) <= fb`, so it never inflates `base_main` and can
                    // only bite when the item is actually shrunk below it. Defer the
                    // content measure to `place_line`, which resolves it only for lines
                    // under real shrink pressure; 0 is the pre-§4.5 baseline used
                    // otherwise (output-identical whenever the line does not shrink,
                    // and it also covers the fill-available `!content.is_finite()` →
                    // no-floor case).
                    deferred_floor[index] = true;
                    0.0
                }
            };
            flex_base.push(fb);
            base_main.push(clamp_floor(fb, min_floor, max_val));
            main_min.push(min_floor);
            main_max.push(max_val);
        }

        // Steps 3–8: break the items into flex lines, then size / justify /
        // align each line independently and stack the lines along the cross axis.
        //
        // `flex-wrap: nowrap` (the default) has exactly one line holding every
        // item; it is byte-for-byte identical to the historical single-line path
        // (`stretch_target = Some(container_cross)` reproduces the old
        // fill-the-container stretch and container-relative alignment).
        let mut child_boxes = vec![LayoutBox::default(); count];

        if !style.flex_wrap.is_multi_line() {
            let line: Vec<usize> = (0..count).collect();

            // The main-axis extent the single line's items occupy: their base main
            // sizes, the inter-item gaps, and their main-axis margins. This is the
            // content main size a column hugs to under an indefinite height.
            let gap_span = gap * (count.saturating_sub(1) as f32);
            let sum_base: f32 = base_main.iter().sum();
            let sum_margin_main: f32 = main_lead
                .iter()
                .zip(main_trail.iter())
                .map(|(l, t)| reserved(*l) + reserved(*t))
                .sum();
            let content_main = sum_base + gap_span + sum_margin_main;

            let placed = place_line(LineArgs {
                line: &line,
                base_main: &base_main,
                flex_base: &flex_base,
                main_min: &main_min,
                main_max: &main_max,
                deferred_floor: &deferred_floor,
                children,
                style,
                is_row,
                is_reverse,
                line_main: container_main,
                gap,
                main_lead: &main_lead,
                main_trail: &main_trail,
                cross_lead: &cross_lead,
                stretch_target: Some(container_cross),
                container_cross,
                bp_left,
                bp_top,
                cross_offset: 0.0,
                // The content cross extent is only needed to hug the (auto,
                // indefinite) height of a ROW; otherwise skip the extra
                // pre-stretch measurement so the definite path stays identical.
                content_cross_needed: height_indefinite && is_row,
                measurer: &mut *measurer,
                child_boxes: &mut child_boxes,
            });

            // Content-box height: `auto` + indefinite hugs its content (a column's
            // stacked main extent, a row's tallest item); otherwise fill the
            // resolved/available height exactly as before.
            let final_content_height = if height_indefinite {
                if is_row {
                    placed.content_cross
                } else {
                    content_main
                }
            } else {
                content_height
            };

            // Clamp the container's OWN box into its authored min/max (spec §4.2)
            // — e.g. `min-height: 100vh` on a flex column fills a tall viewport.
            let own_w = clamp_content(
                content_width,
                style.min_width,
                style.max_width,
                constraints.max.width,
                bp_x,
                style.box_sizing,
            );
            let own_h = clamp_content(
                final_content_height,
                style.min_height,
                style.max_height,
                constraints.max.height,
                bp_y,
                style.box_sizing,
            );
            let border_box = Size::new(own_w + bp_x, own_h + bp_y);
            return LayoutOutput::new(constraints.constrain(border_box), child_boxes);
        }

        // Multi-line: greedily fill lines along the main axis. An item (with its
        // main-axis margins) that does not fit on the current non-empty line
        // starts a new one; an item wider than the container occupies its own line.
        let mut lines: Vec<Vec<usize>> = Vec::new();
        let mut current: Vec<usize> = Vec::new();
        let mut current_main = 0.0_f32;
        for index in 0..count {
            let item_main =
                base_main[index] + reserved(main_lead[index]) + reserved(main_trail[index]);
            let with_item = if current.is_empty() {
                item_main
            } else {
                current_main + gap + item_main
            };
            if !current.is_empty() && with_item > container_main {
                lines.push(std::mem::take(&mut current));
                current_main = item_main;
            } else {
                current_main = with_item;
            }
            current.push(index);
        }
        if !current.is_empty() {
            lines.push(current);
        }

        // `wrap-reverse` stacks the lines in the opposite cross-axis order: the
        // last line leads at the cross start.
        if style.flex_wrap.is_reverse() {
            lines.reverse();
        }

        // Stack each line at the running cross offset; the cross-axis gap sits
        // between adjacent lines. The container's cross extent grows to the sum of
        // the line cross sizes when it is `auto` (so all lines fit without
        // overlap); an explicit cross size is kept as authored.
        let mut cross_offset = 0.0_f32;
        let mut total_cross = 0.0_f32;
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                cross_offset += gap;
                total_cross += gap;
            }
            let line_cross = place_line(LineArgs {
                line,
                base_main: &base_main,
                flex_base: &flex_base,
                main_min: &main_min,
                main_max: &main_max,
                deferred_floor: &deferred_floor,
                children,
                style,
                is_row,
                is_reverse,
                line_main: container_main,
                gap,
                main_lead: &main_lead,
                main_trail: &main_trail,
                cross_lead: &cross_lead,
                stretch_target: None,
                container_cross,
                bp_left,
                bp_top,
                cross_offset,
                // The multi-line path already resolves every item's own cross size
                // to compute the line cross, so no extra pre-stretch pass is needed.
                content_cross_needed: false,
                measurer: &mut *measurer,
                child_boxes: &mut child_boxes,
            })
            .cross;
            cross_offset += line_cross;
            total_cross += line_cross;
        }

        let cross_is_auto = if is_row {
            style.height.resolve(constraints.max.height).is_none()
        } else {
            style.width.resolve(constraints.max.width).is_none()
        };
        let (final_content_width, final_content_height) = if is_row {
            let h = if cross_is_auto {
                total_cross
            } else {
                content_height
            };
            (content_width, h)
        } else {
            let w = if cross_is_auto {
                total_cross
            } else {
                content_width
            };
            (w, content_height)
        };

        // Clamp the container's OWN box into its authored min/max (spec §4.2).
        let own_w = clamp_content(
            final_content_width,
            style.min_width,
            style.max_width,
            constraints.max.width,
            bp_x,
            style.box_sizing,
        );
        let own_h = clamp_content(
            final_content_height,
            style.min_height,
            style.max_height,
            constraints.max.height,
            bp_y,
            style.box_sizing,
        );
        let border_box = Size::new(own_w + bp_x, own_h + bp_y);
        LayoutOutput::new(constraints.constrain(border_box), child_boxes)
    }
}

/// The main-axis space a margin edge **reserves** — what must be subtracted
/// before there is any free space to speak of.
///
/// `auto` reserves nothing: CSS Flexbox §8.1 gives it the opposite job, absorbing
/// free space *after* the items are sized. So it is `0` here and re-enters the
/// arithmetic in [`place_line`], the only place that knows how much there is to
/// absorb.
///
/// The basis is irrelevant: percentages were resolved against the container's
/// inline size when the edges were extracted, so only `Px` and `Auto` reach here.
fn reserved(margin: Margin) -> f32 {
    margin.resolve(0.0)
}

/// The main-axis space a margin edge finally **occupies**: what it reserved, or
/// its share of the line's free space when it is `auto` (CSS Flexbox §8.1).
///
/// The counterpart to [`reserved`], and the two disagree on exactly one input.
/// `reserved` answers "what has to come off the top before there is free space",
/// which for `auto` is nothing; this answers "where does the item actually sit",
/// which for `auto` is its share of what was left. Calling the wrong one is how
/// the free space gets counted twice.
fn used(margin: Margin, auto_share: f32) -> f32 {
    if margin.is_auto() {
        auto_share
    } else {
        reserved(margin)
    }
}

/// Inputs for [`place_line`]: one flex line's items plus the shared container /
/// style context. Grouped into a struct because the single-line sizing pass
/// legitimately needs this many parameters and a positional argument list would
/// be error-prone.
struct LineArgs<'a, 'm> {
    /// The item indices on this line, in source order.
    line: &'a [usize],
    /// Hypothetical main size per item (flex base clamped to content min/max),
    /// indexed by the global item index. Drives line-breaking and the
    /// freeze-inflexible test in the §9.7 resolve-flexible-lengths loop.
    base_main: &'a [f32],
    /// Unclamped flex base size per item, indexed by the global item index. This
    /// is what grow/shrink distribute from in the §9.7 freeze loop.
    flex_base: &'a [f32],
    /// Pre-resolved main-axis min floor / max ceiling per item, indexed by the
    /// global item index (`main_max` is `f32::INFINITY` when the item has no
    /// `max`). `main_min` folds in the §4.5 content-based automatic minimum on the
    /// `auto` path, so the freeze loop shrinks items no further than their content.
    main_min: &'a [f32],
    main_max: &'a [f32],
    /// Whether each item's §4.5 content floor was deferred (a `content.min(fb)`
    /// floor whose content measure is taken here, lazily, only when this line
    /// shrinks). Indexed by the global item index.
    deferred_floor: &'a [bool],
    /// The flex items.
    children: &'a [LayoutChild],
    /// The flex container's style.
    style: &'a LayoutStyle,
    /// Whether the main axis is horizontal.
    is_row: bool,
    /// Whether items are placed in reverse main-axis order.
    is_reverse: bool,
    /// The main-axis extent this line's free space / justification resolve against.
    line_main: f32,
    /// Gap between adjacent items on the line.
    gap: f32,
    /// Leading / trailing main-axis margins, indexed by the global item index.
    main_lead: &'a [Margin],
    main_trail: &'a [Margin],
    /// Leading cross-axis margin, indexed by the global item index.
    cross_lead: &'a [Margin],
    /// When `Some(target)`, `stretch` fills `target` and alignment resolves
    /// against `container_cross` (the single-line / no-wrap semantics). When
    /// `None`, the line's cross size is the max item cross size and `stretch`
    /// fills that (the multi-line semantics).
    stretch_target: Option<f32>,
    /// The container's cross extent (the measurement bound and, for the
    /// single-line path, the alignment reference).
    container_cross: f32,
    /// Content-box origin offsets.
    bp_left: f32,
    bp_top: f32,
    /// The line's leading cross-axis offset (0 for a single line).
    cross_offset: f32,
    /// Whether the caller needs [`PlacedLine::content_cross`] (the line's
    /// pre-stretch content cross extent). When `false`, a stretched item's
    /// hypothetical cross is not measured, keeping the definite path byte-identical
    /// and avoiding an unnecessary measurement.
    content_cross_needed: bool,
    /// Child measurer.
    measurer: &'m mut dyn ChildMeasurer,
    /// Output boxes, written at each item's global index.
    child_boxes: &'a mut [LayoutBox],
}

/// The result of placing one flex line.
struct PlacedLine {
    /// The line's cross extent, used to stack lines (the stretch target for the
    /// single-line path, the max item cross for the multi-line path).
    cross: f32,
    /// The line's **pre-stretch** content cross extent — the tallest item's
    /// hypothetical (measured or explicit) cross size. Equals [`Self::cross`] for
    /// the multi-line path; for the single-line path it is the height an
    /// (auto, indefinite) row hugs to. Only meaningful when the caller set
    /// `content_cross_needed`.
    content_cross: f32,
}

/// Sizes, justifies, and cross-aligns one flex line, writing each item's box and
/// returning the line's cross extent (spec §4.2, §4.6).
///
/// Distributes the line's main-axis free space via `flex_grow` / `flex_shrink`,
/// re-clamps into each item's main min/max range, positions items with
/// `justify_content` along the main axis, and aligns them with `align_items`
/// along the cross axis. Reverse directions place items in reversed main order.
fn place_line(args: LineArgs) -> PlacedLine {
    let LineArgs {
        line,
        base_main,
        flex_base,
        main_min,
        main_max,
        deferred_floor,
        children,
        style,
        is_row,
        is_reverse,
        line_main,
        gap,
        main_lead,
        main_trail,
        cross_lead,
        stretch_target,
        container_cross,
        bp_left,
        bp_top,
        cross_offset,
        content_cross_needed,
        measurer,
        child_boxes,
    } = args;

    let count = line.len();
    let gap_span = gap * (count.saturating_sub(1) as f32);

    // Margins consume main-axis space alongside the gaps; both are constants that
    // do not flex, so the justification pass below reuses this sum.
    let sum_margin_main: f32 = line
        .iter()
        .map(|&i| reserved(main_lead[i]) + reserved(main_trail[i]))
        .sum();

    // §4.5 content floors for the DEFERRED items (row main axis / definite basis)
    // are `content.min(fb) <= fb`, so they can only bite when this line's items are
    // actually shrunk below their flex base — i.e. when the base main sizes overflow
    // the space available to them. Resolve them lazily HERE, and only then: a line
    // that grows or exactly fits takes ZERO extra measures (its floors would never
    // bind, so the pre-§4.5 baseline of 0 is output-identical).
    let available = line_main - gap_span - sum_margin_main;
    let sum_base: f32 = line.iter().map(|&i| base_main[i]).sum();
    let main_min_owned;
    let main_min: &[f32] = if sum_base > available && line.iter().any(|&i| deferred_floor[i]) {
        let mut resolved = main_min.to_vec();
        for &i in line {
            if deferred_floor[i] {
                let content = content_max_main(&mut *measurer, i, is_row, container_cross);
                let floor = if content.is_finite() {
                    content.min(flex_base[i])
                } else {
                    0.0
                };
                resolved[i] = floor.min(main_max[i]).max(0.0);
            }
        }
        main_min_owned = resolved;
        &main_min_owned
    } else {
        main_min
    };

    // Steps 3–4: resolve the final main sizes with the canonical
    // "resolve flexible lengths" freeze loop (§9.7).
    let final_main = resolve_flexible_lengths(
        line,
        flex_base,
        base_main,
        main_min,
        main_max,
        children,
        line_main,
        gap_span + sum_margin_main,
    );

    // Step 5: cross size per item, and the line's cross extent.
    //
    // For the single-line path (`stretch_target = Some`), a stretched item fills
    // the container cross exactly as before and no measurement is taken. For the
    // multi-line path, every item's cross size is resolved (explicit) or measured,
    // the line cross is their max, and a stretched item fills that line cross.
    let mut cross_size: Vec<f32> = Vec::with_capacity(count);
    let line_cross;
    let mut content_cross = 0.0_f32;
    match stretch_target {
        Some(target) => {
            for (pos, &i) in line.iter().enumerate() {
                let cross = cross_item_size(
                    item_align(&children[i].style, style) == AlignItems::Stretch,
                    target,
                    &children[i].style,
                    is_row,
                    final_main[pos],
                    container_cross,
                    i,
                    &mut *measurer,
                );
                cross_size.push(cross);
                // The line's pre-stretch content cross: a stretched item does not
                // report its used (filled) cross but its hypothetical size — its
                // explicit cross, else its measured cross. Only measured when the
                // caller needs it (an auto, indefinite row), so the definite path
                // takes no extra measurement and is byte-identical.
                if content_cross_needed {
                    let hypothetical = cross_item_size(
                        false,
                        0.0,
                        &children[i].style,
                        is_row,
                        final_main[pos],
                        container_cross,
                        i,
                        &mut *measurer,
                    );
                    content_cross = content_cross.max(hypothetical);
                }
            }
            line_cross = target;
        }
        None => {
            // First resolve/measure each item's own cross size (no stretch), so
            // the line cross size is known before stretched items fill it.
            let mut base_cross: Vec<f32> = Vec::with_capacity(count);
            for (pos, &i) in line.iter().enumerate() {
                let bc = cross_item_size(
                    false,
                    0.0,
                    &children[i].style,
                    is_row,
                    final_main[pos],
                    container_cross,
                    i,
                    &mut *measurer,
                );
                base_cross.push(bc);
            }
            let lc = base_cross.iter().copied().fold(0.0_f32, f32::max);
            for (pos, &i) in line.iter().enumerate() {
                let cross_style = &children[i].style;
                let cross_dim = if is_row {
                    cross_style.height
                } else {
                    cross_style.width
                };
                let has_explicit = cross_dim.resolve(container_cross).is_some();
                let cross =
                    if item_align(cross_style, style) == AlignItems::Stretch && !has_explicit {
                        // A stretch fills the line cross, but must still stop at the
                        // item's own cross max (§9.4 step 11) — `base_cross` came from
                        // `cross_item_size` already clamped, this path did not.
                        clamp_cross(lc, cross_style, is_row, container_cross)
                    } else {
                        base_cross[pos]
                    };
                cross_size.push(cross);
            }
            line_cross = lc;
            content_cross = lc;
        }
    }

    // The alignment reference is the container cross for the single-line path and
    // the line cross for the multi-line path.
    let align_ref = match stretch_target {
        Some(_) => container_cross,
        None => line_cross,
    };

    // Step 6: distribute the line's free space — to `auto` margins first, and to
    // `justify_content` only if they leave any (CSS Flexbox §8.1).
    let total_final: f32 = final_main.iter().copied().sum();
    let remaining = line_main - total_final - gap_span - sum_margin_main;

    // §8.1: "if the remaining free space is positive and at least one main-axis
    // margin on this line is `auto`, the free space is distributed equally among
    // those margins" — and, crucially, `justify-content` then has nothing left to
    // distribute. That ordering is the whole behaviour: `margin-left: auto` on one
    // item pushes it (and everything after it) to the end of the row *regardless*
    // of `justify-content`, which is exactly why the idiom is used.
    let auto_main: usize = line
        .iter()
        .map(|&i| usize::from(main_lead[i].is_auto()) + usize::from(main_trail[i].is_auto()))
        .sum();
    // Each `auto` edge's share. Zero when there are none, so `absorbed` below is a
    // no-op and every existing line places byte-identically.
    let auto_share = if auto_main > 0 && remaining > 0.0 {
        remaining / auto_main as f32
    } else {
        0.0
    };
    // What `justify_content` still has to work with: nothing, once any `auto`
    // margin has claimed it.
    let free_for_justify = if auto_main > 0 { 0.0 } else { remaining };

    let (leading, spacing) = if free_for_justify <= 0.0 || count == 0 {
        (0.0, 0.0)
    } else {
        let n = count as f32;
        match style.justify_content {
            JustifyContent::FlexStart => (0.0, 0.0),
            JustifyContent::FlexEnd => (free_for_justify, 0.0),
            JustifyContent::Center => (free_for_justify / 2.0, 0.0),
            JustifyContent::SpaceBetween => {
                if count > 1 {
                    (0.0, free_for_justify / (n - 1.0))
                } else {
                    (0.0, 0.0)
                }
            }
            JustifyContent::SpaceAround => {
                let s = free_for_justify / n;
                (s / 2.0, s)
            }
            JustifyContent::SpaceEvenly => {
                let s = free_for_justify / (n + 1.0);
                (s, s)
            }
        }
    };

    // Steps 7 & 8: position each item and map back to (x, y).
    let mut cursor = leading;
    let order: Vec<usize> = if is_reverse {
        (0..count).rev().collect()
    } else {
        (0..count).collect()
    };
    for &pos in &order {
        let i = line[pos];
        // A leading main-axis margin pushes the item further along the main axis
        // before it is placed — by its own length, or by its share of the line's
        // free space when it is `auto`.
        cursor += used(main_lead[i], auto_share);
        let main = final_main[pos];
        let cross = cross_size[pos];
        let cross_start = cross_offset
            + reserved(cross_lead[i])
            + match item_align(&children[i].style, style) {
                AlignItems::FlexStart | AlignItems::Start | AlignItems::Stretch => 0.0,
                AlignItems::FlexEnd | AlignItems::End => align_ref - cross,
                AlignItems::Center => (align_ref - cross) / 2.0,
            };
        let (origin, size) = if is_row {
            (
                Point::new(bp_left + cursor, bp_top + cross_start),
                Size::new(main, cross),
            )
        } else {
            (
                Point::new(bp_left + cross_start, bp_top + cursor),
                Size::new(cross, main),
            )
        };
        child_boxes[i] = LayoutBox::new(Rect::new(origin, size));
        cursor += main + used(main_trail[i], auto_share) + gap + spacing;
    }

    PlacedLine {
        cross: line_cross,
        content_cross,
    }
}

/// Resolves the final main sizes of one flex line's items with the canonical
/// **resolve flexible lengths** algorithm (CSS Flexbox Level 1, §9.7,
/// <https://drafts.csswg.org/css-flexbox-1/#resolve-flexible-lengths>).
///
/// This replaces a single-pass `size += free · factor / Σfactor` split, which is
/// wrong whenever an item clamps at its min/max (the violation must be frozen and
/// the freed space redistributed) or when `Σ`flex factors `< 1` (the line must be
/// left partly unfilled). The loop freezes items as they reach their limits and
/// re-distributes the remaining free space among the still-flexible ones until
/// every item is frozen.
///
/// `non_flex_main` is the main-axis space the gaps and item margins consume; it is
/// subtracted from the container inner main size once, since those extents never
/// flex. `flex_base` is each item's unclamped flex base size (what grow/shrink
/// distribute from) and `hypothetical` is that base clamped to the item's content
/// min/max (the freeze-inflexible reference). `main_min` / `main_max` are the
/// per-item pre-resolved main-axis bounds (the §4.5 automatic minimum folded into
/// `main_min`). All are indexed by the global item index; the returned vector is
/// indexed by position within `line`.
// These are cohesive positional inputs to one internal freeze loop; grouping them
// into a struct would not aid clarity.
#[allow(clippy::too_many_arguments)]
fn resolve_flexible_lengths(
    line: &[usize],
    flex_base: &[f32],
    hypothetical: &[f32],
    main_min: &[f32],
    main_max: &[f32],
    children: &[LayoutChild],
    line_main: f32,
    non_flex_main: f32,
) -> Vec<f32> {
    let n = line.len();

    // The main-axis extent available to the items themselves (inner main size less
    // the non-flexing gaps + margins). Free space below is measured against this.
    let available = line_main - non_flex_main;

    // §9.7 step 1: use grow factors when the line's items would otherwise leave
    // free space (their hypothetical outer main sizes sum to less than the inner
    // main size), else shrink factors. `growing == (free > 0)` under hypotheticals.
    let sum_hypo: f32 = line.iter().map(|&i| hypothetical[i]).sum();
    let growing = sum_hypo < available;

    // Per-position flex factor for the chosen direction, and the item's content
    // min/max bounds.
    let factor = |pos: usize| -> f32 {
        let s = &children[line[pos]].style;
        if growing { s.flex_grow } else { s.flex_shrink }
    };

    let mut target = vec![0.0_f32; n];
    let mut frozen = vec![false; n];

    // §9.7 step 2: freeze inflexible items at their hypothetical main size. An item
    // is inflexible when its used flex factor is 0, or it is already past the
    // target in the direction of flexing (growing with base > hypothetical, i.e.
    // clamped down by a max; shrinking with base < hypothetical, clamped up by a
    // min) so flexing could only push it further out of range.
    for pos in 0..n {
        let i = line[pos];
        let inflexible = factor(pos) == 0.0
            || (growing && flex_base[i] > hypothetical[i])
            || (!growing && flex_base[i] < hypothetical[i]);
        if inflexible {
            frozen[pos] = true;
            target[pos] = hypothetical[i];
        }
    }

    // The used main size: frozen items contribute their resolved target, unfrozen
    // items their (unclamped) flex base size.
    let used_main = |target: &[f32], frozen: &[bool]| -> f32 {
        (0..n)
            .map(|pos| {
                if frozen[pos] {
                    target[pos]
                } else {
                    flex_base[line[pos]]
                }
            })
            .sum()
    };

    // §9.7 step 3: the initial free space, retained for the `Σfactors < 1` clamp.
    let initial_free = available - used_main(&target, &frozen);

    // §9.7 step 4: loop until every item is frozen.
    loop {
        if frozen.iter().all(|&f| f) {
            break;
        }

        // (b) remaining free space against the current (partly frozen) sizes.
        let mut remaining = available - used_main(&target, &frozen);

        // Sum of the unfrozen flex factors. If it is < 1, the spec caps the space
        // handed out this pass at `initial_free · Σfactors` when that magnitude is
        // smaller — this is what makes a lone `flex-grow: 0.5` item claim only half
        // the free space instead of all of it.
        let sum_factors: f32 = (0..n).filter(|&pos| !frozen[pos]).map(factor).sum();
        if sum_factors < 1.0 {
            let scaled = initial_free * sum_factors;
            if scaled.abs() < remaining.abs() {
                remaining = scaled;
            }
        }

        // (c) distribute the remaining free space to the unfrozen items.
        if growing {
            let sum_grow: f32 = (0..n).filter(|&pos| !frozen[pos]).map(factor).sum();
            if sum_grow > 0.0 {
                for pos in 0..n {
                    if !frozen[pos] {
                        target[pos] = flex_base[line[pos]] + remaining * factor(pos) / sum_grow;
                    }
                }
            }
        } else {
            // Shrinking weights each item by its **scaled** flex shrink factor:
            // `flex_shrink · flex_base` (a larger item gives up more), per §9.7.
            let scaled_shrink = |pos: usize| factor(pos) * flex_base[line[pos]];
            let sum_scaled: f32 = (0..n).filter(|&pos| !frozen[pos]).map(scaled_shrink).sum();
            if sum_scaled > 0.0 {
                for pos in 0..n {
                    if !frozen[pos] {
                        target[pos] = flex_base[line[pos]]
                            - remaining.abs() * scaled_shrink(pos) / sum_scaled;
                    }
                }
            }
        }

        // (d) clamp each unfrozen target to its content min/max (content box floored
        // at 0) and record the signed violation `clamped − unclamped`.
        let mut total_violation = 0.0_f32;
        let mut violations = vec![0.0_f32; n];
        for pos in 0..n {
            if frozen[pos] {
                continue;
            }
            let i = line[pos];
            let unclamped = target[pos];
            let clamped = clamp_floor(unclamped, main_min[i], main_max[i]);
            violations[pos] = clamped - unclamped;
            total_violation += violations[pos];
            target[pos] = clamped;
        }

        // (e) freeze by the sign of the total violation: no net violation freezes
        // everything; a positive total freezes the min-violators (whose clamp
        // pushed them up); a negative total freezes the max-violators. Freezing only
        // the sign-matched items lets the loop redistribute their freed space.
        if total_violation == 0.0 {
            frozen.fill(true);
        } else if total_violation > 0.0 {
            for pos in 0..n {
                if !frozen[pos] && violations[pos] > 0.0 {
                    frozen[pos] = true;
                }
            }
        } else {
            for pos in 0..n {
                if !frozen[pos] && violations[pos] < 0.0 {
                    frozen[pos] = true;
                }
            }
        }
    }

    target
}

/// Resolves one item's cross-axis size.
///
/// When `stretch` is set, the item fills `stretch_target` unless it sets an
/// explicit cross-axis size (which wins, expanded to its border-box extent).
/// Otherwise the item's cross size is measured with its final main size fixed and
/// a loose cross axis bounded by `container_cross`.
#[allow(clippy::too_many_arguments)]
fn cross_item_size(
    stretch: bool,
    stretch_target: f32,
    cross_style: &LayoutStyle,
    is_row: bool,
    main: f32,
    container_cross: f32,
    index: usize,
    measurer: &mut dyn ChildMeasurer,
) -> f32 {
    let raw = if stretch {
        let cross_dim = if is_row {
            cross_style.height
        } else {
            cross_style.width
        };
        cross_dim
            .resolve(container_cross)
            .map_or(stretch_target, |explicit| {
                cross_style
                    .box_sizing
                    .border_extent(explicit, axis_bp(cross_style, !is_row))
            })
    } else {
        let item_constraints = if is_row {
            BoxConstraints {
                min: Size::new(main, 0.0),
                max: Size::new(main, container_cross),
            }
        } else {
            BoxConstraints {
                min: Size::new(0.0, main),
                max: Size::new(container_cross, main),
            }
        };
        let measured = measurer.measure_child(index, item_constraints);
        if is_row {
            measured.height
        } else {
            measured.width
        }
    };
    // Every cross size — explicit, stretched, or measured — is clamped by the
    // item's own cross min/max (§9.4 step 11). Clamping here rather than at each
    // call site keeps the hypothetical and used cross sizes consistent.
    clamp_cross(raw, cross_style, is_row, container_cross)
}

#[cfg(test)]
mod tests {
    use super::FlexLayout;
    use crate::algorithm::{ChildMeasurer, FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{
        AlignItems, AlignSelf, BoxSizing, Dimension, FlexDirection, FlexWrap, JustifyContent,
        LayoutDisplay, LayoutStyle,
    };
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    /// Container style with the given direction/justify/align and gap.
    fn container(
        direction: FlexDirection,
        justify: JustifyContent,
        align: AlignItems,
        gap: f32,
    ) -> LayoutStyle {
        LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_direction: direction,
            justify_content: justify,
            align_items: align,
            gap,
            ..LayoutStyle::default()
        }
    }

    /// An item with a fixed flex-basis, no growth, no shrink.
    fn fixed_basis(raw: u64, basis: f32) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(raw),
            LayoutStyle {
                display: LayoutDisplay::Flex,
                flex_basis: Dimension::Px(basis),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..LayoutStyle::default()
            },
        )
    }

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected {b}, got {a}");
    }

    // ---- per-item align-self + cross min/max (audit Tier-1 §4) --------------

    /// A fixed-basis item carrying a per-item `align-self` override.
    fn with_self(raw: u64, basis: f32, align: AlignSelf) -> LayoutChild {
        let LayoutChild { node, style } = fixed_basis(raw, basis);
        LayoutChild::new(node, style.with_align_self(align))
    }

    #[test]
    fn align_self_overrides_the_containers_align_items() {
        // The container stretches, but item 2 opts out with `align-self: center`
        // and item 3 with `flex-end`. Flex read only the container's `align-items`,
        // so both overrides were silently dropped — while grid honoured them.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [
            fixed_basis(1, 50.0),
            with_self(2, 50.0, AlignSelf::Center),
            with_self(3, 50.0, AlignSelf::End),
        ];
        // The measured (hypothetical) cross size of every item is 20 tall.
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(50.0, 20.0),
            Size::new(50.0, 20.0),
            Size::new(50.0, 20.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Item 1 still inherits `stretch`: fills the 100 cross at y=0.
        assert_eq!(b[0].size().height, 100.0);
        approx(b[0].origin().y, 0.0);
        // Item 2 centers its own 20-tall box: (100-20)/2 = 40.
        assert_eq!(b[1].size().height, 20.0);
        approx(b[1].origin().y, 40.0);
        // Item 3 sits at the cross end: 100-20 = 80.
        assert_eq!(b[2].size().height, 20.0);
        approx(b[2].origin().y, 80.0);
    }

    #[test]
    fn align_self_stretch_opts_into_stretch_under_a_flex_start_container() {
        // The override works in both directions: a `flex-start` container leaves
        // items at their measured cross, but `align-self: stretch` fills.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), with_self(2, 50.0, AlignSelf::Stretch)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 20.0), Size::new(50.0, 20.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(
            b[0].size().height,
            20.0,
            "inherits flex-start: measured cross"
        );
        assert_eq!(
            b[1].size().height,
            100.0,
            "align-self: stretch fills the cross"
        );
    }

    #[test]
    fn a_stretched_item_stops_at_its_own_cross_max() {
        // `align-items: stretch` would fill the 100 cross, but the item's own
        // `max-height: 40` caps it (§9.4 step 11). The cross axis dropped its
        // min/max entirely while the main axis always applied them.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let LayoutChild { node, style: item } = fixed_basis(1, 50.0);
        let children = [LayoutChild::new(
            node,
            LayoutStyle {
                max_height: Dimension::Px(40.0),
                ..item
            },
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].size().height, 40.0);
    }

    #[test]
    fn a_measured_item_is_floored_by_its_own_cross_min() {
        // A 20-tall measured item under a `flex-start` container is floored to its
        // own `min-height: 60`.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let LayoutChild { node, style: item } = fixed_basis(1, 50.0);
        let children = [LayoutChild::new(
            node,
            LayoutStyle {
                min_height: Dimension::Px(60.0),
                ..item
            },
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].size().height, 60.0);
    }

    #[test]
    fn a_column_clamps_its_cross_max_width() {
        // The cross axis is width in a column: `max-width: 30` caps a stretch.
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let LayoutChild { node, style: item } = fixed_basis(1, 50.0);
        let children = [LayoutChild::new(
            node,
            LayoutStyle {
                max_width: Dimension::Px(30.0),
                ..item
            },
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(20.0, 50.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].size().width, 30.0);
    }

    #[test]
    fn row_justify_flex_start_places_items_left() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let mut measurer = FixedChildMeasurer::default();
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(50.0, 0.0));
        // Stretch fills the cross axis.
        assert_eq!(b[0].size(), Size::new(50.0, 100.0));
        assert_eq!(out.size(), Size::new(200.0, 100.0));
    }

    #[test]
    fn row_justify_variants_place_items() {
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let constraints = BoxConstraints::tight(Size::new(200.0, 100.0));

        // Center: leading = remaining/2 = 50.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::Center,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 50.0);
        approx(out.child_boxes()[1].origin().x, 100.0);

        // FlexEnd: leading = remaining = 100.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexEnd,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 100.0);
        approx(out.child_boxes()[1].origin().x, 150.0);

        // SpaceBetween: spacing = remaining/(2-1) = 100.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::SpaceBetween,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 0.0);
        approx(out.child_boxes()[1].origin().x, 150.0);

        // SpaceEvenly: s = 100/3; item0 at s, item1 at 50 + 2s.
        let s = 100.0_f32 / 3.0;
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::SpaceEvenly,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, s);
        approx(out.child_boxes()[1].origin().x, 50.0 + 2.0 * s);
    }

    #[test]
    fn flex_grow_splits_positive_free_space() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 50.0);
        a.style.flex_grow = 1.0;
        let mut b = fixed_basis(2, 50.0);
        b.style.flex_grow = 3.0;
        let children = [a, b];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        // free = 100, split 1:3 → 25 and 75 added.
        let boxes = out.child_boxes();
        approx(boxes[0].size().width, 75.0);
        approx(boxes[1].size().width, 125.0);
        approx(boxes[0].origin().x, 0.0);
        approx(boxes[1].origin().x, 75.0);
    }

    #[test]
    fn flex_shrink_absorbs_overflow() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 80.0);
        a.style.flex_shrink = 1.0;
        let mut b = fixed_basis(2, 80.0);
        b.style.flex_shrink = 1.0;
        let children = [a, b];
        // container main = 100, total base = 160, overflow = 60 split evenly.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        approx(boxes[0].size().width, 50.0);
        approx(boxes[1].size().width, 50.0);
        approx(boxes[1].origin().x, 50.0);
    }

    #[test]
    fn align_stretch_vs_center_sets_cross() {
        let children = [fixed_basis(1, 50.0)];
        let constraints = BoxConstraints::tight(Size::new(200.0, 100.0));

        // Stretch → item cross fills the container, at cross start 0.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::new(vec![Size::new(50.0, 40.0)]),
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(out.child_boxes()[0].size(), Size::new(50.0, 100.0));

        // Center → measured cross 40, centred: (100 - 40)/2 = 30.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::Center,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::new(vec![Size::new(50.0, 40.0)]),
        );
        approx(out.child_boxes()[0].origin().y, 30.0);
        approx(out.child_boxes()[0].size().height, 40.0);
    }

    #[test]
    fn column_direction_swaps_axes() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 200.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        // Items stack vertically; cross (width) stretched to 100.
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[0].size(), Size::new(100.0, 50.0));
        assert_eq!(boxes[1].origin(), Point::new(0.0, 50.0));
        assert_eq!(boxes[1].size(), Size::new(100.0, 50.0));
    }

    #[test]
    fn container_min_height_floors_its_own_box() {
        // A flex column whose content is 50 tall, under loose constraints, is
        // floored to `min-height: 300` — the §4.2 clamp flex used to drop for the
        // container's own box (it applied min/max only to items).
        let mut style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        style.min_height = Dimension::Px(300.0);
        let children = [fixed_basis(1, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut FixedChildMeasurer::default(),
        );
        assert_eq!(out.size().height, 300.0);
    }

    #[test]
    fn gap_is_honored() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            10.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        approx(boxes[0].origin().x, 0.0);
        // 50 (item0) + 10 (gap) = 60.
        approx(boxes[1].origin().x, 60.0);
    }

    #[test]
    fn row_reverse_reverses_order() {
        let style = container(
            FlexDirection::RowReverse,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        // Last child takes the leading slot; first child follows.
        approx(boxes[1].origin().x, 0.0);
        approx(boxes[0].origin().x, 50.0);
    }

    #[test]
    fn explicit_row_width_bounds_main_axis() {
        // A `width: 200px` row under a 500px available extent packs items against
        // its OWN 200px, not the 500px available.
        let mut style = container(
            FlexDirection::Row,
            JustifyContent::FlexEnd,
            AlignItems::FlexStart,
            0.0,
        );
        style.width = Dimension::Px(200.0);
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        // Container is its authored 200px, not the 500px available.
        approx(out.size().width, 200.0);
        let b = out.child_boxes();
        // justify flex-end: remaining = 200 - 100 = 100, so items pack to x=100/150.
        approx(b[0].origin().x, 100.0);
        approx(b[1].origin().x, 150.0);
    }

    #[test]
    fn explicit_column_height_bounds_main_axis() {
        // A `height: 200px` column under a 500px available extent packs items
        // against its OWN 200px along the (vertical) main axis.
        let mut style = container(
            FlexDirection::Column,
            JustifyContent::FlexEnd,
            AlignItems::FlexStart,
            0.0,
        );
        style.height = Dimension::Px(200.0);
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 500.0)),
            &mut FixedChildMeasurer::default(),
        );
        approx(out.size().height, 200.0);
        let b = out.child_boxes();
        // flex-end along the vertical main axis: items pack to y=100/150 within 200.
        approx(b[0].origin().y, 100.0);
        approx(b[1].origin().y, 150.0);
    }

    #[test]
    fn auto_width_fills_available_regression() {
        // With no explicit width, the row still fills the available extent (500),
        // preserving the fill-available default.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexEnd,
            AlignItems::FlexStart,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        approx(out.size().width, 500.0);
        let b = out.child_boxes();
        // flex-end within the full 500: remaining = 500 - 100 = 400.
        approx(b[0].origin().x, 400.0);
        approx(b[1].origin().x, 450.0);
    }

    #[test]
    fn border_box_centers_child_in_padded_toolbar() {
        // Regression (the "chips pressed to the bottom" toolbar bug): a
        // `height: 52; padding: 8; align-items: center` flex row with
        // `box-sizing: border-box` and a 28px-tall child. The parent arranges the
        // container tight at its authored (border-box) 52px, so the container's
        // own layout must derive its content region as `52 - 16 = 36` and centre
        // the child within it — NOT centre against the full 52 and then overflow
        // the bottom when the box is clamped back to 52.
        let style = LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            height: Dimension::Px(52.0),
            padding: Insets::uniform(8.0),
            box_sizing: BoxSizing::BorderBox,
            ..LayoutStyle::default()
        };
        let children = [fixed_basis(1, 96.0)];
        // The child measures 28px tall on the cross axis.
        let out = FlexLayout.layout(
            &style,
            &children,
            // The parent arranges an explicitly-sized flex item tight at its
            // authored border-box size (52px tall here).
            BoxConstraints::tight(Size::new(300.0, 52.0)),
            &mut FixedChildMeasurer::new(vec![Size::new(96.0, 28.0)]),
        );
        // The container's own border box is exactly the authored 52 (no phantom
        // inflation to 68, no clamp mismatch).
        approx(out.size().height, 52.0);
        let child = out.child_boxes()[0];
        // Content region is `padding.top .. padding.top + 36` = 8..44; the 28px
        // child is centred there: top = 8 + (36 - 28) / 2 = 12.
        approx(child.origin().y, 12.0);
        approx(child.size().height, 28.0);
        // Equal gap above and below (12 - 8 == 44 - 40 == 4), and no overflow of
        // the container's border box.
        let gap_above = child.origin().y - style.padding.top;
        let content_bottom = style.padding.top + 36.0;
        let gap_below = content_bottom - (child.origin().y + child.size().height);
        approx(gap_above, gap_below);
        assert!(
            child.origin().y + child.size().height <= out.size().height,
            "child overflows the container's border box",
        );
    }

    #[test]
    fn content_box_default_keeps_authored_content_extent() {
        // The CSS default (`content-box`) is unchanged: an explicit `height: 52`
        // with `padding: 8` treats 52 as the CONTENT extent, so under a loose
        // constraint the border box grows to `52 + 16 = 68` (no subtraction).
        let style = LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            height: Dimension::Px(52.0),
            padding: Insets::uniform(8.0),
            // box_sizing defaults to ContentBox.
            ..LayoutStyle::default()
        };
        let children = [fixed_basis(1, 96.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut FixedChildMeasurer::new(vec![Size::new(96.0, 28.0)]),
        );
        // Content extent 52 + border/padding 16 = 68.
        approx(out.size().height, 68.0);
    }

    /// A wrap container of `width: 100` with three 40-wide items breaks into two
    /// lines (2 items then 1); the second line sits below the first (cross offset
    /// = line-1 height + gap) and the container cross size is the sum of the two
    /// line heights.
    #[test]
    fn flex_wrap_breaks_row_into_two_lines() {
        let style = LayoutStyle {
            width: Dimension::Px(100.0),
            flex_wrap: FlexWrap::Wrap,
            ..container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            )
        };
        let children = [
            fixed_basis(1, 40.0),
            fixed_basis(2, 40.0),
            fixed_basis(3, 40.0),
        ];
        // Each item measures 30px tall on the cross axis.
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Line 1: items 0 and 1 side by side at y = 0.
        approx(b[0].origin().x, 0.0);
        approx(b[0].origin().y, 0.0);
        approx(b[1].origin().x, 40.0);
        approx(b[1].origin().y, 0.0);
        // Line 2: item 2 below line 1 (cross offset = line-1 height 30 + gap 0).
        approx(b[2].origin().x, 0.0);
        approx(b[2].origin().y, 30.0);
        // Item cross sizes are the measured 30.
        approx(b[0].size().height, 30.0);
        approx(b[2].size().height, 30.0);
        // Container grew on the (auto) cross axis to the sum of the two line
        // heights: 30 + 30 = 60.
        approx(out.size().height, 60.0);
        approx(out.size().width, 100.0);
    }

    /// `flex-wrap: nowrap` (the default) keeps all three items on one line, packed
    /// against the main axis (overflowing the container), byte-identical to the
    /// historical single-line path.
    #[test]
    fn nowrap_keeps_all_items_on_one_line() {
        let style = LayoutStyle {
            width: Dimension::Px(100.0),
            flex_wrap: FlexWrap::NoWrap,
            ..container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            )
        };
        let children = [
            fixed_basis(1, 40.0),
            fixed_basis(2, 40.0),
            fixed_basis(3, 40.0),
        ];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // All three share y = 0, laid end to end (40, 80) — no wrapping.
        approx(b[0].origin().y, 0.0);
        approx(b[1].origin().y, 0.0);
        approx(b[2].origin().y, 0.0);
        approx(b[0].origin().x, 0.0);
        approx(b[1].origin().x, 40.0);
        approx(b[2].origin().x, 80.0);
    }

    /// An item wider than the container occupies its own flex line.
    #[test]
    fn wrap_puts_oversized_item_on_its_own_line() {
        let style = LayoutStyle {
            width: Dimension::Px(100.0),
            flex_wrap: FlexWrap::Wrap,
            ..container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            )
        };
        // Item 1 (150px) is wider than the 100px container.
        let children = [fixed_basis(1, 40.0), fixed_basis(2, 150.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(40.0, 30.0), Size::new(150.0, 30.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Item 0 on line 1 at y = 0; the oversized item 1 is pushed to its own
        // line 2 (below line 1's 30px height).
        approx(b[0].origin().y, 0.0);
        approx(b[1].origin().y, 30.0);
        approx(b[1].size().width, 150.0);
    }

    /// `flex-wrap: wrap-reverse` stacks the lines in reverse cross-axis order: the
    /// last line leads at the cross start.
    #[test]
    fn wrap_reverse_reverses_line_order() {
        let style = LayoutStyle {
            width: Dimension::Px(100.0),
            flex_wrap: FlexWrap::WrapReverse,
            ..container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            )
        };
        let children = [
            fixed_basis(1, 40.0),
            fixed_basis(2, 40.0),
            fixed_basis(3, 40.0),
        ];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Reversed: the line holding item 2 leads at y = 0; items 0 and 1 follow
        // on the second-placed line at y = 30.
        approx(b[2].origin().y, 0.0);
        approx(b[0].origin().y, 30.0);
        approx(b[1].origin().y, 30.0);
    }

    #[test]
    fn column_auto_height_hugs_content_under_loose_constraint() {
        // A flex COLUMN with no explicit height, measured under a LOOSE
        // (indefinite) height, sizes its main axis to the stacked children + gaps
        // (shrink-to-fit) — not the full available height. This is the fix for a
        // nested flex-column item over-reporting its height when a parent flex row
        // cross-measures it (the wireframe "cramping").
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            8.0,
        );
        let children = [fixed_basis(1, 40.0), fixed_basis(2, 14.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut FixedChildMeasurer::default(),
        );
        // 40 + gap 8 + 14 = 62, NOT the 1000px available height.
        approx(out.size().height, 62.0);
        // Under a TIGHT (definite) height the column fills as before: unchanged.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(300.0, 1000.0)),
            &mut FixedChildMeasurer::default(),
        );
        approx(out.size().height, 1000.0);
    }

    #[test]
    fn row_auto_height_hugs_tallest_item_under_loose_constraint() {
        // A flex ROW with no explicit height, measured under a LOOSE height, hugs
        // its tallest item's cross size instead of filling the available height.
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 40.0), Size::new(50.0, 24.0)]);
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            ),
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        // Tallest item is 40, so the row hugs 40 — not the 1000px available.
        approx(out.size().height, 40.0);

        // With `align-items: stretch` the auto row height still hugs the tallest
        // item's HYPOTHETICAL (pre-stretch) cross (40), exercising the extra
        // measurement path; items themselves stretch to fill on the definite pass.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 40.0), Size::new(50.0, 24.0)]);
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        approx(out.size().height, 40.0);
    }

    /// §9.7 `Σfactors < 1` clamp: a lone item with `flex-grow: 0.5` in a container
    /// with free space `F` grows by exactly `F · 0.5`, leaving the line half-empty.
    /// The old single-pass split gave the sole grow item ALL of `F` (wrong).
    #[test]
    fn lone_flex_grow_half_consumes_half_free_space() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 100.0);
        a.style.flex_grow = 0.5;
        let children = [a];
        // Container main = 200, base = 100, free = 100. grow 0.5 → +50, not +100.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].size().width, 150.0);
    }

    /// §9.7 min/max freeze + redistribute (grow): three equal-grow items where one
    /// caps at a `max-width` smaller than its share. The capped item freezes at its
    /// max and the OTHERS absorb the freed space, so the line still exactly fills.
    /// The old single-pass clamped once and left the line under-filled.
    #[test]
    fn grow_max_clamp_redistributes_to_others() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mk = |raw| {
            let mut c = fixed_basis(raw, 0.0);
            c.style.flex_grow = 1.0;
            c
        };
        let mut a = mk(1);
        a.style.max_width = Dimension::Px(50.0);
        let children = [a, mk(2), mk(3)];
        // Container = 300, bases 0, even share = 100. Item0 caps at 50; its 50 of
        // freed space is redistributed 1:1 to items 1 & 2 → 125 each. Sum = 300.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(300.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let b = out.child_boxes();
        approx(b[0].size().width, 50.0);
        approx(b[1].size().width, 125.0);
        approx(b[2].size().width, 125.0);
        // Line exactly fills: last item ends at the container main extent.
        approx(b[2].origin().x + b[2].size().width, 300.0);
    }

    /// §9.7 min/max freeze + redistribute (shrink): three equal-shrink items where
    /// one hits a `min-width` larger than its shrunk share. It freezes at its min
    /// and the others take up the remaining overflow, so the line exactly fills.
    #[test]
    fn shrink_min_clamp_redistributes_to_others() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mk = |raw| {
            let mut c = fixed_basis(raw, 50.0);
            c.style.flex_shrink = 1.0;
            c
        };
        let mut a = mk(1);
        a.style.min_width = Dimension::Px(45.0);
        let children = [a, mk(2), mk(3)];
        // Container = 100, bases 50 (Σ=150, overflow 50). Even shrink → 33.33 each,
        // but item0 floors at min 45. It freezes at 45; the remaining 45 of the
        // container is split by scaled shrink between items 1 & 2 → 27.5 each.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut FixedChildMeasurer::default(),
        );
        let b = out.child_boxes();
        approx(b[0].size().width, 45.0);
        approx(b[1].size().width, 27.5);
        approx(b[2].size().width, 27.5);
        approx(b[2].origin().x + b[2].size().width, 100.0);
    }

    /// A basic `flex: 1 1 auto` three-item even split (no clamping, factors ≥ 1) is
    /// unchanged by the freeze loop: the free space splits evenly.
    #[test]
    fn even_split_three_items_unchanged() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mk = |raw| {
            let mut c = fixed_basis(raw, 50.0);
            c.style.flex_grow = 1.0;
            c.style.flex_shrink = 1.0;
            c
        };
        let children = [mk(1), mk(2), mk(3)];
        // Container = 300, bases 50 (Σ=150), free = 150, +50 each → 100 each.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(300.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let b = out.child_boxes();
        approx(b[0].size().width, 100.0);
        approx(b[1].size().width, 100.0);
        approx(b[2].size().width, 100.0);
        approx(b[0].origin().x, 0.0);
        approx(b[1].origin().x, 100.0);
        approx(b[2].origin().x, 200.0);
    }

    /// A default flex item (auto flex-basis, `flex-shrink: 1`, `min: auto`) whose
    /// content size is supplied by the measurer.
    fn auto_item(raw: u64) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(raw),
            LayoutStyle {
                display: LayoutDisplay::Flex,
                ..LayoutStyle::default()
            },
        )
    }

    /// §4.5 automatic minimum size: a flex COLUMN of auto-basis items in a
    /// container SHORTER than their stacked content keeps each item at its
    /// content-based minimum (its measured content height) instead of shrinking it
    /// toward `0`; the content overflows the container. Without §4.5 the default
    /// `flex-shrink: 1` items would collapse below their content.
    #[test]
    fn column_auto_min_keeps_content_size_under_shrink() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let children = [auto_item(1), auto_item(2)];
        // Each item's content is 60 × 40; the container main (height) is only 50,
        // shorter than the stacked 40 + 40 = 80.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(60.0, 40.0), Size::new(60.0, 40.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Each item keeps its 40px content height (floored by §4.5), NOT shrunk.
        approx(b[0].size().height, 40.0);
        approx(b[1].size().height, 40.0);
        // The second item sits below the first — the content overflows the 50px box.
        approx(b[0].origin().y, 0.0);
        approx(b[1].origin().y, 40.0);
    }

    /// An EXPLICIT `min-height: 0` opts out of §4.5: the automatic minimum applies
    /// only to an `auto`/unset main-axis min. With `min-height: 0` the items shrink
    /// below their content and share the undersized container exactly (25 each),
    /// proving the auto-min path is gated on `auto` and does not affect an explicit
    /// min.
    #[test]
    fn explicit_min_zero_still_shrinks_below_content() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let mk = |raw| {
            LayoutChild::new(
                NodeId::from_raw(raw),
                LayoutStyle {
                    display: LayoutDisplay::Flex,
                    min_height: Dimension::Px(0.0),
                    ..LayoutStyle::default()
                },
            )
        };
        let children = [mk(1), mk(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(60.0, 40.0), Size::new(60.0, 40.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // base 40 each, overflow 30 shared evenly → 25 each; the line exactly fills.
        approx(b[0].size().height, 25.0);
        approx(b[1].size().height, 25.0);
        approx(b[1].origin().y, 25.0);
    }

    /// §9.2 flex base = max-content for an auto-basis item, protected by §4.5: a
    /// bare max-content item beside a fill-available sibling in a row where the two
    /// together overflow keeps its FULL max-content width instead of being shrunk
    /// below its content (the select-header "One" → "On" clip). The fill sibling
    /// absorbs the shrink; the content item does not clip.
    #[test]
    fn auto_basis_item_keeps_max_content_beside_fill_sibling() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::SpaceBetween,
            AlignItems::Center,
            10.0,
        );
        let children = [auto_item(1), auto_item(2)];
        // Item 0 is a 28-wide content item (the label); item 1 has an unbounded
        // natural width, so under a bounded row it FILLS the row and reports an
        // unbounded max-content (a flex/flow container with auto width — the
        // disclosure arrow), creating shrink pressure just like `.select-header`.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(28.0, 14.0), Size::new(f32::INFINITY, 14.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 40.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // The content item keeps its full 28px max-content width — no clip. The fill
        // sibling (no automatic floor, since its max-content is unbounded) absorbs
        // ALL the shrink, so the line fits the 200px row exactly instead of
        // overflowing: 28 + gap 10 + sibling 162 = 200.
        approx(b[0].size().width, 28.0);
        approx(b[0].origin().x, 0.0);
        approx(b[1].size().width, 162.0);
        assert!(
            b[1].size().width < 200.0,
            "the fill sibling shrinks to make room for the content item (no overflow)",
        );
    }

    #[test]
    fn explicit_child_cross_size_wins_over_stretch() {
        // `align-items: stretch` normally fills the cross axis, but an author-set
        // cross size on the child overrides the stretch.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 50.0);
        a.style.height = Dimension::Px(30.0);
        let children = [a];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        // Cross (height) is the explicit 30, not the stretched 100.
        approx(out.child_boxes()[0].size().height, 30.0);
    }

    /// A [`ChildMeasurer`] that returns fixed natural sizes (clamped into the
    /// requested constraints, like [`FixedChildMeasurer`]) and TALLIES every
    /// [`measure_child`](ChildMeasurer::measure_child) call, so a test can prove the
    /// §4.5 content measure runs only when it can change the result.
    struct CountingMeasurer {
        sizes: Vec<Size>,
        calls: usize,
    }

    impl CountingMeasurer {
        fn new(sizes: Vec<Size>) -> Self {
            Self { sizes, calls: 0 }
        }
    }

    impl crate::algorithm::ChildMeasurer for CountingMeasurer {
        fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size {
            self.calls += 1;
            constraints.constrain(self.sizes.get(index).copied().unwrap_or(Size::ZERO))
        }
    }

    /// Perf regression guard: in the common column case (a tall-enough container
    /// whose auto-basis items are NOT clamped, and no shrink pressure) the §4.5
    /// content floor is derived from each item's flex base with ZERO extra measures.
    /// Only the per-item BASE measure runs — one call per item, not two.
    #[test]
    fn column_auto_min_takes_no_extra_measure_when_not_clamped() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [auto_item(1), auto_item(2), auto_item(3)];
        // Each item is 60 × 40; the container main (height) is 500 — comfortably
        // taller than the stacked 120, so no item is clamped and the line grows.
        let mut measurer = CountingMeasurer::new(vec![
            Size::new(60.0, 40.0),
            Size::new(60.0, 40.0),
            Size::new(60.0, 40.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 500.0)),
            &mut measurer,
        );
        // Geometry is unchanged: items keep their 40px content height, stacked.
        let b = out.child_boxes();
        approx(b[0].size().height, 40.0);
        approx(b[2].origin().y, 80.0);
        // Exactly ONE measure per item (the flex-base measure) — the §4.5
        // content-floor measure is elided (pre-fix this was TWO per item).
        assert_eq!(
            measurer.calls, 3,
            "no extra §4.5 content measure in the common case"
        );
    }

    /// The card-collapse §4.5 fix is preserved with its extra measure taken ONLY for
    /// the clamped item: a column item taller than an undersized container keeps its
    /// natural (max-content) height, and exactly one extra content measure is spent
    /// to discover it.
    #[test]
    fn column_auto_min_measures_only_the_clamped_item() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [auto_item(1), auto_item(2)];
        // Item 0 is 60 × 80 — TALLER than the 50px container main, so its flex base
        // clamps to 50 and the natural 80 must be measured (card collapse). Item 1
        // is 60 × 20 — fits, so it is NOT measured for a floor.
        let mut measurer =
            CountingMeasurer::new(vec![Size::new(60.0, 80.0), Size::new(60.0, 20.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Item 0 keeps its natural 80 (floored by §4.5 above the clamped base);
        // item 1 keeps its 20. Content overflows the 50px box.
        approx(b[0].size().height, 80.0);
        approx(b[1].size().height, 20.0);
        approx(b[1].origin().y, 80.0);
        // Two base measures + one content measure for the single clamped item = 3
        // (NOT 4 — the unclamped item spends no content measure).
        assert_eq!(
            measurer.calls, 3,
            "only the clamped item spends a §4.5 measure"
        );
    }

    // ---- intrinsic flex-basis / main size (audit Tier-1 §4-D) --------------

    /// An item that behaves like a paragraph: it fills the inline space it is
    /// offered, but never below `min` (its longest unbreakable word) and never
    /// above `max` (its whole content on one line). Unlike
    /// [`FixedChildMeasurer`] it ANSWERS the offer, which is the only way a probe
    /// can be caught not probing.
    struct Paragraph {
        min: f32,
        max: f32,
    }

    impl crate::algorithm::ChildMeasurer for Paragraph {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            Size::new(constraints.max.width.clamp(self.min, self.max), 10.0)
        }
    }

    /// A row of one item with the given main-axis style, in a `line` wide
    /// container: the item's arranged width. Neither grow nor shrink, so what
    /// comes back IS the flex base size.
    fn base_width_of(main: LayoutStyle, line: f32, measurer: &mut dyn ChildMeasurer) -> f32 {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let children = [LayoutChild::new(NodeId::from_raw(1), main)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(line, 50.0)),
            measurer,
        );
        out.child_boxes()[0].size().width
    }

    /// An item with the given `flex-basis`, rigid on both flex factors.
    fn rigid_basis(basis: Dimension) -> LayoutStyle {
        LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_basis: basis,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..LayoutStyle::default()
        }
    }

    #[test]
    fn flex_basis_max_content_measures_rather_than_degrading_to_auto() {
        // A 100px row holding an item whose content wants 200 on one line.
        // `flex-basis: max-content` is 200 and overflows; `auto` would have taken
        // the 100 it was offered — which is what the keyword silently did before,
        // because `Dimension::resolve` answers `None` for it exactly as for `auto`.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        approx(
            base_width_of(rigid_basis(Dimension::MaxContent), 100.0, &mut m),
            200.0,
        );
        approx(
            base_width_of(rigid_basis(Dimension::Auto), 100.0, &mut m),
            100.0,
        );
    }

    #[test]
    fn flex_basis_min_content_measures_rather_than_degrading_to_auto() {
        // The same 100px row: `min-content` is the 30-wide unbreakable run, so the
        // item shrinks to its widest word instead of filling the line.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        approx(
            base_width_of(rigid_basis(Dimension::MinContent), 100.0, &mut m),
            30.0,
        );
    }

    #[test]
    fn an_intrinsic_width_feeds_the_flex_base_when_flex_basis_is_auto() {
        // §9.2 step 3: with `flex-basis: auto` the item's main-axis SIZE supplies
        // the base — including when it is a keyword, which used to fall through to
        // the natural-size measure.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        let style = LayoutStyle {
            display: LayoutDisplay::Flex,
            width: Dimension::MinContent,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..LayoutStyle::default()
        };
        approx(base_width_of(style, 100.0, &mut m), 30.0);
    }

    #[test]
    fn fit_content_basis_shrinks_a_fill_available_item_to_its_content() {
        // The item is a nested `width: auto` block: it fills whatever it is
        // offered, and reports its own 200-wide content only when the offer is
        // unbounded. `fit-content` = min(max-content 200, max(min-content 0,
        // stretch 400)) = 200 — the content. `auto` fills the 400px row, which is
        // the answer `fit-content` degraded to before.
        struct FillAvailable;
        impl crate::algorithm::ChildMeasurer for FillAvailable {
            fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
                let width = if constraints.max.width.is_finite() {
                    constraints.max.width
                } else {
                    200.0
                };
                Size::new(width, 10.0)
            }
        }
        approx(
            base_width_of(
                rigid_basis(Dimension::FitContent),
                400.0,
                &mut FillAvailable,
            ),
            200.0,
        );
        approx(
            base_width_of(rigid_basis(Dimension::Auto), 400.0, &mut FillAvailable),
            400.0,
        );
    }

    #[test]
    fn an_intrinsic_basis_costs_no_measure_for_an_item_that_authored_none() {
        // The guard on the cost of the extra probes: an ordinary auto-basis item
        // in a stretching row (which needs no cross measure either) still spends
        // exactly one measure, so the keywords are paid for only by the items that
        // author them.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [auto_item(1), auto_item(2)];
        let mut measurer =
            CountingMeasurer::new(vec![Size::new(30.0, 10.0), Size::new(30.0, 10.0)]);
        FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(400.0, 50.0)),
            &mut measurer,
        );
        assert_eq!(measurer.calls, 2, "one base measure per item, no probes");
    }
}
