The transform that undoes this one, or `None` when there is none.

`t.inverse()?.apply(t.apply(p))` recovers `p` (to floating-point
tolerance). This is what maps a screen point back into an element's own
untransformed coordinate space — the pointer path's half of what
painting does forward.

A **singular** matrix — a zero determinant, as produced by `scale(0.0,
1.0)` or any transform that collapses the plane onto a line or a point —
has no inverse and yields `None` rather than a division by zero. A
matrix whose determinant is finite but so small that inverting it
overflows to a non-finite coefficient is rejected the same way, so the
returned transform is always usable.
