The **live** capture target for `pointer`: its captor, or [`None`] when it
is uncaptured — releasing a capture whose node has since left `tree`.

§2.4 releases a capture on node removal. Rather than hook every path that
can remove a node (reconciliation, portal teardown, an error boundary
unmounting a subtree), the capture is validated where it is used: a captor
that is no longer in the tree releases here and the event falls back to
hit-testing. The observable rule is the same, and it cannot be defeated by
a removal path nobody remembered to hook.
