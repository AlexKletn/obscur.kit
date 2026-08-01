Checks a §2.11 collection tag. Both collection constructs this pipeline knows
— `<For>` and `<VirtualFor>` — are honoured, so every tag is accepted.

`Ok(())` for every tag: `<For>` materializes per item, `<VirtualFor>`
realizes a viewport-bounded window (spec §4.9), and no other tag is this
rule's business. The seam remains so a future §2.11 constraint has one home
both front ends share; see the module docs.

# Errors

Currently never returns an error — both §2.11 constructs are compiled. The
[`Result`] shape is kept for the front ends that call it and for the future
constraint the module docs describe.
