Whether this field is **readonly** (the HTML `readonly` attribute, WHATWG
HTML §attr-input-readonly). **Distinct from — and independent of —
[`disabled`](Self::disabled):** a readonly field can still receive focus, a
caret, and a selection (it is selectable / copyable), but its text cannot be
*edited* (no insert/delete, no IME commit, no paste — [`apply`](Self::apply)
drops mutating edits, and the runtime gates only the mutation path on the
reserved `READONLY` attribute this emits). A field may be neither, one, or
both flags; the default (`false`) is a normal editable field —
byte-identical to before.
