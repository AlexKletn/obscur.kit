What [`lower_custom_value`] answered, by property and then by declared text.

A `RefCell` because the port hands the resolver a `&self` — the resolver reads
a grammar, it does not own one — and this is a single-threaded runtime: the
parser is reached through an `Rc`, so no second thread can hold this borrow.
