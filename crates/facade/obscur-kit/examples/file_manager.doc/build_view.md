Builds the whole UI from the current state. Every `{…}` in **body** position
below reads a real local of this function — `{path}`, `{status}` — and that is
what the inline macro buys over a compiled `.okui` file.

A `{…}` in an `on:` attribute is the other thing the same syntax means: a
command *name*, hashed into the symbol the runtime dispatches on. `goParent`
below is not a binding and there is no Rust item of that name anywhere — only
the `command("goParent")` registration in `main`.
