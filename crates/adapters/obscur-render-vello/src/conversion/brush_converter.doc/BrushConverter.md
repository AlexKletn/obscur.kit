A converter from obscur-scene paint sources to peniko/kurbo equivalents.

It owns a [`ColorConverter`] so brush and gradient-stop colors go through the
same sRGB mapping.

# Gradient mapping

- [`Brush::Linear`] → [`peniko::Gradient::new_linear`] over the same axis.
- [`Brush::Radial`] → [`peniko::Gradient::new_radial`] over the same
  center/radius. The domain models a concentric radial gradient, which is the
  single-circle case of peniko's radial gradient.
- [`Brush::Conic`] → [`peniko::Gradient::new_sweep`]. The domain conic
  gradient sweeps one full turn clockwise from `start_angle`, so it maps to a
  sweep over `[start_angle, start_angle + 2π]`. peniko's sweep advances in the
  same offset direction, so the stop offsets and colors are preserved; only
  the exact angular rasterization is Vello's (spec §5.2).

In every case the gradient's color stops (offset + converted color) and its
[extend mode](obscur_scene::paint::ExtendMode) carry over unchanged.
