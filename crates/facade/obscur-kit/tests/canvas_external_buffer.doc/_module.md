A canvas drawing pixels somebody else produced, all the way to real ones.

The road an application takes to put a video frame, a camera image or a screen
capture on screen: its own bytes, in the layout a real producer hands over —
BGRA with a padded row — through a painter, a display list and a rasterizer.

What is proved here is the two things that fail silently rather than loudly:
the channel order (getting it wrong draws a blue face) and the row stride
(getting it wrong shears every row).

Spec §5.5 also allows a texture-backed foreign surface that never touches the
CPU. That road is not built.
