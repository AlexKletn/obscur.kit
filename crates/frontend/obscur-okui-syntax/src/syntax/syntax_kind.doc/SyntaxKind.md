The kind of any element in the lossless syntax tree (spec §2.8).

`SyntaxKind` is a superset of [`TokenKind`]: it mirrors every lexical token
kind (the *leaf* kinds) and adds the *node* kinds that the parser uses to
group tokens into structure (elements, tags, attributes, control flow).

A tree is therefore uniform — both interior nodes and leaves are labelled
with a `SyntaxKind` — while [`is_token`](Self::is_token) /
[`is_node`](Self::is_node) recover the distinction.
