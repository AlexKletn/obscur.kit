A face that really parses.

`vello::DrawGlyphs::draw` unwraps `FontRef::from_index`, so a blob of three
bytes panics inside Vello rather than failing a test. This is the file the
toolkit's own bootstrap installs as the sans-serif generic, and the one the
keyboard draws its legends with.
