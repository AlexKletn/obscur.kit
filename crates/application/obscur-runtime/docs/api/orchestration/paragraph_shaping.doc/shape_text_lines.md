Shapes the wrapped multi-line [`ParagraphLayout`] for `content` on `node`
using the SAME style inputs and `box_width` (the text node's content-box
width, its wrap scrollport) the compose pass wraps the drawn runs with
(spec §5.10) — so a caret / hit-test resolved against this layout lands on
the same line the glyphs paint on. The runtime uses it for vertical caret
motion and 2D click→byte hit-testing over the live editing text.
