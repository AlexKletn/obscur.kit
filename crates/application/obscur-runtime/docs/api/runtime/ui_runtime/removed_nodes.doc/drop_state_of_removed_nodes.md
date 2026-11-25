Drops the computed style, cascade inputs, pseudo-class state and scroll offset of every node the tree no longer holds.

Run on a reconciled frame — the only frame that can remove a node — beside the focus orphan check, which is the same question asked about one field. Cost is one sweep of maps that are then the size of the live tree, on a frame that already walks that tree several times; before this the cost was the opposite way round, because those same maps are scanned whole every composing frame by `layout_styles_from`, the resource bridge and the grid track passes, so frame time tracked nodes *ever removed* rather than nodes present.

Generational `NodeId`s mean nothing here can be reclaimed by coincidence: a remounted node never matches the id of the dead one, so a stranded entry was unreachable rather than wrong. What it was not is free — a 30-row window sliding over a long model kept every row it had ever shown.

This also restores what `styling.doc/computed_style.md` already promised: `None` for a node absent from the tree.
