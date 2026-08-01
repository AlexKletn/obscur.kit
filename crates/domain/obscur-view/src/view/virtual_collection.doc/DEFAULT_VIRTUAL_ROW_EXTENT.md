The default estimated row extent, in logical pixels, a `<VirtualFor>` uses
until its rows are really measured (spec §4.9 *estimated geometry*).

A comfortable single-row height: enough that a normal viewport shows tens of
rows, which is the order [`realize_virtual_collections`] materializes.
