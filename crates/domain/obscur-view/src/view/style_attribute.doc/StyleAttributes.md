The CSS-relevant identity of an element view node: its local name, `id`, and
class list, extracted from a node's [`ViewProperty`] bindings.

Names are kept as owned strings (not interned) because CSS selector names
are matched as strings — the authored stylesheet and the tree intern names
in separate tables, so string comparison is the only coherent match.
