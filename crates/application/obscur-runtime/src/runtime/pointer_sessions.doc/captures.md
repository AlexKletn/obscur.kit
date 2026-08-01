Which node, if any, has captured each pointer (spec §2.4: "pointer capture
is per pointer ID and is released on pointer completion, node removal,
window close or system loss").

A captured pointer routes to its captor instead of the hit-tested target —
capture *overrides* hit-testing — which is how a control keeps receiving a
gesture that has wandered off it: a thumb being dragged, a finger that
slid past the edge of the row it started on.
