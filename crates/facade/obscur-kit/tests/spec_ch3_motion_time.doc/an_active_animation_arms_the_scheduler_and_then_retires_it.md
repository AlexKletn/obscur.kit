§3.5 Rules — "active animations continue requesting frames".

A 500 ms animation with no external frame requests at all keeps the scheduler
armed for its whole duration, then retires it.
