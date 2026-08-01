The direction a transition plays in (spec §4.17 "reversal").

A transition runs either `Forward` (an *enter* animation, driving progress
from its origin toward its target) or `Reverse` (an *exit* or *undo*, driving
progress back from the target toward the origin). Modeling reversal as a
direction — rather than swapping the endpoints — lets an in-flight transition
be turned around in place while keeping its stable identity and endpoints
intact (see [`TimelineId`](crate::timeline::TimelineId)).

[`orient`](Self::orient) folds this direction into a raw progress value so a
single forward-parameterised curve serves both directions.
