Pure measure pass: computes `node`'s border-box size under `constraints`
without writing anything.

This mirrors the algorithm selection of
[`arrange`](super::arrange_pass::arrange) but discards the child
boxes and returns only the node size. Because it never touches the results
map, it can be called freely from inside an algorithm (via [`RecMeasurer`])
while `arrange` holds a mutable borrow of that map.

`depth` bounds the recursion: at zero it returns [`Size::ZERO`], guaranteeing
termination on a cyclic or pathologically deep tree.
