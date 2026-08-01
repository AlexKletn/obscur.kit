The font chosen for one itemized run: a [`FontId`] paired with the concrete
[`FontFace`] it names (spec §5.10 font fallback, §5.18 resolution).

A [`FontResolver`](crate::font::FontResolver) walks a fallback chain and
returns a `ResolvedFont` for the first family that can render the run. The
[`font`](Self::font) tags the run's glyphs so later stages can look the face
back up in whatever registry issued the id, while the [`face`](Self::face) is
handed straight to the shaper. Because [`FontFace`] embeds `f32` metrics this
type derives `PartialEq` but not `Eq`.

# Synthetic styling (spec §5.18)

When the request asks for italic/oblique or a bold weight but the *matched*
face is registered upright and/or lighter, the resolver cannot supply a true
cut — the nearest face wins by [`FontAttributes::distance`](crate::font::FontAttributes::distance)
but is upright/regular. Rather than render the run plainly, the resolver
flags it for **synthesis**: [`synth_oblique`](Self::synth_oblique) shears the
glyphs to fake a slant and [`synth_bold`](Self::synth_bold) thickens them to
fake weight. The flags are render hints carried downstream to the renderer;
they default to `false` (a true match needs no synthesis).
