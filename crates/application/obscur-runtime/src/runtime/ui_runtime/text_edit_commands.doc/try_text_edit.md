Attempts to consume a key press as an **edit** into the runtime-owned
editing session for the focused editable `target` (spec §5.11, §6.7).
Returns whether the key was consumed (so [`handle_key`](Self::handle_key)
does *not* also treat it as a shortcut or activation).

[`KeyEditResolver::resolve`] owns the keymap — which key means which edit
against a laid-out field, and which keys the field does not claim — and its
documentation is the written account of it. This is the acting half: it
applies the resolved edit through [`apply_edit`](Self::apply_edit), which is
where the readonly gate, the `on-input` dispatch and the frame request live.
After a mutating edit the field's `on-input` handler is called with the
resulting text; every consumed edit requests an input frame.
