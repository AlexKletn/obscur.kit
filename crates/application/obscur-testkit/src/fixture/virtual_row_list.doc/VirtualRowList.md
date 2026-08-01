A `<div>` wrapping a `<VirtualFor>` marker over a single 24px `row` template.

The collection it stands for is assumed huge, and the fixture says nothing about
how many items there are — that is the point. §4.9 realizes a window sized from
`VirtualLayout` against the *committed viewport*, so what a test varies is the
viewport, and what it counts is realized rows.
