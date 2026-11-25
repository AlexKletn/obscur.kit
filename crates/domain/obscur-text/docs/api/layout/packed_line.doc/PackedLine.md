One packed line: its logical-order tagged glyphs, the number of words it holds
(so its inter-word gap count is `word_count - 1`), and whether the line was
closed by a *mandatory* (hard) break. Justification uses `word_count` and
`mandatory_break_after` to decide whether the line stretches to fill the width.

A packed line's advances are only correct once they are measured against the
line's **own** edges: [`retab`](Self::retab) re-expands its tabs from its left
edge and [`hang_trailing`](Self::hang_trailing) trims what hangs off its right
edge. Both are line-local corrections that only exist once the packing has
decided where the line begins and ends, which is why they live with the packed
line rather than with the passes that read its width.
