Registers `handler` as the application logic to run when the focused
editable bound to `command` produces a text edit (spec §6.7), replacing
any handler previously registered for that command.

The `command` symbol is app-owned and must be the **same** symbol the
application binds its `on-input` event to in the View (via
[`EVENT_INPUT`](obscur_view::view::EVENT_INPUT) — an
[`Input`](../../obscur_widgets/input/struct.Input.html) does this). The
runtime now *owns* the focused field's editing session (caret, selection
and undo; spec §5.11): each mutating keystroke is applied to that session's
buffer and `handler` is called with the whole **resulting text**
(`FnMut(&str)`), which the app mirrors into its *own* state (typically an
`Rc<RefCell<String>>` the reactive view builder reads back). Caret-only
motions (arrows/Home/End/select) do not call the handler. Registering
does not itself schedule a frame.
