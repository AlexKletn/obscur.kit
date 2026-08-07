§4.7: "The result is a hit chain that maps visual geometry back to the
logical event tree." and "Hit testing accounts for:" / "- portals;"
§4.14: a portal preserves the logical parent's style/theme scope.

`Trigger` holds a portal whose content `P` is projected into `Overlay` at
window point (400, 300). A hit at (420, 320) must return `P`'s **logical**
ancestors — `[P, Portal, Trigger, PanelA, Root]` — not the visual chain
`[P, Overlay, Root]`, so a pointer event bubbles through `Trigger` and
`PanelA`; and `P`'s inherited `color` must come from `Trigger`, not from
`Overlay`.

# What each half is worth today

Portals are **not wired** (see `docs/api/12-reality.md`, row *Portals*): the
primitive's target name resolves to nothing, so `P` never leaves its logical
parent. That makes the *chain* assertions true for the wrong reason — they
are also what an implementation with no portals at all produces.

The §4.14 colour assertions are the ones that bite. Three scopes declare
three different colours, so a failure names the defect: inheriting
`Overlay`'s red means portals were wired by **reparenting the logical tree**,
which drags style and theme scope along with paint and is the one thing
§4.14 forbids; inheriting `Root`'s blue means the inherited value was lost
altogether. Both are live failure modes of the obvious wrong fix, and both
fail here.
