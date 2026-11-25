The exported contract for one prop of a component (spec §2.13).

A prop is referenced entirely by interned handles: its own name and the
name of its declared value type. The value type itself is modelled and
interned elsewhere; this contract only names it, so no value/type system is
duplicated here. Whether the prop is required and whether the component
supplies a default are recorded as plain flags, because there is not yet a
value model that could carry the default itself.
