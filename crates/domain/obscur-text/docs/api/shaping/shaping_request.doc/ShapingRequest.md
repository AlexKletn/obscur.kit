The input to shape a single run of uniformly-styled text (spec §5.10).

A request carries the text plus everything a shaper needs to turn it into
glyphs: the resolved [`FontFace`], the font size in logical pixels, the base
[`TextDirection`], and an optional BCP-47 language hint. The run is assumed to
be **already itemized**: a single face and a single direction apply to the
whole text. Splitting mixed-script, mixed-style, or mixed-direction text into
uniform runs (itemization and bidi splitting) happens *before* shaping, so the
shaper never has to re-segment its input.

Embedding an `f32` font size means this type derives `PartialEq` but not
`Eq`.
