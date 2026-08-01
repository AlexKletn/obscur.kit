Registers the row builder a virtual list realizes through (spec §4.9).

The view names this list with `ViewNode::virtual_list(id, count, row_extent)`
and states how many items exist; `build(index)` produces one row and is
called only for the indices the viewport can show. The runtime owns the
window, the scroll offset and the content extent — which is the half an app
cannot own for itself, because stating the full height would mean building
the full list.
