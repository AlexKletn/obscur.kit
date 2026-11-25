Where every grid item landed, and how large the grid had to become to hold
them (css-grid-2 §8.5).

The four vectors are parallel to the child list: item `i` occupies the
`row_span[i] × col_span[i]` block anchored at `(col[i], row[i])`. `n_cols`
and `n_rows` are the counts *after* implicit tracks: a definite line past the
template, a span wider than the template, and a column-flow item that ran off
the end all grow them.
