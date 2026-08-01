A headless columns/rows data table with single-column sort (spec §2.6).

`Table` owns its [`Column`] definitions, its `rows` (each row is a [`Vec`] of
cell strings, one per column), and the active [`sort`](Self::sorted_column)
(a column index plus a [`SortDirection`], if any). It bakes in **no** visual
theme: [`view`](Self::view) derives a [`SemanticRole::Table`] element carrying
the sorted column index, a header [`SemanticRole::Row`] child (one
[`SemanticRole::Cell`] per column, carrying its title and sortable flag), and
one row child per data row (one cell child per cell), rendered by the style
layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).

The state machine is [`sort_by`](Self::sort_by): sorting by a sortable column
stable-sorts the rows by that column's cell text ascending; sorting by the
same column again flips the direction.
