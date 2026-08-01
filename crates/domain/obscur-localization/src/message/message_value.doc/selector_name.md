Returns the name a `select` arm matches against.

Both [`Selector`](Self::Selector) and [`Text`](Self::Text) answer with
their text — a plain string argument is a perfectly good selector — and a
number answers with its digits, so `{n, select, 0{none} other{some}}` is
possible even though `plural` is the right tool.
