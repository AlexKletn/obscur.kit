The same component as [`Card`], with its render **authored** rather than
hand-built — the point of the `<slot>` spelling.

Deliberately a second component rather than a change to `Card`: the two must
be able to disagree for either test to mean anything, and `Card` is what pins
that a hand-built outlet still works.
