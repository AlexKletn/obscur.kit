§7.14, verbatim: "Raster golden tests are allowed with explicit tolerances
for anti-aliasing and font/backend variation."

WEAKENED 2026-07-19: this originally required the mismatch to NAME THE
DIFFERING PIXEL'S COORDINATE, (37, 21). §7.14 is two sentences — the list of
preferred backend-independent snapshot kinds, then the sentence quoted above
— and neither mentions what a raster failure reports. Pixel-coordinate
reporting was derived from what a good diff tool does, not from this spec.
The typed mismatch does carry the differing-pixel count, the total, the max
channel delta and the tolerance that was allowed, and the assertions on the
parts §7.14 does speak to — that the tolerance is EXPLICIT, that it decides
pass from fail, and that a failure is a typed value rather than a bare bool
— are kept. Candidate spec amendment: state what a raster golden failure must
report, since "1 pixel differs by 2" is not enough to debug from and locating
it by hand is the expensive part.
