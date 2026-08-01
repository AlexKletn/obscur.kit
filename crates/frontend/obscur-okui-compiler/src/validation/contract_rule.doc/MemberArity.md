How many nodes one member of an [`ElementUse`] contributes to its parent's
child list.

A static check cannot count a conditional, so it does not pretend to: the
arity is the honest bound, and [`check_contract`] only reports a cardinality
violation that holds for *every* value the bound admits.
