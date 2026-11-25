The callback through which a [`LayoutAlgorithm`] measures one of a node's
children under chosen constraints, without knowing what the child *is*.

[`LayoutAlgorithm`]: super::LayoutAlgorithm

Layout algorithms in this crate do not recurse into the retained tree; the
runtime owns tree traversal (spec §4.2). When an algorithm needs a child's
size under some constraints, it calls [`measure_child`](Self::measure_child)
with the child's index (into the `children` slice the algorithm was given).
The runtime's implementation decides how to satisfy the request:

* for a **text leaf**, it shapes and wraps the run to the constraints' width
  and returns the resulting box — no box algorithm is chosen for it at all;
* for a **replaced element** such as an image, it lays the child out like any
  other box: the decoded resource's intrinsic size has already been written
  onto the child's `width`/`height` before the layout pass began, so nothing
  is measured here;
* for a **container**, it recursively lays the child out under the given
  constraints and returns the resulting border-box size.

There is no separate intrinsic-measure port behind this one. An earlier
`IntrinsicMeasure` trait was documented here as the leaf road and never had a
production implementation; it is deleted, and the two bullets above are what
the runtime does instead.

Either way the algorithm sees only a `Size`, which keeps it independent of
text shaping, image decoding, and nested-context recursion.

Implementations should be deterministic: the same `(index, constraints)`
should yield the same `Size` within a single layout pass so an algorithm may
measure a child more than once (e.g. once for its natural main size, once for
its cross size) and reason about the results consistently.
