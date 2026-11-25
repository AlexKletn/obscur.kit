The CSS `white-space` processing model (CSS Text Module Level 3 §3, §4.1.1).

`white-space` is the single property that folds together two orthogonal
behaviours the spec's §3 table lays out:

| value      | collapse spaces | segment break `\n` | soft wrap |
|------------|-----------------|--------------------|-----------|
| `normal`   | yes → one space | collapses to space | yes       |
| `nowrap`   | yes → one space | collapses to space | no        |
| `pre`      | preserved       | preserved (forced) | no        |
| `pre-wrap` | preserved       | preserved (forced) | yes       |
| `pre-line` | yes → one space | preserved (forced) | yes       |

Those three columns are exposed here as [`collapses_spaces`](WhiteSpace::collapses_spaces)
(the collapsing done at the pre-shape seam), [`preserves_segment_breaks`](WhiteSpace::preserves_segment_breaks)
(whether a `\n` becomes a mandatory line break), and [`wraps`](WhiteSpace::wraps)
(whether soft wrap opportunities exist). [`LineBreak::opportunities`](super::LineBreak::opportunities)
reads the latter two to decide which break points a run yields, and
[`ParagraphLayout::wrap`](super::ParagraphLayout::wrap) reads `wraps` (via those
opportunities) to decide whether a line may break at the box edge at all.
