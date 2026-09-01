A row of component-backed keys, one of them pressed.

The same shape as the rest of this file's `view`, with the one difference that
matters: every child is a component host rather than a plain element, so a press
runs a component sub-reconcile.
