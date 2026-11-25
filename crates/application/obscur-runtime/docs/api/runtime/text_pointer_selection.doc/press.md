Resolves a press at `point` on `target` into the selection it makes, given
how many clicks the pointer has accumulated (spec §5.11).

Resolves `target`'s editable ancestor and answers only when the active
editing session already covers it (it was just (re)seeded by the runtime's
focus sync). One click places a caret and arms a drag; two select the word
under the pointer; three or more select the line. A press on a
non-editable — or on one whose text box is not laid out — is
[`TextPress::Ignored`] and leaves the session byte-identical.
