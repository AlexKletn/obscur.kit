Installs the grammar that reads a custom property's declared text for the
property it is substituted into.

`KitBootstrap` wires this alongside the `style` attribute's compiler. Until
one is installed a `var()` naming a custom property declared as plain text
computes to `unset` rather than to a value read half-way — a runtime whose
custom properties were handed in as typed values is unaffected.
