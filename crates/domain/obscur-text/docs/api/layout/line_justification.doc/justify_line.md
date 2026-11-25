Distributes the extra space that fills `max_width` across one line's
inter-word gaps by widening the trailing glyph of each gap.

A gap sits before every glyph (after the first) whose cluster is in
`word_starts`; the glyph immediately before it is the gap's trailing glyph.
The `max_width - line_width` surplus is split equally across the gaps, so a
line with `g` gaps grows to exactly `max_width`. Nothing happens when the
line has no interior gap or is already at/over `max_width`.
