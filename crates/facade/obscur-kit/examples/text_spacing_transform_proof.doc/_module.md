**Proof: CSS Text 3 `letter-spacing` (§8.1), `word-spacing` (§8.2), and
`text-transform` (§2.1) in the live text path.**

Renders five text blocks through the REAL runtime + software [`CpuRenderer`]
(no GPU), stitched top-to-bottom into `target/text_spacing_transform_proof.png`,
and machine-checks the composed `DrawText` runs before writing the PNG:

- **A — baseline `hello`** (no spacing): the reference run width `W0`.
- **B — `hello` with `letter-spacing: 6px`**: the painted run is visibly wider;
  its advance ≈ `W0 + 6·N` for the `N = 5` typographic character units (the
  tracking is added once per cluster, §8.1).
- **C — `ab cd` baseline** vs **D — `ab cd` with `word-spacing: 12px`**: the
  two-word run grows by exactly `12px` — the extra spacing lands on the single
  word separator (§8.2), not on every character.
- **E — `hello` with `text-transform: uppercase`**: the shaped/painted run's
  source text is `HELLO` (the string is cased BEFORE shaping, §2.1).

Run it:
`cargo run -p obscur-kit --example text_spacing_transform_proof \
  --features "render-cpu text-harfrust text-unicode"`
