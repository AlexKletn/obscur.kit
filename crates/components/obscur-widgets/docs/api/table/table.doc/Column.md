A single column definition in a [`Table`](super::Table).

Each column carries a display `title` and whether it is
[`sortable`](Self::is_sortable). Non-sortable columns are ignored by
[`Table::sort_by`](super::Table::sort_by).
