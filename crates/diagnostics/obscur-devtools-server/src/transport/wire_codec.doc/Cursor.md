A bounds-checked reading cursor over a byte slice. Every read advances past
the bytes consumed and maps a short read to [`WireError::UnexpectedEof`].
