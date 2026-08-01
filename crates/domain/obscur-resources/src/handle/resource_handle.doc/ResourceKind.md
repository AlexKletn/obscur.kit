The kind of resource a [`ResourceHandle`] refers to (spec §5.15).

`ResourceKind` is a tightly-coupled part of [`ResourceHandle`]: it tags a
handle's id with the category of resource it denotes, and has no meaning
apart from the handle model, so it is colocated here rather than given its
own module. The variants are the resource categories the Resource Hub
manages.
