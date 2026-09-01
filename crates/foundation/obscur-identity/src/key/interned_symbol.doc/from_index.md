A symbol for a table-backed interner's `index`, lifted into the high
namespace (top bit set) so it can never alias a reserved symbol.

An interner that hands out sequential indices numbers from **zero**, which is
squarely inside the low range the reserved constants occupy: `from_raw(0)`,
`from_raw(1)`, … are attribute and event names, and `layout_keyword`'s
`OVERFLOW_HIDDEN` is `0x0000_1E02`. A sheet that interned enough words reached
that number, and the two became indistinguishable — `overflow-x: sideways`
came out equal to `overflow-x: hidden` and the box clipped. Setting the top
bit is the same partition `named` uses, and it makes the collision unreachable
rather than merely unlikely.

The index survives in the low 31 bits, so `index` gives it back.
