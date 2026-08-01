Creates a fresh entry for a newly requested resource (spec §5.15).

The entry starts [`ResourceState::pending`] at [`ResourceVersion::initial`]
with a reference count of `1`.
