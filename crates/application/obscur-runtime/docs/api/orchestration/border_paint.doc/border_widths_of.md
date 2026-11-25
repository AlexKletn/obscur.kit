Reads the node's four computed border widths — top, right, bottom, left — in
one pass, each by [`border_width_of`].

The scene walk calls this ONCE per node and hands the answer to whichever
border shape it then tries; see [`BorderWidths`] for why that matters.
