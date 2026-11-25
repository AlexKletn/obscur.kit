Whether any mounted component is waiting to be re-rendered.

A host reads this after a frame: a component that asked to update during
its own render is serviced by the *next* frame, so someone has to schedule
one.
