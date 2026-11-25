The [`GridTemplate`] derived for every grid node that authored one, keyed by
[`NodeId`].

This is the grid twin of [`LayoutStyles`](super::LayoutStyles): a per-node
side-table the coordinator threads into the layout walk so a
[`LayoutDisplay::Grid`](obscur_layout::model::LayoutDisplay::Grid) node with an
authored template dispatches the real track-sizing algorithm. A node absent
from this map simply has no authored template and falls back to the default
single implicit column.
