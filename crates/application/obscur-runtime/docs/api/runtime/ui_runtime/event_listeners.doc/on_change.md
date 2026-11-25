Registers `handler` as the application logic to run when the focused
editable bound to `command` **commits** its value — the toolkit's `change`
event (WHATWG HTML §4.10.5.5 *Common input element APIs*: `change` fires
when the user commits an edit, i.e. on blur or, for a single-line `input`,
on `Enter`), replacing any handler previously registered for that command.

The `command` symbol is app-owned and must be the **same** symbol the
application binds its `on-change` event to in the View (via
[`EVENT_CHANGE`](obscur_view::view::EVENT_CHANGE) — an
[`Input`](../../obscur_widgets/input/struct.Input.html) binds it). Unlike a
per-keystroke [`on_text_input`](Self::on_text_input) handler, a change
handler runs **once per edit session** — only when the value actually
changed since focus — and receives the field's **committed text**
(`FnMut(&str)`). Because it is a distinct registry from `on_text_input`, an
app can register both on the same field command and receive per-keystroke
`input` *and* a single commit `change` without conflation. Registering does
not itself schedule a frame.

(A non-editable checkbox/radio/select fires its `EVENT_CHANGE` command
through the *activation* path on click — see
the activation mapping a click resolves through — so those keep
using [`on_command`](Self::on_command); this registry is for editable
text-field commits.)
