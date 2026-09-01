The two `overflow` axes of one box, read together (css-overflow-3 §3).

`overflow-x` and `overflow-y` are the storage — `overflow` is a shorthand the
CSS compiler expands into the pair — so every phase that used to ask one
question of one property now has two values to reconcile. The reconciliation is
a spec rule rather than a convention (`visible` beside a scrolling sibling
computes to `auto`), and it is here, once, so the scroll path, the clip and the
hit test cannot disagree about what a box does.
