§7.6, verbatim: "Inspect mode uses the same hit-testing engine as runtime and
can highlight bounds."

Four probes at a 100x40 box rotated 45° about its centre, each of which
inspect and the runtime must answer identically:

* **Q (50, 20)** — the rotation's fixed point, on the shape: a hit.
* **P (5, 65)** — inside the rotated AABB, off the shape: a miss. On its
  own it proves little, being outside the untransformed rect too.
* **R (2, 2)** — *inside* the layout rect but off the rotated shape. A hit
  test that ignores the transform reports the box here; a correct one does
  not, since the point inverse-rotates to local (3.3, 41.2).
* **S (92.4, 41.2)** — *outside* the layout rect but on the shape (local
  (95, 5)). A hit test that ignores the transform finds nothing here; a
  correct one finds the box.

R and S are what make this test an oracle for §7.6 rather than a
restatement of "inspect returns something": they are the two points where
the layout rect and the painted shape disagree, so inspect can only match
the runtime by going through the same transform-aware engine.
