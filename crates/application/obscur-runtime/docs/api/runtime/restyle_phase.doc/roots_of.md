Returns the restyle roots implied by the reconciliation plan, or `None`
when the plan cannot be bounded.

An update contributes its own node. Positional and relational dependencies
conservatively invalidate the parent on child-list edits. For sibling
combinators, the added roots are the surviving children that can match the
right-hand compound's mandatory class or ID. An unkeyed compound treats all
children as candidates. Insertions and moves also contribute their own nodes.
Departed nodes are pruned separately.
