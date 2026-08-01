Flex has its own sizing code and had the same defect. Measured on a
**fixed-width** flex parent, which isolates the percentage rule from a
separate flex bug: a flex container clamps its own border box to the space
available instead of overflowing it, so `width: 100%` + padding comes out
1000 where the block path correctly gives 1040. That is not this fix's
business and is deliberately not exercised here.
