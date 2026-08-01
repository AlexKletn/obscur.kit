**Proof: multi-line auto-wrapping text + `line-height` in the coordinator.**

Renders three text blocks through the REAL runtime + software [`CpuRenderer`]
(no GPU), stitched top-to-bottom into `target/multiline_text_proof.png`:

- **Block A — a long paragraph in a ~200px box.** The text auto-wraps into
  several visual lines at the box's content width; the lines stack by the
  font's natural (metric) line height so they do not overlap.
- **Block B — an explicit `\n` two-line string** in a wide box: even though
  nothing wraps, the mandatory newline forces two lines.
- **Block C — the same long paragraph with `line-height: 2`** (inherited from
  the container): it wraps to the same width but its lines are spaced twice as
  far apart (2 · font-size), proving the CSS `line-height` property feeds the
  line box height.

Before writing the PNG it machine-checks the composed display list: Block A
emits ≥ 2 `DrawText` runs at strictly increasing y (wrapped, non-overlapping);
Block B emits exactly 2 runs (`\n` break); Block C's per-line spacing equals
`2 · font-size` and is strictly wider than Block A's metric spacing.

Run it:
`cargo run -p obscur-kit --example multiline_text_proof \
  --features "render-cpu text-harfrust text-unicode"`
