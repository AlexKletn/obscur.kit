An immutable, versioned view of a node's computed style.

Wraps an owned [`ComputedStyle`] plus the [`VersionStamp`] it was produced at.
It is read-only: consumers observe the style and compare version stamps but
never mutate the snapshot.

Because [`ComputedStyle`] may carry `f32` data, this derives [`PartialEq`] but
not [`Eq`]/[`Hash`].
