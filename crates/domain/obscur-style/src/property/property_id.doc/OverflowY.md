The `overflow-y` property (Keyword-valued: `visible` (initial) / `hidden` /
`clip` / `scroll` / `auto`; NOT inherited). See the reserved `OVERFLOW_*`
handles in [`crate::property::layout_keyword`]. It states the BLOCK axis
alone; the `overflow` shorthand expands to [`PropertyId::OverflowX`] and this
longhand, which are the storage. This is the axis the kit scrolls: `scroll` /
`auto` here make the box a scroll container the wheel can move, and any value
other than `visible` clips the subtree to the box.
