Returns the root plus all transitive descendants in deterministic
pre-order (depth-first, children in link order).

Only ids that resolve to live nodes in `tree` are included, so a dangling
child link is silently skipped. A cyclic link cannot cause a loop: each
node is emitted at most once. If the root itself is not live, the result
is empty.
