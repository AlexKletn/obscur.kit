Resolves the computed `writing-mode` keyword to a [`WritingMode`], defaulting to
[`WritingMode::HorizontalTb`] (the CSS initial).

Compares the interned symbol against the reserved [`layout_keyword`]
`WRITING_MODE_*` handles; an absent or unrecognized value maps to
[`WritingMode::HorizontalTb`] (which also covers `WRITING_MODE_HORIZONTAL_TB`),
so a box that never sets `writing-mode` is byte-for-byte identical to before.
This is the same value the style crate resolved to fold logical box-model
longhands onto physical ones; flow layout re-reads it because it also needs the
geometric half (which physical axis and origin it stacks children along).
