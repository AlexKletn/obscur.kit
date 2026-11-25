Appends one rounded-rectangle contour to `segments`, starting just after the
top-left corner and running clockwise.

The single emitter for the shape, shared by [`Path::rounded_rect`] and both
contours of [`Path::rounded_rect_ring`]. It writes into a caller-owned buffer
so the ring can lay its outer and inner contours into one allocation reserved
for both, rather than building the inner one as a throwaway `Path` and copying
it across.

Emits exactly [`ROUNDED_RECT_SEGMENTS`] segments for any input: the radii are
clamped and normalised to the rect first, and a zero radius degenerates the
corner cubic rather than dropping it, so the count never varies.
