The character an authored `options="…"` list is split on: a comma, because a
human writes that list and a comma is what a human writes.

It is emphatically *not* the separator [`Select::view`] emits — that one is
[`Select::OPTION_SEPARATOR`], which no typed label contains. The consequence
of this one, stated rather than discovered: **an option label containing a
comma cannot be authored in markup.** It can be built in Rust, and it survives
there, which is the half that matters — no existing program's options change
when the built-in is registered.

[`Select::view`]: super::Select::view
[`Select::OPTION_SEPARATOR`]: super::Select
