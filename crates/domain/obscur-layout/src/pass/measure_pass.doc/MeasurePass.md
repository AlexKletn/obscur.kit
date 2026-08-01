The **measure** phase input for a node (spec §4.2): the [`BoxConstraints`]
under which the node is asked for its desired size.

Measure returns a desired size (a
[`LayoutResult`](crate::model::LayoutResult)) without positioning anything;
positioning is the job of the separate arrange phase
([`ArrangePass`](super::ArrangePass)).
