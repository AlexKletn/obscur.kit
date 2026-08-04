Registers what a numeric control — a slider — reports its new value to.

Both ends are passed. A single slider reports the same number twice, which is
what it is: a range of one point. The value is the CONTROL's answer, not the
application's state — a slider is controlled like a field, so an application
that does not re-render with the new value will see its thumb spring back.
