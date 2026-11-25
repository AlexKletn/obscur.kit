Decodes an `<input>` / `<textarea>` node's properties.

The value is read from **two** sources, in order, because two front ends
state it two different ways:

1. [`Input::PROP_VALUE`], the widget's own value property. Only
   [`Input::view`] emits it, and it is emitted on *every* node the builder
   makes — including the empty ones. Reading it first is what makes mounting
   idempotent on the Rust-API path, and it is also what stops a stale
   authored attribute from resurrecting text the buffer no longer holds.
2. Otherwise the authored [`VALUE`](Self::VALUE) attribute — markup's
   `value="…"`, HTML's *default value* for the control.

Nothing here reports: every prop is defaulted, so a bare `<input/>` decodes
to an empty field rather than to a props failure.

[`Input::PROP_VALUE`]: super::Input
[`Input::view`]: super::Input::view
