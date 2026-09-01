A face the host installs under the family `name`, in the cut nearest
`attributes`, or [`None`] when the host has no such family.

Answered per *name*, not per character: an author sheet that says
`font-family: "Figtree"` names a family the application never registered, and
without this the request resolves to nothing and the text is drawn in whatever
the rest of the chain offers — so installing the font would change nothing.

The default implementation answers [`None`], which is exactly the previous
behaviour for a host that only knows how to look a face up by character.
