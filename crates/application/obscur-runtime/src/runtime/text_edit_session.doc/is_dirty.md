Returns whether the field's live text differs from the last committed
value — i.e. whether a commit (blur / single-line `Enter`) should fire a
`change` event (WHATWG HTML §4.10.5.5: `change` fires only when the value
actually changed since it was last committed / since focus).
