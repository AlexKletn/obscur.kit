**Proof: CSS Text 3 `white-space` processing + `tab-size` in the coordinator.**

Renders text blocks through the REAL runtime + software [`CpuRenderer`] (no
GPU), stitched top-to-bottom into `target/white_space_proof.png`, proving the
four behaviours of the white-space model (CSS Text 3 §3, §4.1.1, §4.3):

- **A — `white-space: normal`** on `"a    b\nc"` (four spaces + a newline) in a
  wide box: the run of spaces collapses to ONE space and the segment break
  collapses to a space, so it is a single line of width `W_normal`.
- **B — `white-space: pre`** on the SAME `"a    b\nc"`: the four spaces are
  PRESERVED (so the first line is wider than A's collapsed one) and the `\n`
  forces a SECOND line.
- **C — `white-space: nowrap`** on a long phrase in a NARROW box: it never
  soft-wraps, staying on ONE line whose width exceeds the box (overflow).
- **D — `white-space: pre; tab-size: 4`** on `"a\tb"`: the preserved tab
  advances the pen to the next tab stop (a multiple of `4 × space-advance`),
  proven against a plain single-space `"a b"` (narrower) and a `tab-size: 8`
  render (wider), with the tab landing exactly on the stop.

Before writing the PNG it machine-checks the composed display list:
`W_normal < W_pre` (collapsing is narrower than preserving); B is two lines;
C is one line wider than its box; and D's pen after the tab lands on
`4 × space-advance`, with `tab-size: 8` landing on `8 × space-advance`.

Run it:
`cargo run -p obscur-kit --example white_space_proof \
  --features "render-cpu text-harfrust text-unicode"`
