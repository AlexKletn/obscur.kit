# Servo / Stylo / Taffy Algorithm Catalog → obscur.kit

- **Date:** 2026-07-08
- **Purpose:** algorithms/formulas mined from the Servo browser engine (`~/projects/aliens/servo`), its `stylo` style dep (pinned rev `0151ab3…`), and the `taffy` layout crate, mapped to obscur.kit so we can **validate** what we built, **fix** approximations, and **plan** missing features.
- **Method:** 6 parallel read-only research agents, one per subsystem, each cross-referencing our code.
- **License:** Servo/stylo/taffy are **MPL-2.0**. Everything below is a *summarized algorithm/formula* — reimplement from the algorithm + the cited CSS spec; do **not** copy source verbatim.
- **Key structural finding:** Servo computes CSS→display-list **geometry/layout** in-tree and delegates all **rasterization** (border stroking, gaussian blur, gradient/filter rasterization, nine-patch) to **WebRender** (a crates.io dep, *not* in the checkout). So Servo's tree validates our geometry + layout; for render *effects* the reference is the CSS spec (and WR primitive names, noted where relevant). Grid is delegated to **taffy**; the style system to **stylo**.
- **Legend:** `[HAVE]` we do this — validate against the reference · `[IMPROVE]` we approximate — this is the exact version · `[MISSING]` we lack it.

---

## 0. Master priority

### 0.1 Concrete correctness bugs found (fix first — small, high-confidence)
| # | Bug | Where in our code | Fix |
|---|---|---|---|
| B1 | **Border-radius overlapping-radius clamp is wrong** — we clamp each corner independently to `min(w,h)/2`; the spec applies ONE uniform scale `f = min(side / (sum of the two radii on that side))` to **all four** corners. Overflows / over-shrinks on non-square boxes with asymmetric radii. | `path.rs::rounded_rect`, `scene_coordinator.rs::corner_radii_of` | Add a `normalize_radii(rect, &mut CornerRadii)` §5.5 pass at radius-resolution time; feed normalized radii to fill/clip/border/shadow. (§C-1) |
| B2 | **Stacking is FLAT, not the §9.9 7-layer paint order** — a positioned `z:auto/0` element must paint ABOVE non-positioned block siblings; our `(z, tree_index)` sort gets sibling/positioned ordering wrong; no block/float/inline/positioned separation. | `orchestration/stacking.rs` | Replace the sort key with a **layer rank** `{neg_z, block, float, inline, z0_positioned, pos_z}` primary key; PaintUnit machinery stays. (§E-1) |
| B3 | **`is_positioned` omits `sticky`** (sticky doesn't establish a stacking level / CB). | `stacking.rs::is_positioned` | Add `sticky` (and ideally opacity<1/transform/filter as SC roots). (§E-2) |
| B4 | **box-shadow spread reuses the box radii** — should grow (outset) / shrink (inset) each corner by `±spread`, sharp (0) corners stay sharp. | `scene_coordinator.rs::emit_box_shadow`, cpu `draw_shadow_approx` | `offset_radii(radii, ±spread)`. (§C-2) |
| B5 | **inset box-shadow uses border-box + box radii** — should use the **padding box** with radii shrunk by border width. | `emit_box_shadow` | padding rect + `offset_radii(radii, −border_width)`. (§C-3) |
| B6 | **sticky % insets resolve against the CB** — spec resolves against the **scrollport / scroll-frame** size. | `inset.rs::sticky_offset` | Change the percentage basis to the scrollport. (§E-4) |
| B7 | **dashed period is fixed `[3w,3w]`, not fitted to the side** (dashes don't meet corners); **dotted folds into dashed** (square dashes, not round dots). | `scene_coordinator.rs::dashed_border_pattern` | Fit `period = side/round(side/desired)`; give dotted `[w,w]` + round caps. (§C-13/14) |

### 0.2 High-value improvements (validated need, clear how-to)
1. **Sizing substrate** (§A-1..4): adopt Servo's `Size<T>` keyword enum + `Sizes{preferred,min,max}` + `ContentSizes{min,max}` pair + `SizeConstraint(Definite|MinMax)` with ONE shared `resolve()` used by block/flex/grid. This is the structural pattern that would prevent whole classes of the flex bugs we just fixed, and unlocks real `min/max-content`.
2. **Flex automatic minimum size** (§B-8): items won't shrink below content-min in an auto track (the "min-width:auto" rule). Likely `[MISSING]` — high visual impact.
3. **Real min/max-content intrinsic sizing** (§A-2): a pair, not a single "measure" — needed for `fit-content`, `width:max-content`, nested/inline flex, grid.
4. **Grid two-value track model + span distribution** (§D-1..8): our single-pass one-value model is the source of most grid remnants (subgrid, stretch, fr-in-implicit, spanning-to-first-track). Refactor to `base_size + growth_limit` staged pipeline.
5. **Color: oklch/lab→sRGB exact** (§F-1..2) + **gradient interpolation in Oklab, premultiplied** (§F-4..5) + **operate in linear light** for blur/opacity/blend (§F-6). Replaces our oklch approximation; visible correctness.
6. **Text: letter-spacing/word-spacing** (§T-6/7, cheap post-shaping bumps), **text-transform** (§T-5), **white-space variants** (§T-3/4, collapse×wrap model), **per-font baseline in mixed runs** (§T-11), **tabs** (§T-12), **text-overflow: ellipsis** (§T-18, spec-driven — no Servo ref).
7. **Margin collapsing** (§B-19): the `{max_positive, min_negative}` reduction — elegant, easy to copy, hard to get right otherwise.
8. **Transform interpolation** (§G-8/9/10): 2D decompose+lerp, 3D unmatrix + quaternion Slerp, matched-prefix-then-matrix-fallback — needed for transform animations.

### 0.3 Validated CORRECT (no action — good news)
- **Linear/radial gradient geometry** matches Servo line-for-line (line length `|W·dir.x|+|H·dir.y|`, corner dir `(±H,±W).normalize()`, radial extent keywords, ellipse `·√2`). (§C-15/16) — confirms commit `ada933f`.
- **background-size cover/contain + background-position %** formulas match. (§C-8/9)
- **Grid placement, dense packing (reset-to-origin first-fit), auto-fill/fit, negative lines, Space* distribution** are spec-correct. (§D-10/12)
- **Transform/opacity/blur/clip paint brackets** ordering is Servo-aligned. (§E-11/12)
- **Relative-position offset** matches. (§E-5)

### 0.4 Larger deferred features (self-contained; do when needed)
- **Native `<table>`** (§D-tables): slot grid + colspan/rowspan + the auto-width distribution engine + border-collapse — the distribution engine is a pure `(measures[], target) → widths[]` function, very portable. Only if `display:grid` tables prove insufficient.
- **Subgrid** (§D-15): large; changes track sizing to a cross-container fixpoint. Do after the two-value refactor.
- **Floats + clearance** (§B-7/8): the band-tree model; low priority for a UI toolkit.
- **3D transforms** (§G-9): decomposition + Slerp; needed for `rotate3d`/`perspective`.
- **clip-path basic shapes** (§C-17), **border-image nine-patch** (§C-18): ready blueprints.

### 0.5 "Built-but-not-wired" flags found
- `obscur-tree/src/layer/stacking_context.rs` has a correct per-context `StackingContext{root,z_index}` + `paint_order` comparator that the flat `orchestration/stacking.rs` does **not** use (relevant to B2).
- `positioned_layout.rs::PositionedLayout` (margin-offset abspos, no right/bottom/inset) is superseded by the coordinator's `resolve_absolute_box` path — dead code.

---

## A. Sizing substrate (Servo `layout/sizing.rs`, `style_ext.rs`) — the foundational pattern

- **A-1 `Size<T>` keyword model + three-size resolution** `[IMPROVE]`. A size = `Initial(auto/none)|MinContent|MaxContent|FitContent|FitContentFn(T)|Stretch|Numeric(T)`. Used size = `preferred.clamp(min,max)` where each resolves: `auto`→context automatic size; `fit-content`→`available.min(max_content).max(min_content)` (note deliberate min-wins order); `stretch`→available−margins; `min:auto`→automatic minimum size. Spec css-sizing-3 §5–7, css-sizing-4. **Adapt:** one `resolve(pref,min,max, automatic, stretch, ||content_sizes)` shared by all layout modes.
- **A-2 `ContentSizes{min_content,max_content}` pair + Contribution-vs-Size** `[MISSING/IMPROVE]`. Two intrinsic numbers bottom-up: inline siblings **union** (`min=max(mins), max=Σmax`); block stacking **max**. *Contribution* = outer (adds PBM + own sizing props); *Size* = inner. `outer_inline()` is the canonical wrap. Spec css-sizing-3 §4.1/4.2. **Adapt:** the biggest gap — compute a real min/max pair, not one "measure."
- **A-3 `SizeConstraint = Definite(Au) | MinMax(min, max?)`** `[IMPROVE]`. An axis is Definite if preferred resolves to a number, else carries the range; percentages resolve only against Definite. Directly relevant to our fixed "auto-height fills under indefinite" bug — thread this instead of sentinels. Spec css-sizing-3 §3, flexbox §9.8.
- **A-4 `LazySize`** `[IMPROVE]`. Defer block-size resolution: hold `(Sizes, automatic, stretch)` + a `OnceCell`; `.resolve(||intrinsic)` computes `clamp` only when the content size is finally needed. Implements "auto height = max(explicit clamp, content)" once, no double layout.
- **A-21 box-sizing conversion** `[HAVE — validate]` (`style_ext.rs`): border-box → subtract `padding+border`; **min/max clamp result at 0**, but *preferred* may go negative (clamped later). Apply to explicit sizes AND flex-basis. **Block-axis padding/border % resolves against inline (CB width)** — classic error. Spec css-sizing-3 §5.3.
- **A-22 aspect-ratio dependent size** `[MISSING]`: ratio `i_over_b` + `box_sizing_adjustment` (=PB sums iff `aspect-ratio` no-`auto` + border-box). `inline = (block+adj.b)·ratio − adj.i`. Feeds flex base/min (case B). Spec css-sizing-4 §4.
- **A-24 cyclic/indefinite %** `[IMPROVE]`: preferred/max % against indefinite → treat as auto/none; min/pad/margin % against indefinite → 0 (asymmetric). Prevents %-height-in-auto-height loops.

---

## B. Flexbox (Servo `flexbox/layout.rs`) + block flow — validate `flex_layout.rs`

- **B-5 Resolve Flexible Lengths (freeze loop)** `[HAVE — validate details]`, §9.7. Direction from `Σ hypothetical < inner`. Freeze inflexible; loop: remaining free; **if Σ unfrozen factors < 1, multiply *initial* free space by that sum** and use if smaller; distribute (grow `base+free·f/Σf`; shrink by **scaled** factor `shrink·base`); clamp to `[min,max]`; freeze only sign-matched violators; repeat. **Verify:** (i) scaled-shrink weight uses inner flex-base; (ii) the Σf<1 clamp (over-distribution bug); (iii) sign-matched freezing.
- **B-6 Flex base size (cases A–E)** `[HAVE]`, §9.2. From `flex-basis`: `content`→max-content; `auto`→the main size prop; case B transferred (ratio·cross); then `apply_box_sizing` (border-box subtracts main PB, may go negative). Validates our box-sizing-on-flex-basis fix.
- **B-7 hypothetical main** = `flex_base.clamp(min,max)` (before line breaking).
- **B-8 Automatic minimum size (`min:auto`)** `[MISSING — high value]`, §4.5: min of specified-suggestion (definite pref), transferred (ratio), content-suggestion (min-content). Result `min(specified,content)` else content, clamped below max. This is "flex item won't shrink below content." Feeds the freeze-loop min.
- **B-9 line breaking** `[HAVE]`: greedy accumulate `outer_hypothetical + (gap if non-empty)`; ≥1 item per line. Verify gap only between items.
- **B-10/11 line cross size + stretch relayout** `[IMPROVE]`: track `max_ascent/max_descent` for baseline items; line cross = `max(max_outer_cross, ascent+descent)`. Stretch only when auto-cross ∧ align stretch ∧ no auto margins; **relayout item with definite cross if size depends on it** (percentage-height children).
- **B-12 auto margins absorb free space** `[MISSING]`: positive main free + auto main margins → each gets `free/n`, justify free→0. Cross auto margins similar. (`margin-left:auto` push idiom.)
- **B-13 justify/align-content fallback** `[HAVE — validate]`: `normal→stretch`; negative-free/single-subject fallbacks (`space-between→start`, `space-around/evenly→center`); intervals `between=free/(n−1)`, `around=free/n`, `evenly=free/(n+1)`; per-line remainder trick for rounding.
- **B-14 baseline placement** `[MISSING unless text baselines]`: cross offset baseline = `max_line_baseline − item_baseline`; synthesized baseline = border-box bottom.
- **B-15 container intrinsic main size** `[MISSING]`, §9.9: per item `desired_flex_fraction`, line max, container max-content = `Σ(base+f·fraction).clamp + gaps`. Needed for nested/inline flex widths.
- **B-16 definite/indefinite propagation** `[IMPROVE]`: main definite iff grid/flex-basis-definite/container-main-definite → decides %-children resolution.
- **B-17 block inline-size** `[HAVE]`: automatic = Stretch (normal blocks) vs FitContent (float/inline-block/table/replaced = `clamp(min-content, available, max-content)`).
- **B-18 block auto-height** `[HAVE]`: position after last child, not fill. Collapse-through propagation.
- **B-19 margin collapsing** `[MISSING/IMPROVE — copy this]`, §8.3.1: a collapsed margin = `{max_positive, min_negative}`; **adjoin** = `{max(pos), min(neg)}`; **solve** = `max_positive + min_negative` (`+30`∧`−20`→`10`). Parent-start collapses with first child start; collapse-through when nothing separates start/end margins; **BFC roots never collapse with children**; clearance suppresses collapse.
- **B-20 auto inline margins** `[HAVE — validate]`: `margin:0 auto` block centering; block-axis auto → 0 in flow.

---

## D. CSS Grid (taffy `compute/grid/`) — validate `grid_layout.rs`

Core gap: our single-pass **one-value-per-track** model vs the canonical **two-value (`base_size`+`growth_limit`) staged** model that distributes each spanning item across the tracks it spans. Most grid remnants trace here.

- **D-1 two-value track model** `[IMPROVE — foundational]`: `base_size` (floor, grows) + `growth_limit` (ceiling, may be ∞) + scratch increases. Prereq refactor.
- **D-2 initialize tracks**: base=`min_fn.definite.unwrap_or(0)`, growth=`max_fn.definite.unwrap_or(∞)`, floor growth by base; gutters→0. §11.4.
- **D-3 resolve intrinsic track sizes (span distribution)** `[IMPROVE — highest value]`, §11.5: items in **span batches**, span=1 fast path; span≥2 runs 5 sub-steps in order (intrinsic-min→content-min→max-content-min→intrinsic growth-limit→max-content growth-limit), each via D-4 across only matching-filter spanned tracks, flushing planned increases per step. Replaces our "attribute to first spanned track" (wrong for `auto auto`+colspan-2).
- **D-4 distribute extra space up to limits (+ beyond)** `[MISSING]`, §11.5.1: `extra=max(0, space−Σbase)`; grow equally, **freeze each at its limit** (find smallest headroom, raise all, repeat); remainder "beyond limits" to filtered subset. ~45 self-contained lines, reused by D-5.
- **D-5 maximise tracks** `[MISSING]`, §11.6: `free=available−Σbase`; if ∞ set base=growth; else distribute-up-to-limits(free, all). Lets `minmax(100,300)` grow to 300.
- **D-6 find-size-of-fr (with restart)** `[IMPROVE]`, §11.7.1: iteratively `fr=leftover/max(Σfactors,1)`; **demote any flex track with `factor·fr<base` to inflexible and restart**. Our fr is a single split with no restart → over-shrinks `minmax(200px,1fr) 1fr` in a narrow container.
- **D-7 stretch auto tracks** `[MISSING → the "stretch" remnant]`, §11.8: count auto-**max** tracks; if `free>0` add `free/n` to each base. Run this (not `content_distribution`) when justify/align-content is stretch/normal.
- **D-8 automatic minimum size of grid items** `[MISSING]`, §6.6: content-min only if spans an auto-**min** track and (if span>1) none flexible; else 0. Feeds D-3 step 1.
- **D-9 baseline shims** `[MISSING → baseline remnant]`: group by row-start line; `shim = row_max_baseline − item_baseline` applied as extra top margin. Needs text baseline plumbing.
- **D-10 placement** `[HAVE — validate]`: definite-both → secondary-definite → cursor (sparse advances, **dense resets to origin** = correct first-fit; our dense-worry was over-cautious). Fix: half-definite **column-flow** uses the primary-definite branch (we route through the row helper).
- **D-11 implicit tracks are real Vec entries** `[IMPROVE — falls out of D-1..7 free]`: materialize implicit tracks into the track vector (cycling `grid-auto-*`) so an `fr` implicit track participates in D-6.
- **D-12 auto-fill/auto-fit** `[HAVE — validate]`: definite/max → floor count; min → ceil; auto-fit collapses empty tracks (+ gutters merge). Our "fr/auto in auto-fill invalid → ~1" is spec-conformant.
- **D-13 track alignment + Stretch** `[IMPROVE]`: gutters as zero tracks; add `Stretch` (via D-7) and the safe/fallback rules.
- **D-14 area-implicit named lines / nth-line** `[MISSING — CSS-lowering only, cheap]`: each `grid-template-areas` area `foo` implicitly declares lines `foo-start`/`foo-end` on both axes; `grid-column: foo` (bare) → `foo-start`/`foo-end`; `grid-column: foo 2` → 2nd occurrence. Fix in the line-name table builder, independent of the sizing refactor.
- **D-15 subgrid** `[MISSING — large, defer]`, css-grid-2 §7: adopt parent tracks; subgrid items contribute intrinsics to the parent's tracks (cross-container fixpoint). Do after D-1..8.

### D-tables — native `<table>` (all `[MISSING]`; we only have grid-based tables)
- **Data model** (Servo `table/mod.rs`): `slots: Vec<Vec<TableSlot>>`, `Cell{colspan,rowspan}` / `Spanned(offset)` / `Empty`; build via `incoming_rowspans` (negative=infinite). Self-contained, port first.
- **Auto-width distribution engine** (`table/layout.rs`, "crown jewel"): `distribute_width_to_columns(target, [ColumnLayout]) → widths`. Build 4 guesses per column (min-content / min+%/ min+specified / max-content); if `target≥Σmax` distribute excess via a 6-level priority cascade; else linearly blend the two bracketing guesses, residue into col 0. **Constrained column caps at cell min-content** (matches Blink/Gecko). Pure data in/out; reused for colspan.
- **Fixed layout**: widths from first row + `<col>`, same distribution.
- **Row/rowspan block sizing**: row height = `max(max_cell_outer, max_ascent+max_descent)`; rowspan via a 5-level cascade; `vertical-align` Top/Bottom/Middle/Baseline.
- **border-collapse** (clean standalone port): `CollapsedBorder` comparator = **hidden > wider > style-priority (double>solid>dashed>dotted>ridge>outset>groove>inset>none)**; origin precedence (cell>row>rowgroup>col>colgroup>table) encoded as application order; each cell owns **half** the border.

---

## C. Display-list geometry (Servo `layout/display_list/`)

- **C-1 border-radius overlapping-radius clamp** `[IMPROVE → bug B1]`, css-backgrounds §5.5: `f = min(W/(tl.w+tr.w), W/(bl.w+br.w), H/(tl.h+bl.h), H/(tr.h+br.h))`; if `f<1` multiply ALL four corners (both axes) by `f`, once, at resolution; reuse for all consumers. Also widen `CornerRadii` to per-corner `(x,y)` for elliptical corners.
- **C-2 shadow spread→radii** `[IMPROVE → B4]`: `offset_radii(r, ±spread)` (grow outset / shrink inset; 0 stays sharp).
- **C-3 inset shadow padding-box + shrunk radii** `[IMPROVE → B5]`: padding rect + `offset_radii(r, −border_width)`.
- **C-4 blur clip inflation** `[IMPROVE]`: bounds inflate by `spread + ceil(3·blur)` (WR `BLUR_SAMPLE_SCALE=3`, gaussian tail=3σ) so the soft edge isn't clipped.
- **C-5 `background-repeat: round`** `[MISSING]`: `tile' = area / round(area/tile).max(1)` per axis before positioning.
- **C-6 `background-repeat: space`** `[MISSING]`: `count=floor(area/tile)`; if ≥2 → position 0, `spacing=(area−tile·count)/(count−1)`; else no-repeat. Needs a `tile_spacing` field on `DrawImage`.
- **C-7 tile-grid origin** `[IMPROVE]`: first tile = `position − stride·ceil(offset/stride)` (step back so top-left is covered), `stride = tile + spacing`.
- **C-8/9 background-size cover/contain + position %** `[HAVE]`: matches (minor: both-auto/no-intrinsic → contain, we use intrinsic).
- **C-10 background-clip / background-origin** `[MISSING]`: two independent rects — clip box (from `background-clip`) vs positioning box (from `background-origin`); we only use the border box.
- **C-11 inner rounded-clip radii** `[HAVE — validate]`: padding-edge clip radii = `offset_radii(r, −border_width)`; content-edge = `−(border+padding)`. Ensure our overflow-clip uses shrunk radii, not outer.
- **C-12 clip-chain intersection** `[IMPROVE]`: nested rounded clips must **intersect** (parent-linked chain); skip non-rounded clips to a scissor-rect fast path.
- **C-13 dashed dash-length fitting** `[IMPROVE → B7]`: `n=round(side/desired).max(1)`, `period=side/n` so dashes meet corners.
- **C-14 dotted borders** `[MISSING → B7]`: pattern `[w,w]` with **round line caps**, period `round(side/(2w))`.
- **C-15 linear gradient geometry** `[HAVE]`: `len=|W·dir.x|+|H·dir.y|`, angle dir `(sin,−cos)`, corner `(±H,±W).normalize()`.
- **C-16 radial extent keywords + ellipse** `[HAVE]`: side=min/max distances, corner=hypot, ellipse corner `·√2`. (Matches `ada933f`.)
- **C-17 clip-path basic shapes** `[MISSING — blueprint]`: `inset()` rect+radii (inner-rect guard); `circle()` radius closest/farthest-side=`min/max`, corner=`hypot`, `<length>` against `basis=hypot(w,h)/√2`; `ellipse()` per-axis.
- **C-18 border-image nine-patch** `[MISSING — blueprint]`: outset (`Number→border·f`), width (`Number→border·x`, `auto→border`), slice (`%→round(p·img)`, integer device px); repeat modes stretch/repeat/round/space (WR does the tiling — we'd implement it).

---

## T. Inline / text layout (Servo `layout/flow/inline/`, `fonts/`)

- **T-1 break opportunities computed over the WHOLE paragraph string** `[HAVE — validate]` (UAX#14 via ICU), then sliced per run with a cursor + synthetic end-of-range break. Makes cross-run/span breaks correct.
- **T-2 break filtering by word-break/overflow-wrap** `[IMPROVE/MISSING]`: `break-all|anywhere|break-word` add anywhere-breaks; `keep-all` skips mid-word breaks; **separate trailing whitespace into its own sub-run per word** (enables trimming + justify counting).
- **T-3 white-space collapsing** `[MISSING]`, css-text §4.1.1: single-pass `char` iterator, 5 bits of state; collapse runs of spaces/tabs to one U+0020 emitted before a non-space; `\n` handling per mode; **do it before shaping**, carry leading/trailing-trim state across inline boxes.
- **T-4 white-space = collapse-mode × wrap-mode** `[MISSING]`: `normal=(collapse,wrap)`, `nowrap=(collapse,nowrap)`, `pre=(preserve,nowrap)`, `pre-wrap=(preserve,wrap)`, `pre-line=(preserve-breaks,wrap)`, `break-spaces=(break-spaces,wrap)`. Variants fall out of T-2/3.
- **T-5 text-transform** `[MISSING]`: upper/lower = streaming `char` map (pending buffer for 1→N like ß→SS); **capitalize** needs a Unicode word segmenter + cross-run boundary flag. Before shaping.
- **T-6 letter-spacing** `[MISSING]`: post-shaping `advance += spacing` on each **cluster's last glyph**; **skip Cf codepoints**; **disable for cursive scripts** (Arabic/Syriac/…) + turn off ligatures when nonzero.
- **T-7 word-spacing** `[MISSING]`: `advance += spacing` only for U+0020 and U+00A0.
- **T-8 justification** `[HAVE — validate]`: count word-separators; `adjustment = (available − indent − line_len)/opportunities` (≥0, off on last/forced line per `text-align-last`); apply the extra advance **at paint time** after each separator (keeps hit-test consistent). Servo is inter-word only.
- **T-9 text-align (+text-align-last)** `[HAVE — validate]`: `line_len = pos − trimmed − indent`; start→indent, end→`max(available−len, indent)`, center→`max((available−len+indent)/2, indent)`; last/br line overridden by `text-align-last`.
- **T-10 line-height / half-leading** `[HAVE — validate]`, css-inline-3: non-normal `L=line_height−(A+D)`, `A'=A+L/2`, `D'=line_height−A'`, from the **first-available-font** metrics (not tallest fallback); leading may be negative.
- **T-11 mixed-font baseline (strut model)** `[MISSING]`: per-run `(ascent,descent)` from its own font; **max component-wise** across the line; line box = `max(line_height, ascent+descent)`; each run placed at `line_baseline − run.font.ascent`; include a strut from the block's own font.
- **T-12 tab stops** `[MISSING]`: `\t` as its own item; `next = ceil((advance + 0.5ch)/tab_advance)·tab_advance − advance`; numeric `tab-size` unit = `space+word+letter spacing`, length form = the length.
- **T-13 bidi (UAX#9)** `[HAVE — validate]`: levels once over the paragraph (explicit base level); reorder **line items** at break time via `reorder_visual` on the per-item level array; extra whole-line reversal for RTL base; atomic inlines/tabs carry their level.
- **T-14 itemization + per-codepoint fallback** `[HAVE — validate]`: split on font/script/bidi change; **two-char lookahead** (emoji/regional-indicator); combining marks & joiners inherit the base run's font; Common/Inherited script absorb the neighbor.
- **T-15 line-edge whitespace trimming** `[IMPROVE]`: trailing (and leading) whitespace drained from line width for align/justify; preserved under pre/pre-wrap.
- **T-16 FontMetrics naming caveat** `[IMPROVE]`: Servo `line_gap` = **full line height**; `leading` = the gap. If we store OpenType `lineGap` (gap only), `normal` line-height = `ascent+descent+lineGap`. Store `space_advance` (tabs) + `zero_advance` (`ch`).
- **T-17 compact glyph storage** `[IMPROVE]`: carry per-run word-separator count (justify), `is_whitespace`/`ends_with_whitespace` (trim), and glyph→char `character_count` (caret/IME/selection through ligatures). `Arc<ShapedText>+range` slice reuse.
- **T-18 text-overflow: ellipsis** `[MISSING — no Servo ref, spec-driven]`: at line finalization, if overflowing single-line-clipped, walk slices from the visual end (post-bidi) removing advance until `remaining + ellipsis ≤ available`; shape '…' in the truncation-point font; don't split a grapheme cluster.

---

## E. Positioning / stacking (Servo `layout/flow/`, `display_list/`)

- **E-1 §9.9 7-layer paint order** `[MISSING → bug B2]`: per stacking context, sorted-child list consumed as: (1) SC bg/border; (2) `z<0`; (3) non-positioned block descendants; (4) non-positioned **floats** (deferred list); (5) inline-level; (6) `z==0/auto` positioned; (7) `z>0`. Outlines deferred to the end. **Adapt:** primary sort key = layer rank, not tree order.
- **E-2 stacking-context establishment** `[IMPROVE → B3]`: will-change / `z-index≠auto` on positioned-flex-grid / **`fixed|sticky` always** / transform / opacity<1 / filter / mix-blend / clip-path / isolation / root. Real SC (isolates z) vs container (positioned z:auto, **hoists** children into parent's z-space). Sort stable by z, tie=tree order. Our `is_positioned` misses sticky and only z-index establishes a level → no nesting isolation (a deep child's high z escapes). The unused `obscur-tree` `StackingContext` is the right vehicle.
- **E-3 abspos/fixed containing-block (three CBs)** `[IMPROVE]`: thread THREE CBs — non-absolute (content edge), absolute (padding edge of nearest `position≠static`), and **all-descendants (padding box of nearest transform/filter/root)**. We have the first two; **transform/filter ancestors must become the fixed/abspos CB** (add `establishes_cb_for_all_descendants = transform ∨ filter ∨ will-change`).
- **E-4 sticky** `[HAVE — fix B6]`: our clamp algebra matches; change % inset basis to the **scrollport** (not CB); per-container scroll is a larger follow-up.
- **E-5 relative offset** `[HAVE]`: `(auto,auto)→0`, `(auto,end)→−end`, `(start,_)→start`; pure post-layout translation.
- **E-6 static-position fallback** `[IMPROVE]`: when both insets auto, use the true in-flow **static origin** (we already compute it for sticky) minus `cb.origin`, not `cb.origin` (top-left).
- **E-7 abspos margin resolution / over-constrained** `[IMPROVE]`: inset-modified `available = max(0, cb − inset_sum)`; `(auto,auto)`→**center** (`free/2`), one-auto absorbs; inline axis avoids negative start margin; then `align-self`/`justify-self`. Gives `left:0;right:0;margin:auto` centering (centered modals).
- **E-11 reference frames (transforms)** `[HAVE brackets / IMPROVE CB]`: transform = new coord origin at border box (separate from SC grouping); non-invertible → not displayed; establishes CB for all descendants (link to E-3). Our PushTransform bracket is right; add CB-establishment + non-invertible cull.
- **E-12 opacity/filter as group** `[HAVE]`: Servo folds opacity into the filter pipeline `[...filters, Opacity(a)]` with blend as the group arg; we keep separate brackets (fine for our CPU/GPU split).
- **B-7/8 floats + clearance** `[MISSING — defer]`: `FloatContext` band-tree (`FloatBand{top, inline_start?, inline_end?}`), `place_object` walks bands down until `object_fits` (rules 3/7); clearance = `max float block-end − hypothetical top`, suppresses margin collapse.

---

## F. Color (stylo `color/`)

- **F-1 oklch→sRGB exact** `[IMPROVE — #1 color gap]`: **oklch→oklab** (`a=C·cosH, b=C·sinH`) → **oklab→XYZ** (`lms = OKLAB_TO_LMS·lab; lms=lms³; xyz = LMS_TO_XYZ·lms`) → **XYZ→linear-sRGB** (`Srgb::FROM_XYZ`) → **gamma** (`v>0.0031308 ? 1.055·v^(1/2.4)−0.055 : 12.92·v`). ~20 flops, exact. Matrices in the transcript / css-color-4 §17.
- **F-2 lab/lch→sRGB + D50↔D65 Bradford** `[MISSING]`: lab via D50 white + `κ=24389/27, ε=216/24389`; needs Bradford adaptation when a D50 color meets sRGB/oklab. Required for `lab()`/`lch()` and gradient `in lab`.
- **F-3 powerless/missing hue (NaN)** `[MISSING, subtle]`: hue=NaN when achromatic; carry as "missing" so interpolation doesn't produce garbage hues.
- **F-4 gradient/mix interpolation space + premultiplied** `[MISSING — gradient gap]`, css-color-4 §12: default **Oklab** (sRGB only if both endpoints legacy rgb/hsl/hwb); premultiply non-hue components by clamped alpha, lerp, divide by interpolated alpha; alpha lerped straight; carry-forward analogous missing components. Doing gradient lerp in Oklab avoids muddy mid-tones.
- **F-5 hue interpolation methods** `[MISSING]`: shorter (default, adjust when `|Δ|>180`), longer, increasing, decreasing; normalize `[0,360)`; dual-NaN→0, single-NaN→copy. Needed for conic/hue-space gradients.
- **F-6 sRGB↔linear for blur/opacity/blend** `[IMPROVE]`: blur, opacity compositing, gradient blends are only correct in **linear light** (gamma-space gaussian darkens edges). Wrap CPU blur/opacity/gradient sampling `to_linear → op → to_gamma`; thresholds `0.04045 / 0.0031308, 12.92, 1.055, 2.4`; signum-preserving for out-of-gamut.
- **F-7 compositing** `[note]`: premultiplied source-over `out = src + dst·(1−src.a)` in **linear** space (stylo confirms premultiplied domain; rasterization is WR/peniko).

---

## G. Transforms (stylo `values/animated/transform.rs`, `generics/transform.rs`)

- **G-8 2D matrix decompose + interpolate** `[IMPROVE]`, css-transforms-1: `MatrixDecomposed2D{translate,scale,angle,matrix}`; decompose = row lengths (det<0 flip), `angle=atan2(row0y,row0x)`, residual shear; interpolate with flip-normalization + "don't rotate the long way" (`|Δangle|>180` shift 360), lerp components, recompose. This is the transform animation we lack.
- **G-9 3D decompose (unmatrix) + quaternion Slerp** `[MISSING]`: normalize by m44, solve perspective (inverse-transpose), Gram-Schmidt → scale + 3 skews, quaternion from rotation; **Slerp** = Blink form `sin(t·θ/2)/sin(θ/2)` (avoids the precision bug of the spec's cos form). Needed for `rotate3d`/`perspective`/`matrix3d` interpolation.
- **G-10 transform-list matching + fallback** `[MISSING]`: interpolate matched-class prefix component-wise; mismatched remainders → `InterpolateMatrix` (decompose both); one-sided → pad with identity. Makes `translate rotate` animate against `translate scale`.
- **G-11 matrix composition** `[HAVE — validate]`: fold `transform = op.then(&transform)` in **f64** (f32 trig drifts), cast to f32; perspective `d.max(1)`; zero-axis rotate → identity.

---

## V. Values, cascade, selectors (stylo)

- **V-12 calc() simplify tree** `[IMPROVE]`, css-values-4: `simplify_and_sort` — flatten Sum/Product, `sort by SortKey` (canonical unit order = spec serialization order), `dedup` merging same-unit leaves; `clamp = max(min, min(center,max))`; MinMax bails to a function if children aren't comparable (px vs % unknown basis); nonlinearity guard forbids distributing through trig. Port `SortKey` + Sum/Product fold for deterministic `calc(1px+2px+3%)`.
- **V-13 length units** `[IMPROVE]`: add `lh/rlh`, viewport `sv/lv/dv` variants (`svh/lvh/dvh…`), and resolve `ch`/`ex` from **real glyph metrics** (0-advance / x-height), not `0.5em`/`0.25em`. (Container-query units `cq*` if/when container queries.)
- **V-14 LengthPercentage / PositivePercentageBasis** `[note]`: defer `min(50%,10px)` when the % basis is unknown, else you mis-simplify.
- **V-15 specificity** `[HAVE — validate]`, selectors-4 §16: `u32 = min(id,1023)<<20 | min(class_like,1023)<<10 | min(element,1023)` (saturating **per field**); `:is/:not/:has`/`:nth(An+B of S)` take their most-specific arg; **`:where` = 0**.
- **V-16 :nth-child An+B** `[HAVE — validate]`: matches iff ∃ `n≥0` with `A·n+B=i`; impl `an=i−B; A==0 ? an==0 : an/A≥0 ∧ A·(an/A)==an`. Naive `(i−B)%A==0` wrongly matches negative n (e.g. `2n+10` at i=8).
- **V-17 cascade origin + !important inversion** `[HAVE — validate]`, css-cascade-4 §6.3: effective precedence transitions > important-UA > important-user > important-author > animations > normal-author > normal-user > normal-UA. **`!important` REVERSES the origin order** (important-UA beats important-author — opposite of normal). Map onto our `SystemDefault<Theme<GlobalApp<…` and replicate the important-inverts rule.

---

## Sources
- Servo: `~/projects/aliens/servo/components/{layout,fonts,geometry}` (MPL-2.0). WebRender + taffy + stylo are deps, not in the checkout.
- taffy: `~/.cargo/registry/.../taffy-0.5.2/src/compute/grid/`.
- stylo: pinned rev `0151ab3…` (`color/`, `values/`, `selectors/`, `stylesheets/`).
- Specs: css-sizing-3/4, css-flexbox-1, css-grid-2, css-text-3/4, css-inline-3, css-backgrounds-3, css-images-3/4, css-color-4, css-transforms-1/2, css-values-4, selectors-4, css-cascade-4, UAX#9/#14/#24, CSS2 §8/§9/§10/§17.
