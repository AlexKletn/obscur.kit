A `Sized` wrapper over a shared `Arc<[u8]>` pixel buffer so it can back a
peniko [`Blob`] without copying: `Arc<[u8]>` is an unsized fat pointer that
cannot coerce to `Arc<dyn AsRef<[u8]>>`, but `Arc<SharedPixels>` can.
