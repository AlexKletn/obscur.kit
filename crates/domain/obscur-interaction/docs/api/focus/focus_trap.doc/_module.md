The reserved property that declares a focus trap.

A trap is not something an application *calls* — it is something a view *says*.
An element carrying [`FOCUS_TRAP`] traps focus for exactly as long as it is in the
committed tree, which is what makes a modal dialog's trap open and close with the
dialog itself rather than with a pair of calls an author has to keep balanced.

The symbol lives here, beside the rest of the focus vocabulary, rather than with
the style attributes: nothing about it selects, cascades or paints. It is read by
the runtime's focus reconciliation and by nothing else.
