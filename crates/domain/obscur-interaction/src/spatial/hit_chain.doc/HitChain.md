The ordered chain of hits from the topmost (target) node down to the root
(spec §4.7).

A hit chain is the result of a hit-test query: it maps a point in visual
geometry back to the logical event tree. Index `0` is the topmost node that
was struck (the event target); the last element is the root. Events
propagate along this chain (capture from root toward target, bubble back).
