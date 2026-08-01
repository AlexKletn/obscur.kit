Reserved symbol naming which registered row builder a virtual list realizes
through (spec §4.9).

The builder itself cannot live in the [`View`]: a `View` is plain data —
`Clone`, `Debug`, `PartialEq` — and a closure is none of those. So the view
carries only this name and the runtime holds the builder, which is what lets a
list of 10,000 entries put 10,000 *items* in the model while only ever building
the nodes the viewport can show.
A realized virtual list's spacer: an empty box standing in for the rows outside
the window, sized to exactly the space they would have taken.

It is what makes the container's content extent true without building the rows,
so the scrollbar is proportioned correctly and native scrolling covers the whole
list rather than only the part that happens to exist.
