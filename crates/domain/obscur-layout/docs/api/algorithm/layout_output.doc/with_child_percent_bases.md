Attaches the containing block each child's percentages resolve against,
parallel to the child boxes.

Only an algorithm whose children have a containing block *other* than its
own content box needs this — in practice grid, whose items are contained
by their grid areas. See the type-level note on [`LayoutOutput`].
