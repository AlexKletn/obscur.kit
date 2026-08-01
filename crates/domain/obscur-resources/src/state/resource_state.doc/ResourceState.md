The load lifecycle state of a resource (spec §5.15).

Models the mutually exclusive stages a resource passes through as it is
requested, loaded/decoded/compiled, and either becomes usable or fails:
*pending* → *loading* → *ready* or *failed*. This is pure data; state
transitions are driven by the [`crate::hub`] `ResourceHub` (a later sub-step),
not by this type. A [`ResourceState::Ready`] or [`ResourceState::Failed`]
carries a [`ResourceVersion`] so a newer reload can supersede it.
