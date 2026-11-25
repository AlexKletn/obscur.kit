A run's glyphs where the command asked for them, as an iterator that knows its
own length.

The domain glyph id IS this font's glyph index — both were shaped against the
same host font — so it is used directly, and the positions come from the domain
and are never re-shaped here. All this adds is the command's origin.

# The return type is the point of the function

`DrawGlyphs::draw` feeds whatever it is given straight into `Vec::extend`, which
reserves the iterator's `size_hint` lower bound in one allocation. A
`Map<slice::Iter, _>` states the exact count, so the reservation is exact; a
`FlatMap` — which is what gathering several commands into one draw would need —
states a lower bound of zero, and the reservation quietly stops happening, one
grow per glyph instead.

That is a property of a type, not of a value, so it is stated as one:
`impl ExactSizeIterator` is a promise the compiler holds, and the shape that
would break it stops compiling here rather than getting slower somewhere else.
Which also means this must stay a named function returning that type — inlined
back into `draw_text`, the claim has nowhere to live but a comment.

Kept out of `draw_text` for the second reason too: a private function is
something a test can call, and the property worth testing —
`len()` before the iterator is walked — is not observable through the scene
`draw_text` writes into.
