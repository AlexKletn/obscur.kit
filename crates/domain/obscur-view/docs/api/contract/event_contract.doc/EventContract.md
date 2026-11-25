The exported contract for one event a component emits or accepts
(spec §2.3, §2.13).

Events are strictly typed: both the event name and its payload type are
referenced by interned handles, never by string dispatch. The two flags
describe how the event participates in the capture→target→bubble→default
propagation model: `bubbles` records whether it travels back up the bubble
phase, and `cancelable` records whether a handler may suppress the default
action. They are kept as distinct booleans because they are independent
traits rather than mutually exclusive modes.
