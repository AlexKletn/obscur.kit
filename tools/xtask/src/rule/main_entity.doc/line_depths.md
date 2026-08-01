The brace depth each line *starts* at, or `None` when it starts inside a
string literal or a block comment.

String- and comment-aware on purpose: this workspace embeds whole CSS
stylesheets in raw string constants, and a naive brace count over
`crates/facade/obscur-kit/examples/uikit/material_theme.rs` would put the
rest of the file at depth 40.
