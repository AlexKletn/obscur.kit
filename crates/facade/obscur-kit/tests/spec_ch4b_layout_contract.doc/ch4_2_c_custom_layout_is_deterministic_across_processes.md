§4.2: "Custom layout uses restricted deterministic domain APIs for geometry
and constraints."

The identical tree is laid out in two **separately-launched processes** (a
same-process double run cannot expose a `HashMap`/`HashSet` iteration order
randomized per process by `RandomState`, nor an order derived from
allocation addresses). The two serialized `(x, y, w, h)` lists must be
byte-identical.
