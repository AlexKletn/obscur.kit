The retained node reached by walking child indices down from the tree root.

Fixture views are small and their shape is fixed, so "root → panel → field" is a
complete and stable name for a node — and it is the name the tests were already
using, spelled out as three lines of `tree().get(id).unwrap().child_nodes()[0]`
each time. Written that way the *shape* being asserted is invisible: `NodePath`
puts it in one place, `&[0, 0]`, where a reader can see the depth at a glance.

Every failure panics with the path and the depth that failed, because a silent
`unwrap` in a selector reports the test's *subject* as broken when what is
actually broken is the test's idea of the tree.
