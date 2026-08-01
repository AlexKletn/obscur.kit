The granularity of a caret movement, i.e. how far one navigation step travels
(spec §5.11 editing engine — word/line/document navigation, Home/End).

A [`NavigationUnit`] answers "by how much?" for a move or shift-extend: it
pairs with a direction (and an *extend* flag) in
[`EditCommand::Move`](crate::editing::EditCommand::Move) to describe every
caret motion a keymap can bind. It carries no direction of its own — moving
left or right by a word both use [`Word`](Self::Word).
