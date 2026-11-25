The granularity of a deletion, i.e. how much text one delete step removes
(spec §5.11 editing engine — deletion semantics).

A [`DeletionUnit`] answers "how much?" for a delete; it pairs with an
[`EditDirection`](crate::editing::EditDirection) in
[`EditCommand::Delete`](crate::editing::EditCommand::Delete) so the same unit
covers both the backward (Backspace) and forward (Delete) forms. When the
selection is non-empty the engine deletes the selection instead and the unit
is ignored; the unit governs the collapsed-caret case.
