Resolves a key press over the focused editable `target` into the edit it
means (spec §5.11, §6.7). [`KeyEdit::Declined`] means the key was not an
edit, so the caller's shortcut / activation path stays byte-identical.

Requires the session to cover `target`'s editable ancestor; a press
elsewhere declines. The keymap:
* a printable [`LogicalKey::Character`] with no command modifier →
  [`EditCommand::Insert`] (replacing any selection);
* `Backspace`/`Delete` → [`KeyEdit::DeleteGrapheme`] backward/forward;
* `ArrowLeft`/`ArrowRight` → grapheme [`EditCommand::Move`], extending the
  selection when `Shift` is held;
* `Home`/`End` → visual-line boundary motion, shift-extendable;
* `ArrowUp`/`ArrowDown` → a wrapped-line move in a multiline field only;
* `Enter` → a newline in a multiline field, a [`KeyEdit::Commit`] in a
  single-line one;
* `Ctrl`/`Meta` + `A` → [`KeyEdit::SelectAll`]; `+Z` → undo,
  `+Shift+Z`/`+Y` → redo (the buffer exposes both).

Any other key — including a `Ctrl`/`Alt`/`Meta` chord that is not one of
the editing chords above, and `Escape`/`Tab` — declines.
