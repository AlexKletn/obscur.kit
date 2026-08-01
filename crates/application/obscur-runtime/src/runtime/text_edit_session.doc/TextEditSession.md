The runtime's editing session for the focused text field.

Unlike the append-only per-keystroke model it replaces, the runtime is now
the **source of truth** for the focused field's text, caret and selection
(an *uncontrolled* input): a [`TextEditSession`] owns a deterministic
[`TextBuffer`] (spec §5.11) that the keymap in
[`UiRuntime::handle_key`](super::UiRuntime::handle_key) drives with
[`EditCommand`](obscur_text::editing::EditCommand)s. The app is notified of
the *resulting text* after each mutating edit; step-2 rendering reads the
live [`text`](Self::text)/[`selection`](Self::selection) back to paint the
caret and selection highlight.

A session is bound to one [`NodeId`] — the editable node whose ancestor chain
binds [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT). When focus leaves that
node the session is dropped; when it lands on a *different* editable a fresh
session is seeded (see
[`UiRuntime::begin_text_editing`](super::UiRuntime::begin_text_editing)).
