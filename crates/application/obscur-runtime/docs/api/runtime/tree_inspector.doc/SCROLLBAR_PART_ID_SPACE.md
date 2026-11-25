The high bit that lifts a scroll container's own [`NodeId`] into the synthetic
id space of its scrollbar parts (spec §4.16).

Real ids are minted by the retained tree and never reach `1 << 62`, so setting
this bit — over the container id shifted up by one to leave room for the part
index — yields an id that is stable across frames and cannot collide with a
real node, with the container's other part, nor with
[`PLACEHOLDER_PART_ID_SPACE`], which is a different bit.
