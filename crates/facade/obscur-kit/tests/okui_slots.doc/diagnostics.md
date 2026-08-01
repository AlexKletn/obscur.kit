Compiles `source` through the real pipeline with `Card`'s derived surface
registered, returning every diagnostic's message.

`ready` and `rows` are defined so a document may condition and iterate; the
name resolver is a different pass and its verdict is not what is under test.
