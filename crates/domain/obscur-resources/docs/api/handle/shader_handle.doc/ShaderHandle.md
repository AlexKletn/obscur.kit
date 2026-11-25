A type-safe handle to a shader program resource (spec §5.15).

`ShaderHandle` is a distinct newtype over [`ResourceId`] so that, at compile
time, it can only denote a shader. It converts to and from the generic
[`ResourceHandle`] via [`Self::into_handle`] and [`Self::from_handle`]. Like
all handles it is a pure identifier: cheap, `Copy`, and owning no data.
