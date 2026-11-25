Whether this container shows a scrollbar.

`overflow: hidden` is a scroll container — it clips, and it can be scrolled
programmatically — so it still needs the clip and the content translation this
struct drives. What it must not have is a **scrollbar**: CSS gives one only to
`scroll` (always) and `auto` (when overflowing). Painting one for `hidden`
advertises a scroll the user cannot perform.
