Applies an [`EditCommand`] to the underlying buffer — the way all text
edits (typing, deletion, navigation, IME/programmatic edits) flow in.

A no-op while [`disabled`](Self::is_disabled) (the field is fully inert).
While [`readonly`](Self::is_readonly) (but not disabled) only the
**text-mutating** commands (insert / delete / replace) are dropped — caret
and selection navigation still apply, matching the HTML `readonly` model
(WHATWG HTML §attr-input-readonly: focusable + selectable, not editable). A
normal field applies every command (byte-identical to before).
