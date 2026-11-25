The axis-aligned bounding box of the rect's four transformed corners.

An affine map sends a rectangle to a parallelogram, which is not a
[`Rect`]; this returns the smallest axis-aligned rect that still contains
it. It is therefore **conservative** — under a rotation the result has
area to spare in the corners — which is exactly what a broad-phase index
wants: it may over-report, never under-report.

Under a translation-only transform the parallelogram is already
axis-aligned, so the result is exact.
