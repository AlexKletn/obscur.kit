Registers the row builder a virtual list realizes through (spec §4.9).

`build(index)` produces one item's node and is called only for the indices
the viewport can show, so a model of any size costs a screenful of nodes. The
view names this builder with `ViewNode::virtual_list` and states the item
count; the runtime owns the window, the scroll offset and the content extent
— the half an app cannot own, since stating the full height would mean
building the full list.
