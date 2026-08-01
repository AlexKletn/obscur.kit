The literal content, for a text node — empty for every other node.

Text is the only evidence that survives a *component* render: what a
component puts on screen is a subtree the application never authored, so
"did it run?" can only be asked of its output. `src/badge.rs` asks it.
