Converts obscur-scene gradient stops into peniko color stops, **densified in
Oklab** so Vello rasterizes the CSS-correct interpolation.

CSS Images 4 §3.4 interpolates a gradient's stops in Oklab by default, but
this Vello/peniko build samples the ramp linearly in sRGB regardless of
`interpolation_cs` (see [`GRADIENT_INTERPOLATION_CS`]). To honor the spec we
pre-compute [`DENSIFY_SEGMENTS`]`- 1` interior stops between each authored
pair by interpolating in **Oklab with premultiplied alpha** (convert each
endpoint sRGB→linear→Oklab, premultiply by alpha, lerp `L,a,b,α`,
un-premultiply, convert Oklab→sRGB) — reusing the CSS Color 4 conversions in
[`obscur_scene::color::convert`]. peniko's sRGB-linear sampling between the
now-dense stops closely approximates the true Oklab ramp.

Authored stops keep their **exact** converted color and offset (the endpoints
are not round-tripped through Oklab); only the inserted interior stops are
Oklab-computed. A gradient with fewer than two stops has nothing to
interpolate and passes through unchanged (byte-identical to before).
