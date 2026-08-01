The **readonly** field's command — no handler needed: a readonly field is
focusable and its text selectable, but its value can never change, so its
`change`/`input` never fire (WHATWG HTML §attr-input-readonly).
