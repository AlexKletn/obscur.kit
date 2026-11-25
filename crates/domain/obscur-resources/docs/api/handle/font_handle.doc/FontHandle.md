A type-safe handle to a font resource (spec §5.15).

`FontHandle` is a distinct newtype over [`ResourceId`] so that, at compile
time, it can only denote a font. It converts to and from the generic
[`ResourceHandle`] via [`Self::into_handle`] and [`Self::from_handle`]. Like
all handles it is a pure identifier: cheap, `Copy`, and owning no data.
