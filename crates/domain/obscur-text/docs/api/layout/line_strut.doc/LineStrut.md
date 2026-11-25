The inline **line-box strut** plus the used `line-height` (CSS 2.1 §10.8.1 /
CSS Inline 3 §5).

Per CSS 2.1 §10.8 each line box is sized from a *strut* — an imaginary
zero-width inline box with the element's own (primary) font metrics — combined
with the inline content actually on the line. The strut is **always present**,
even on an empty line, so a line never collapses below the element's own font
height. `ascent`/`descent` are that primary face's vertical metrics (the
descent carries everything below the baseline, including line-gap, so
`ascent + descent` is the primary content box); `line_height` is the CSS
*used* `line-height` L, into which any excess over the line's content box is
distributed as half-leading (§10.8.1).
