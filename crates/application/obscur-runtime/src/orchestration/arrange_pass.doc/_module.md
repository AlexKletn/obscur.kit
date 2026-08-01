The arrange walk: [`arrange`] and [`arrange_out_of_flow`].

Where [`measure`] answers a question, `arrange` *places*: it lays a node out
under its constraints, writes its **absolute** [`LayoutBox`] into the results
map, and recurses into each child at the accumulated origin. Everything that
only arrange knows lives here — the containing block a positioned or
transformed node establishes, the scroll container's block-axis relaxation,
the percentage basis a child inherits, `position: sticky`'s scrollport clamp,
and out-of-flow placement.
