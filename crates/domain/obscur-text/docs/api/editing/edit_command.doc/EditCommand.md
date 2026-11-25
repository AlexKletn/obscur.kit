A single editing operation submitted to the editing engine (spec §5.11
editing engine).

[`EditCommand`] is the engine's input vocabulary: a platform keymap resolves
key chords, clipboard actions, and IME events into these commands, which the
engine applies to the buffer and selection. The command names *what* to do;
resolving it against the current text and selection — and producing the
invertible [`EditTransaction`](crate::editing::EditTransaction) — is the
engine's job, not the command's.
