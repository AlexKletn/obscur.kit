An immutable, versioned snapshot of a computed style: [`StyleSnapshot`]
(§1.6).

A [`StyleSnapshot`] hands a node's [`ComputedStyle`] to heavy downstream
subsystems (layout, paint) together with a [`VersionStamp`], so a consumer can
cheaply tell whether the style it holds is stale — mirroring how the tree
hands out a versioned `TreeSnapshot`. The snapshot owns its style and exposes
read-only accessors; it has no mutators.
