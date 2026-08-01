Turns an interaction-state change into the per-node style dirty set the style
phase consumes: [`RestyleScope`] (Obscur Kit 2 specs §3.4 phase 5, §4.3).

§4.3 requires that "recalculation starts at the nearest affected layout
context" and propagates "downward only when constraints change". For the style
phase the equivalent question is: when node `N`'s `:hover` toggles, whose
*computed style* can differ? This module answers it — soundly, from the shape
of the selector engine rather than from a guess — and records the answer as a
[`StyleInvalidation`], the typed accumulator §4.3 asks for.
