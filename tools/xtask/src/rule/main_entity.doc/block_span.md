How many lines the block opened at `start` spans, counting `start` itself.

Measured off the brace depths rather than the text, so a `where` clause, an
attribute or a doc comment inside the block cannot confuse it. A declaration
with no block (`pub struct Foo;`) spans one line.
