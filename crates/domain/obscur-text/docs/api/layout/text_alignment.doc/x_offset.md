Computes the horizontal start offset (in logical pixels) for a line of
width `line_width` within `available_width`, under base `direction`
(spec §5.10).

The result is the x position of the line's leading glyph relative to the
paragraph's left edge:

- [`Left`](Self::Left) and [`Justify`](Self::Justify) return `0`.
- [`Right`](Self::Right) returns `available_width - line_width`.
- [`Center`](Self::Center) returns half of the remaining space.
- [`Start`](Self::Start) returns `0` under left-to-right and the
  right-aligned offset under right-to-left; [`End`](Self::End) is the
  mirror of `Start`.

When the line is wider than the available width the raw offset would be
negative; it is clamped up to `0` so the line never starts left of the
paragraph edge.
