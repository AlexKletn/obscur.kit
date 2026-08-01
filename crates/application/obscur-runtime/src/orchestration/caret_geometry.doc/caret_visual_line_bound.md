The byte offset of the **visual line** boundary — line start (`to_end =
false`, Home) or line end (`to_end = true`, End) — of the line the caret byte
`index` sits on, resolved against the SAME wrapped
[`ParagraphLayout`] the caret paints
from (spec §5.11 "Home/End are visual-line operations").

Home/End are **visual-line**, not paragraph, operations: on soft-wrapped text
they stop at the wrap boundary, not at a `\n` or the buffer end. The caret's
line is resolved downstream ([`line_of_byte`]), so a caret
at a wrap boundary belongs to the following line; Home then returns that
line's content start and End its range end (the next line's start, or the text
end on the last line). A single-line / hard-`\n`-only field yields the same
offsets its logical line boundaries did — the two diverge only under soft
wrapping, which is the behaviour this resolves.
