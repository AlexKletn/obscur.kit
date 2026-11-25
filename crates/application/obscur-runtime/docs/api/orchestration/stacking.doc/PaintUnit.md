Paint commands for one node or one subtree bracket.

The compositor sorts by resolved stacking band, visual pre-order index,
and command phase. A context's background precedes its members. Brackets
owned by a context can close in the last band of that context.
