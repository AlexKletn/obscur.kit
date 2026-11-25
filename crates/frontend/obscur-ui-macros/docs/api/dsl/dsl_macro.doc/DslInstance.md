The strings and spans one [`DslElement`] contributes to an [`ElementUse`].

The mirror of the file front end's `AstInstance`, and for the same two
reasons: the rule borrows `&str` from storage that must outlive it, and a
[`ViolationAnchor`] has to be resolved back to something the author can be
pointed at — a [`Span`] here, a byte offset there.
