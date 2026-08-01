End-to-end: **relative units survive the layout↔paint seam** (spec §4.5,
§4.14).

CSS specifies that the computed value of a `<length>` is already absolute:
`em` / `rem` / `vw` / `vh` are folded once, at the computed-value step, so
every consumer downstream of it reads plain pixels. These tests hold the
*whole pipeline* to that, because it is the one place the property could be
resolved twice and get two answers: layout used to fold relative units itself
while paint read the same computed property through `Length::as_px` (absolute
`px` only), so the two phases disagreed about the same box — a `.1em` border
reserved space and painted nothing.

Each test authors real CSS, runs one real frame through [`UiRuntime`], and
asserts against the *published* [`DisplayList`] — never a hand-built scene —
so a regression anywhere from the CSS compiler through the resolver to the
scene coordinator fails here.
