The node's computed `color`, or `None` when it resolved none.

Used by the §4.7/§4.14 portal test to read the *inherited* value: `color`
inherits, so a node that declares none answers with whatever its logical
ancestor chain supplies — which is exactly the fact §4.14 pins down for a
portalled node.
