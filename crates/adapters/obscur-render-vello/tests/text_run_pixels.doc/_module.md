What folding adjacent text commands into one Vello glyph run does to the pixels.

`SceneConverter::draw_text` draws **one glyph run per `DrawText` command** and
this file is why. A glyph run is not a batch: `vello_encoding`'s resolver counts
one path per run (`layout.n_paths += 1`), emits every glyph of the run as
subpaths and closes them with a single `PathTag::PATH`, so N commands drawn
apart are N non-zero fills composited one over another, while the same glyphs
folded into one run are a single non-zero fill of their union. Those two agree
only where no two glyphs reach the same pixel: elsewhere `1-(1-a)(1-b)` is not
`min(a+b, 1)`, and a counter wound against a neighbouring outline cancels to
zero instead of being painted over.

The tests here render both drawings through the shipping renderer
(`VelloRenderer::render_to_image`, the same path the on-screen present uses) and
read the pixels back, so nothing is argued. They pin down both halves of the
line:

* **far apart** — no visible pixel changes. This is the keyboard's own shape,
  and it is why a fold could sit in the tree for four passes without anyone
  seeing it. (A handful of *bytes* still change, in pixels whose alpha is zero
  on both sides. A transparent pixel's colour channels reach no screen — and
  they are not even stable between two renders of the *same* list, which the
  control test measures and which is why nothing here compares bytes.)
* **touching** and **overlapping** — visible pixels change: 2 pixels by up to
  22/255 in alpha, and 76 pixels by up to 58/255, on this backend.

A fold was in the tree, accepted on the claim that it was pixel-identical and
guarded by a test that compared the folded pair against a single command already
holding both glyphs — that is, against what the fold produces. It was measured
with this file and removed.

**This file needs an adapter, is `#[ignore]`d because of it, and fails outright
if it is asked for without one.** On a runner with no GPU these four show up in
the *ignored* column of `cargo test`, which is a number that is visibly not
"passed"; they never report a green line having checked nothing. The sentry that
runs everywhere, with or without a GPU, is
`every_text_command_gets_a_glyph_run_of_its_own` in `scene_converter.rs`: it
counts encoded glyph runs and fails the moment a fold comes back. What is
written here is the *reason* that sentry exists, in measured pixels.
