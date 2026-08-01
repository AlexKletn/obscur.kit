The keyboard as a command surface: routing to focus, shortcuts, the §6.7 unified
`Command`, and Enter/Space activation.

Eleven tests about one claim — that a key press and a click are two *surfaces*
over one command model, not two parallel vocabularies. A key routes to the
focused node or, with nothing focused, nowhere at all: it resolves no command,
runs nothing and schedules no frame, so an unfocused window costs a keystroke
nothing. Modifiers are held state rather than part of the chord that arrives:
a bare Ctrl press resolves no command, folds into the held set, and the S that
follows resolves Ctrl+S; releasing Ctrl unresolves it again.

`register_command` is the §6.7 half. One descriptor carries its own shortcut, so
registering it binds the chord — no separate `bind_command` — and the one
`CommandState` on that descriptor gates **every** surface: disabled, the router
still resolves the chord (the state is what refuses it, not the binding) and
neither the shortcut nor the click runs the handler; re-enabled, both work again.
A symbol that was never registered is unconstrained, which is what keeps a
raw-symbol application working.

Activation is the other half, and it is what makes a button a button. Enter on a
focused node fires its `on-click` binding with no shortcut bound anywhere, Space
fires an `on-change` binding on a checkbox-like node, a click resolves click
*else* change, and an activation whose command has no registered handler is a
no-op rather than a panic. `handle_key_is_deterministic` is the whole path run
twice from a fresh runtime, asserting the two dispatches are equal.

Two tests about `focus_accepts_text` — the question a host asks before raising an
on-screen keyboard — stayed in `ui_runtime.rs` with the text-editing theme they
share a fixture and a documented pairing with.
