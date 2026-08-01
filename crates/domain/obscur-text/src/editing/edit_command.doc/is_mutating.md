Returns whether applying this command mutates the buffer's text.

[`Insert`](Self::Insert), [`Delete`](Self::Delete), and
[`Replace`](Self::Replace) change text and therefore produce an
[`EditTransaction`](crate::editing::EditTransaction);
[`Move`](Self::Move), [`SelectAll`](Self::SelectAll), and
[`SetSelection`](Self::SetSelection) only move the selection.
