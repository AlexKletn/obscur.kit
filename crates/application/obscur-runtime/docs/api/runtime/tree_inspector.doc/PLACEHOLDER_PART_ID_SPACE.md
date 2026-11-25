The high bit that lifts a field's own [`NodeId`] into the synthetic id space of
its `placeholder` part (spec §4.16).

Real ids are minted by the retained tree and never reach `1 << 61`, so setting
this bit and carrying the field id in the low bits yields an id that is stable
across frames and cannot collide with a real node — nor with
[`SCROLLBAR_PART_ID_SPACE`], which is a different bit.
