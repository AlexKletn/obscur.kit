The owned strings one parsed [`Element`] contributes to an [`ElementUse`],
plus the byte offset each part is reported at.

It exists because the rule borrows `&str` while the AST hands out `String`:
the storage must outlive the borrow, and the offsets must travel with it so a
[`ViolationAnchor`] can be resolved back to a source position.
