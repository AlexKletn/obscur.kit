§4.7: "Hit testing accounts for:" / "- clipping;"

A 100x100 `overflow: hidden` parent holds a 300x300 child. (150, 50) is
inside the child's box but outside the parent's clip, so neither Parent nor
Child may appear in the chain.
