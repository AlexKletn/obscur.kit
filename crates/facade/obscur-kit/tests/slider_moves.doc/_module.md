A slider moving, reached the way an application reaches it.

The whole road in one place: a pointer or a key enters the runtime, the hit test
finds a node inside the control, the walk up finds the control, its scale turns
a position into a value, and the value is reported through the node's `on:input`.

A slider is CONTROLLED, like a field. The runtime reports the value and never
writes it; these cases assert what was REPORTED, because that is the contract —
an application that does not re-render with the new value will see its thumb
spring back, and it should.
