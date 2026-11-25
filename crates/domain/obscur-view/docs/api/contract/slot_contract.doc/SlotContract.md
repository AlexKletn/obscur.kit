The exported contract for one named slot of a component (spec §2.11,
§2.12).

A slot names a position where children may be placed. It pairs an interned
slot name with a [`SlotCardinality`] (how many children it accepts) and a
[`ChildContract`] (which children it accepts). Both the shape and the
membership rules are therefore fully typed.
