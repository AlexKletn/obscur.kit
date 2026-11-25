FNV-1a 32-bit hash of `s`, used to derive stable [`InternedSymbol`] ids.
The [`InternedSymbol`] raw value a **property** name has in the desired view
tree, resolved through the shared `name_binding` registry.

This must not be a bare hash. `name_binding::attr_symbol` maps a handful of
reserved names (`class`, `id`, `type`, `src`, `disabled`, `checked`) to fixed
low constants, and every other name to `fnv32(name) | 0x8000_0000`. A bare
`fnv32` therefore disagrees with the real view symbol for every reserved name
and for every other name whose hash does not already have the top bit set —
i.e. it happens to agree about half the time, by luck. Resolving through the
registry is what makes a contract's prop symbol comparable to the
`ViewProperty::name()` a node actually carries.

[`InternedSymbol`]: obscur_identity::InternedSymbol
