How a scene [`Brush`] becomes pixels on the software backend: the real linear
and conic gradient rasterizations, and the flat colour every other brush
degrades to (§5.1 gradients, spec §5.2 rasterization approximation).

The three answers this module gives share one vocabulary — stops, offsets,
spans, interpolated colour — and one hard constraint the domain imposes on all
of them: css-color-4 §12.3 (and §5.7's "premultiplied alpha") interpolate stop
colours in **premultiplied** space, while `tiny-skia` interpolates in straight
alpha. Every gradient here therefore samples its own curve and hands `tiny-skia`
only points that already sit on it. That constraint, not the drawing, is why
this is one entity: its interface is three calls wide, and its internals are a
stop-expansion scheme, an angular wedge decomposition, and two colour mixers
that differ in exactly the way the spec cares about.
