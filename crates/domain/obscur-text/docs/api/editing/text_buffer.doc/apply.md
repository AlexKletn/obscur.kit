Applies one editing command, updating the text and selection
deterministically.

Mutating commands ([`Insert`](EditCommand::Insert),
[`Delete`](EditCommand::Delete), [`Replace`](EditCommand::Replace)) push
an [`UndoUnit`] and clear the redo stack; navigation commands
([`Move`](EditCommand::Move), [`SelectAll`](EditCommand::SelectAll),
[`SetSelection`](EditCommand::SetSelection)) only move the selection and
leave the undo/redo stacks untouched.
