A `Vec` moved in. Taken as-is: converting it to an `Arc<[u8]>` would allocate a
second buffer and copy the whole thing.
