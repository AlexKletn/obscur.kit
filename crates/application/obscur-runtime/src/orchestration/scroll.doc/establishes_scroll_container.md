Whether a node's computed `overflow` makes it a **scroll container** in the
CSS Overflow 3 §overflow sense: `scroll`, `auto`, **or `hidden`**. Unlike
[`overflow_scrolls`] (which names only the visibly-scrollable `scroll`/`auto`),
this includes `hidden` because "the content must still be scrollable
programmatically […] and the box is therefore still a scroll container". Only
`overflow: clip` (and `visible` / absent) is excluded — `clip` "forbids
scrolling entirely, through any mechanism, and therefore the box is not a scroll
container".
