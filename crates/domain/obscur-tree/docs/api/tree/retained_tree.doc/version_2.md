Returns the tree's current version stamp.

The version advances on every structural mutation
([`alloc`](Self::alloc), [`remove`](Self::remove),
[`set_root`](Self::set_root)), so two [`TreeSnapshot`](super::TreeSnapshot)s
taken at different states carry different versions.
