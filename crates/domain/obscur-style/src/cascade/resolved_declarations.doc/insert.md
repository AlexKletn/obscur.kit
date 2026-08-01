Records `value` as the cascade winner for `property`, returning whatever it
displaced.

The priority-less twin of
[`insert_with_priority`](Self::insert_with_priority): use it where the
origin and specificity that won are not being tracked, and nothing will ask
[`priority_of`](Self::priority_of) about this property afterwards.
