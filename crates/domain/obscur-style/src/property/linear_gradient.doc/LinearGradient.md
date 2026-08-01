A CSS `linear-gradient(...)` image, as authored in a stylesheet.

The gradient is described by an `angle_deg` on the CSS convention — `0` points
*to top*, `90` *to right*, `180` *to bottom*, `270` *to left*, increasing
clockwise — and an ordered list of color [`GradientStop`]s. The `to <side>`
keyword forms are folded to their equivalent angle by the CSS compiler, so
this type stores only an angle.

The angle defines the gradient *line*; the scene coordinator turns it plus the
painted box's bounds into the concrete start/end points the renderer's
`obscur_scene::paint::LinearGradient` needs. This type is a pure style concept
and deliberately has **no dependency on the scene**.

# Repeating gradients

`repeating-linear-gradient(...)` is modelled as this same value with
`repeating` set and a `period` carried. A non-repeating gradient spans the
whole box (offset `0`→`1` across the painted box); a repeating one instead
defines *one period* — the extent from the last authored color-stop position —
and the pattern tiles across the box. The [`stops`](Self::stops) are always
normalized to `0.0..=1.0` **over that period** (the CSS lowering divides each
authored position by the period), and `period` records the period's length so
the scene coordinator can size the gradient axis to exactly one period and let
the renderer tile it with `ExtendMode::Repeat`:

- [`Length::Px`] — an absolute px-positioned period (the common stripe case,
  e.g. `repeating-linear-gradient(45deg, #000 0 8px, #fff 8px 16px)` = a 16px
  period);
- [`Length::Percent`] — a period expressed as a fraction of the gradient line.

Because [`GradientStop`] carries `f32` data, this type derives [`PartialEq`]
but not [`Eq`]/[`Hash`].
