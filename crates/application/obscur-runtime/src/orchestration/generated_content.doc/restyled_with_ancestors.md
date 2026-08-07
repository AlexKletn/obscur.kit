The nodes a scoped pass must revisit: everything `dirty` holds, plus every
ancestor of it.

The restyle's own reach rule (subtree plus following siblings) is right for
selector matching, which never reads downwards. Generated content does read
downwards — `:placeholder-shown` asks whether a control's descendants render
text — so the ancestors of an edited node are exactly the extra nodes whose
answer can have changed.
