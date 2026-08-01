Explicit, author-provided stable identity for a keyed child in the desired
view tree.

Keyed collections (`For`, conditionals, spec §2.11) require a mandatory
stable key so a child preserves its retained identity across reconciliation
even when siblings are inserted, removed, or reordered (spec §2.2). The key
lives on the desired node as plain data; it never references a retained-tree
`NodeId`.

This is a thin newtype over [`StableKey`] so the view layer owns its own
vocabulary type while reusing the foundation identity contract.
