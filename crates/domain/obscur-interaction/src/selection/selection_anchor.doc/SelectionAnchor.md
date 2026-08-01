The anchor and lead endpoints of a range selection.

Range (shift) selection extends from a fixed [`anchor`](Self::anchor) — set
on the initial click — to a moving [`lead`](Self::lead), the currently
active item. A shift-click updates only the lead, keeping the anchor put, so
the selected span sweeps from the anchor to wherever the lead lands. When
the two endpoints coincide the anchor is *collapsed* onto a single item.
