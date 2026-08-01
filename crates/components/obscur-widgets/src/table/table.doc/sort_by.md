Sorts the rows by the cell text of column `column`, returning `true` on
success. Sorting by a fresh column sorts ascending; sorting by the
already-active column flips the direction. The sort is **stable**.
Returns `false` (leaving state untouched) when `column` is out of range or
its column is not [`sortable`](Column::is_sortable).
