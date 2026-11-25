An opaque identifier for a style scope (§4.15).

This is a newtype over a `u32`: values are only meaningful relative to the
allocator that hands them out. `Copy + Eq + Hash + Ord` so it can key maps
and order deterministically.

[`ScopeId::GLOBAL`] is a reserved value denoting the root / global
application scope; it is handed out by convention rather than being special
to this type.
