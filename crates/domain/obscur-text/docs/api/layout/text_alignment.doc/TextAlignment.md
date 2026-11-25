The horizontal alignment applied to each laid-out line within the available
paragraph width (spec §5.10).

[`Start`](Self::Start) and [`End`](Self::End) are *direction-relative*: they
resolve to the leading or trailing edge according to the run's
[`TextDirection`], so `Start` hugs the left edge in left-to-right text and the
right edge in right-to-left text. [`Left`](Self::Left) and
[`Right`](Self::Right) are *absolute* and ignore direction.
[`Center`](Self::Center) centres the line, and [`Justify`](Self::Justify)
stretches inter-glyph spacing to fill the line (so its start offset is `0`).
