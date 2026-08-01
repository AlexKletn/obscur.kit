#![doc = include_str!("resolve_flexible_lengths.doc/_module.md")]

use super::LayoutChild;
use super::flex_axis_bounds::clamp_floor;

#[doc = include_str!("resolve_flexible_lengths.doc/resolve_flexible_lengths.md")]
// These are cohesive positional inputs to one internal freeze loop; grouping them
// into a struct would not aid clarity.
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_flexible_lengths(
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

        // An **unbounded** main axis carries no free space to distribute. It is
        // the `max-content` *question* — "how big would you like to be?" — not a
        // decision, and §9.7 resolves flexible lengths against a used main size
        // that is definite. Every item therefore keeps its flex base size, which
        // is what `max-content` means for a flex line.
        //
        // Leaving this to the loop below is how a squeezed window hung the entire
        // application: the item grew toward `inf`, was clamped against an `inf`
        // maximum, and its violation came out `inf − inf = NaN`. NaN compares
        // false against `== 0`, `> 0` and `< 0` alike, so the sign test froze
        // *nothing*, the state repeated exactly, and the frame never returned —
        // taking the event loop with it, which is why the window could not even be
        // closed.
        if !remaining.is_finite() {
            for pos in 0..n {
                if !frozen[pos] {
                    let i = line[pos];
                    target[pos] = clamp_floor(flex_base[i], main_min[i], main_max[i]);
                    frozen[pos] = true;
                }
            }
            continue;
        }

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
        let before = frozen.iter().filter(|&&f| f).count();
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

        // Termination is a *property of this loop*, not of the arithmetic that
        // feeds it. Every pass must freeze at least one item, or the next pass
        // recomputes the identical state and the loop never ends — and a layout
        // that never ends does not merely draw wrongly, it takes the frame, the
        // event loop and the window's close button with it.
        //
        // The sign test above can freeze nothing whenever a violation is not an
        // ordinary number: NaN is false against every comparison. The guard above
        // removes the one source of that we know of; this makes the loop safe
        // against the ones we do not.
        if frozen.iter().filter(|&&f| f).count() == before {
            frozen.fill(true);
        }
    }

    target
}
