Builds the placeholder [`FontFace`] used to seed a text-measure
[`ShapingRequest`].

The measure pass shapes through the runtime's installed shaper: the mock is
font-agnostic and the wired HarfRust adapter ignores the request face in favor
of its host font, so a generic monospace metadata face suffices as the request
carrier. Real per-node face resolution (via a `FontProvider`) is a later phase.

This face is deliberately **identical** to the scene coordinator's default
face (same family, attributes, source, and metrics), so a text node measured
here and drawn by the scene phase carry the same shaping request. That keeps
measure and draw font-consistent and lets the runtime's shaping cache serve
the compose pass from the run the measure pass already shaped.
