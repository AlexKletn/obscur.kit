The resolved tab navigation sequence for a set of focus nodes (spec §7.1).

[`compute`](Self::compute) derives the sequence from a slice of
[`FocusNode`]s. Only [`is_tab_focusable`](FocusNode::is_tab_focusable) nodes
participate (excluding negative `tab_index` and disabled nodes). The ordering
mirrors HTML `tabindex`:

1. nodes with a positive `tab_index`, in ascending `tab_index` order; ties
   keep their original (input) order;
2. followed by nodes with `tab_index == 0`, in their original order.

The sort is stable, so the sequence is fully deterministic given the input.
