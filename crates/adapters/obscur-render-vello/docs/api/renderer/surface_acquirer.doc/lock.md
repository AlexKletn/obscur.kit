Take the shared state, poison and all.

A panic on either side must not turn a rendering fault into a deadlock: the
state is a handful of plain fields, and every one of them is meaningful after a
panic elsewhere.
