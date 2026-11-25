A compiled `style` attribute: the declarations written on one node.

Origin 5 of the cascade (spec §4.14) — above every stylesheet, below a runtime
override. It carries no selector and no specificity, because it did not match
anything: it was written ON the element.

Refcounted, because the same block is usually written on many nodes at once —
a thousand rows of a virtualized list sharing one `style` string compile once
and share the result.
