§4.7: "The result is a hit chain that maps visual geometry back to the
logical event tree." and "Hit testing accounts for:" / "- portals;"

`Trigger` holds a portal whose content `P` is projected into `Overlay` at
window point (400, 300). A hit at (420, 320) must return `P`'s **logical**
ancestors — `[P, Trigger, PanelA, Root]` — not the visual chain
`[P, Overlay, Root]`, so a pointer event bubbles through `Trigger` and
`PanelA`.
