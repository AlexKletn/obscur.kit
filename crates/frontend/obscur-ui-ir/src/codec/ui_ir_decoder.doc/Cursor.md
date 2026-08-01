A bounds-checked reading cursor over a byte slice. Every read advances
[`pos`](Cursor::pos) and maps a short read to
[`UiIrDecodeError::UnexpectedEof`].
