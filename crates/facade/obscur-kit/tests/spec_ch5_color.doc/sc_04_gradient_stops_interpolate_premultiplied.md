§5.7 "- premultiplied alpha;", reinforced by css-color-4 §12.3: "Interpolating
colors using the premultiplied representations tends to produce more attractive
transitions than the non-premultiplied representations".

The premultiplied interpolation of `transparent` → blue yields
`rgba(0,0,255,0.5)` at the midpoint — **full-strength** blue at half alpha.
Un-premultiplied interpolation treats `transparent` as `rgba(0,0,0,0)` and
lerps the colour toward black, giving `rgb(0,0,128)` at alpha 0.5 — the classic
"fade-to-transparent goes grey" bug. The `B == 255` assertion isolates that and
is independent of whether compositing is linear or sRGB.
