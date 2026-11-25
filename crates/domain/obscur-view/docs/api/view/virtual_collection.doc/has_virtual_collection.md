Whether `nodes`, or anything beneath them, contains a virtual-collection
marker (spec §2.11).

[`realize_virtual_collections`]'s early-out: a document that authors no
`<VirtualFor>` pays a single shallow walk and is returned untouched.
