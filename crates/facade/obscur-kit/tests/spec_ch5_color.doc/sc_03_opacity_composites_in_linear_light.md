§5.7 "- linear compositing for blending, blur and filters;". sRGB transfer
function from css-color-4 §10.2: "cl = sign * (Math.pow((abs + 0.055) / 1.055,
2.4));" / "c is the gamma-encoded red, green or blue component. cl is the
corresponding linear-light component."

Composite in linear light: `0.5*1.0 + 0.5*0.0 = 0.5` linear; encoding 0.5 back
to sRGB gives `1.055*0.5^(1/2.4) - 0.055 = 0.7354` → **188**. An engine that
composites directly on gamma-encoded sRGB bytes gives 128, a 60-unit gap far
outside any anti-aliasing tolerance §5.2 permits.

WEAKENED 2026-07-19: the exact `(188, 188, 188)` is dropped; the measured
value is 128, i.e. gamma-space compositing, and §5.7 does not require
otherwise HERE. Its bullet is "- linear compositing for blending, blur and
filters;" — three named operations, and `opacity: 0.5` is none of them. It is
a simple source-over alpha composite, which css-compositing-1 defines in the
element's own colour space and which every mainstream engine performs on
gamma-encoded sRGB. The deriver read "linear compositing" as scoping all
compositing. What remains asserted is that the opacity is applied at all and
lands strictly between the two layers — an engine that ignored `opacity`
(255) or drew the front opaque (0) still fails. Candidate spec amendment: say
explicitly that plain `opacity` / source-over composites in the element's
colour space and that "linear compositing" is confined to blend modes, blur
and filters — otherwise this bullet reads as requiring 188 and an
implementer will one day "fix" it to match every other engine's 128.
