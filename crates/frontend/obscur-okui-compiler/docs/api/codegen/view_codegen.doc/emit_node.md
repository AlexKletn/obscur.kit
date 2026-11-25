Appends the statement pushing the single `ViewNode` `node` lowers to.

A transparent fragment and a (still-spliced) loop push their children instead
of a node of their own. An [interpolation](UiIrNode::interpolation) pushes a
**text** node — it is a body `{expr}`, and its content is the expression.
