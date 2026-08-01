Returns `true` when any computed value on this node contains a `ch` term.

A caller that must *measure* the `0` glyph to resolve `ch` asks this
first, so the overwhelmingly common "no `ch` anywhere" node costs an
allocation-free scan instead of a shaping call. See
[`PropertyValue::uses_ch`].

Custom properties are not scanned, and need not be: a `var()` reference is
substituted during the cascade, so `--w: 20ch; width: var(--w)` reaches
this type as a computed `width` of `20ch` and is found on the `values`
pass. A custom property that is never referenced resolves nothing.
