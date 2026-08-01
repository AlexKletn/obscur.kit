Whether `point` lies inside the rounded rectangle `bounds` with per-corner
`radii` (top-left, top-right, bottom-right, bottom-left as `(rx, ry)`) — the
hit region of a `border-radius` box (§4.7 "custom shapes"; CSS Backgrounds 3
§4.1 border-radius curve).

The point must first be inside the rectangle. Then, in each corner's quadrant
where the point is nearer the corner than the straight edges begin, it must
also fall inside that corner's ellipse: `((px−cx)/rx)² + ((py−cy)/ry)² ≤ 1`,
where `(cx, cy)` is the ellipse centre inset by the radii. A zero-radius corner
contributes no cut-out (its quadrant test never triggers), so an all-zero
`radii` reduces exactly to `bounds.contains(point)`.
