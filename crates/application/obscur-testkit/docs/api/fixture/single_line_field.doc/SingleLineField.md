A single-line editable field, in the two overflow regimes CSS gives it.

[`view`](Self::view) is the scrolling one — 100 px wide, `white-space: nowrap`
so the value runs past the box rather than wrapping, `overflow: hidden` so the
scrolled-out glyphs clip. It is the fixture the horizontal caret-scroll tests
and the compose-only tests both drive, and its 100 px width is the number every
"the caret stayed inside the box" assertion is written against.

[`wrapping_view`](Self::wrapping_view) is the other one — 40 px wide with an
exact 20 px `line-height` and `white-space: normal`, so a space-separated value
wraps and every wrapped line is exactly one `line-height` tall. That exactness is
the point: it lets a test compare a measured box height against a painted line
*count*.

Both bind [`EditablePanel::CMD_INPUT`](super::EditablePanel::CMD_INPUT), so one
registered `on_text_input` handler serves whichever is on screen.
