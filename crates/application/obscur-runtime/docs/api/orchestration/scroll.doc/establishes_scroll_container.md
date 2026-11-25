Whether a node's computed `overflow` makes it a **scroll container** in the
css-overflow-3 §3 sense: `scroll`, `auto`, **or `hidden`**. Unlike
[`overflow_scrolls`] (which names only the visibly-scrollable `scroll`/`auto`),
this includes `hidden` because "the content must still be scrollable
programmatically […] and the box is therefore still a scroll container". Only
`overflow: clip` (and `visible` / absent) is excluded — `clip` "forbids
scrolling entirely, through any mechanism, and therefore the box is not a scroll
container".

The two axis longhands are read through
[`OverflowAxes`](obscur_style::computed::OverflowAxes), so an author who states
one axis is answered: §3's pairing rule has already carried a lone `overflow-x:
hidden` over to the block axis by the time this asks.
