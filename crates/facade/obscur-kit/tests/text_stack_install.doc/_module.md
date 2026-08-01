End-to-end: `KitBootstrap::install_text_stack` gives a host a working text
stack in one call, and the faces it installs actually drive layout (spec
§5.10, §5.18).

# Why these fonts, and not the machine's

Every assertion here is a number read out of a specific font's tables, so the
faces must be the same on every machine or the file asserts nothing. They are
the vendored `assets/fonts/` faces, embedded at compile time — never
`fc-match`ed. On the authoring machine `fc-match sans-serif` resolves into
`/usr/share/fonts/WindowsFonts/`, which is exactly the drift this avoids.

The expectations, derived from the vendored faces (both 1000 upem):

* Noto Sans `hhea` ascender 1069, descender −293, line-gap 0 ⇒ at a 16px
  font-size a line box is `(1069 + 293)/1000 × 16` = **21.792px**. The
  idealized 0.8/0.2 ratios the stack falls back to give **16.0px** — so the
  two are 5.8px apart and a test cannot pass by accident.
* Noto Sans Mono `0` advance 600 ⇒ `1ch` = **9.6px** at 16px. CSS Values 4
  §6.1.2's `0.5em` substitute gives **8.0px**.
* Noto Sans glyph ids: `A` = 36, `Б` = 425. The
  [`MockTextShaper`](obscur_kit::text::shaping::MockTextShaper) emits the
  Unicode scalar instead (`A` = 65), so a real index is unmistakable.

This whole file needs both text adapters; they are in `default`, but the gate
keeps `--no-default-features` builds honest.
