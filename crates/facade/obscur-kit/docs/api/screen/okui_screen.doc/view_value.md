The [`ViewValue`] a **non-text** IR literal is worth — the same mapping the
bridge makes for a property, needed here because rendering goes through
[`ScreenData::render`].

Text is handled by the caller: cloning a `Box<str>` is not possible in a `const
fn`, and a literal string needs no mapping to read as itself.
