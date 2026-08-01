Builds a scene [`SceneLinearGradient`] fill brush for a style
[`LinearGradient`] over the box `bounds` (spec §5.2).

The direction defines the gradient *line*: a line through the box center, with
its `0%` / `100%` endpoints at the projections onto that line of the box extent
it covers. The endpoints are `center ∓ dir · half_len` where `dir` is the unit
gradient-line vector (screen space, y down) and, per the note under
<https://drafts.csswg.org/css-images-3/#linear-gradient-syntax>, the gradient
line length is `|w·dir_x| + |h·dir_y|` (so `half_len` is half of that). Two
direction forms feed `dir`:

* an explicit `<angle>` (CSS `0` = to top, increasing clockwise) → `dir =
  (sin θ, −cos θ)`, giving the classic `|w·sin θ| + |h·cos θ|` line length;
* a `to <corner>` → the **aspect-ratio-aware** direction (spec §3.1): the line
  points into the corner's quadrant and is perpendicular to the diagonal
  joining the two neighbouring corners, i.e. `dir = normalize(±h, ∓w)`. This is
  *not* a fixed 45° except on a square box.

Each style stop's authored (resolved) `0.0..=1.0` position becomes the scene
stop's offset, and its [`StyleColor`](obscur_style::property::StyleColor) becomes the scene paint
[`Color`](obscur_scene::color::Color); an
unpositioned stop (never produced by the CSS lowering, which resolves all
positions) falls back to its index-proportional offset.

A non-repeating gradient keeps the default
[`ExtendMode::Pad`](obscur_scene::paint::ExtendMode::Pad) and its axis spans
the whole box (the `0%`/`100%` endpoints at `center ∓ dir · half_len`). A
[`repeating`](LinearGradient::repeating) gradient instead sizes its axis to
**one period** (the axis runs from the box's `0%` anchor a distance of one
period along `dir`, with the stops already normalized to `[0, 1]` over that
period) and uses [`ExtendMode::Repeat`](obscur_scene::paint::ExtendMode::Repeat)
so the renderer tiles the pattern across the box. The period is either an
absolute px length or a fraction of the full gradient line
(`2 · half_len`). On Vello this tiles as a true repeating ramp; on the CPU
path a repeated gradient flattens to its first stop (the documented
GPU-class-effect policy), so a stripe reads as a flat first-stop fill there.
