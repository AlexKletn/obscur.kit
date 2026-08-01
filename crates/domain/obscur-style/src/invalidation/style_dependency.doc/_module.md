The per-node style dependency record: [`StyleDependency`] (§4.3, §4.14).

When a node's computed style is resolved it consumes a few typed inputs. This
record captures *which* of those inputs the node actually read, so a later
change to one of them can decide precisely whether the node is affected —
invalidating only affected consumers (§4.14) rather than everything.

# Status: nothing populates this

No production code constructs a `StyleDependency`, because nothing records what
a node's matching consulted: `StyleMatcher::match_element_in` returns the
cascaded declarations and no dependency trace, so there is no moment at which
the `with_*`/`add_*` builders could be called. Its `affected_by_*` gates
therefore have no production consumer either, and it is not what makes today's
style invalidation per-node.

What does: the runtime's `RestyleScope` bounds a `:hover`/`:focus` change
*structurally* — a node's state can only be read while matching its own
subtree and its following siblings' subtrees, because those are the only nodes
`MatchElement`'s `parent()`/`previous_sibling_element()` can reach it from. That
is a sound upper bound and needs no per-node record. This type would narrow it
(a node whose matching never mentioned `:hover` need not be restyled at all),
which is the reason to keep it — but narrowing needs the matcher to trace its
reads first.
