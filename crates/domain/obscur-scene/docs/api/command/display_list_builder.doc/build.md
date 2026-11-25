Consumes the builder and returns the accumulated [`DisplayList`] **as
is**, in paint order.

This does **not** auto-close open clips or layers: if the builder is
unbalanced, the returned list is unbalanced too. Callers who need the
guarantee should verify [`DisplayList::is_balanced`].
