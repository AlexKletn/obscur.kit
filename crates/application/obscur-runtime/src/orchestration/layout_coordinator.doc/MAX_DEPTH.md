The maximum recursion depth of the layout walk.

The retained tree is expected to be acyclic, but the coordinator never trusts
that: a shared or cyclic child link would otherwise recurse forever. This cap
bounds both `measure` and `arrange` so recursion always terminates and never
overflows the stack. It is deliberately generous (far deeper than any
realistic UI tree) so it never truncates a well-formed tree.
