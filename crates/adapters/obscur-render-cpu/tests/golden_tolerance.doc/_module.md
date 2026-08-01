§7.14's raster golden tolerance, exercised against real CPU renders.

§7.14: *"Raster golden tests are allowed with explicit tolerances for
anti-aliasing and font/backend variation."* §5.2 repeats it — *"Golden tests
use tolerances"* — under an **allowed differences** list naming
rasterization, anti-aliasing, font rendering details, and backend-specific
precision, while **scene geometry**, **clipping** and **paint order** are on
the *must match* side.

Neither sentence names a metric, so [`ImageTolerance`] derives one from the
two variation *sources* the spec does name, and this file is where the
resulting numbers are justified. Every constant below was **measured** on the
renders this file produces, not chosen because it sounded safe.

# The measurements this file's constants come from

Scene: a 48×48 rounded rect (radius 14) filled red on a 64×64 transparent
surface — 4096 pixels, of which exactly **80 (1.95%)** have partial alpha,
i.e. lie on an anti-aliased corner arc. The straight edges are integer-
aligned and produce no AA at all.

| change to the render                        | pixels past a delta-4 floor |
|---------------------------------------------|-----------------------------|
| same scene rendered twice                   | 0.00% (max delta 0)         |
| fill colour shifted by 4/255                | 0.00% (max delta exactly 4) |
| **worst-case AA disagreement (see below)**  | **1.95%** (max delta 240)   |
| corner radius 14 → 15                       | 2.64%                       |
| corner radius 14 → 16                       | 3.61%                       |
| **translated 1px in x**                     | **4.30%** (max delta 255)   |
| corner radius 14 → 20                       | 6.45%                       |
| fill colour shifted by 8/255                | 52.03% (max delta 8)        |

Hence [`tolerance`]: a **delta-4 noise floor** (accepts the 4/255 colour
shift, rejects the 8/255 one) and a **3% differing cap** (accepts the 1.95%
AA band, rejects the 4.30% translation).

# What this does NOT cover — read before trusting it

- **The AA variation is a model, not a second rasterizer.** There is no
  second rasterization of these pixels to compare against: the CPU backend
  hardcodes `anti_alias: true` (`cpu_renderer.rs`), and Vello's rasterizer
  needs a GPU, which this workspace has no test target for (every GPU proof
  in the repo is a hand-run `[[example]]`). So a *true* cross-backend golden
  comparison is out of reach here, exactly as `clip_cross_backend.rs` says.
  What [`worst_case_aa_variation`] does instead is take the real render and
  replace **every partial-coverage pixel** with the most extreme value any
  rasterizer could hold about it — fully covered or fully empty, alternating
  both ways. That is *strictly worse* than a real AA disagreement (it is a
  240-delta swing on every band pixel at once), so accepting it implies
  accepting any milder one; it is not evidence that two named backends agree.
- **No golden ships on disk.** The comparison is render-vs-perturbed-render.
  Storing a reference PNG and diffing a fresh render against it across font
  versions — §7.14's motivating workload — is served *in principle* by this
  facility and is not exercised by any test in this repo.
- **The margin is real but modest.** Worst-case AA sits at 1.95% and the
  tightest regression rejected here is 4.30%, a factor of ~2.2. A corner
  radius drifting 14 → 15 lands at 2.64% and is **accepted** by this cap —
  see [`the_cap_does_not_catch_a_sub_pixel_radius_drift`], which pins that
  limitation rather than letting it be discovered later. A tolerance is a
  permission to differ, and this one really does grant it.
- **A tolerance is not an equivalence.** `compare_within` is not transitive:
  three renders can pairwise pass while the first and last differ wholesale.
  It answers "is this render still its golden", not "are these the same".
