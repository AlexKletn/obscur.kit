A borrowed snapshot of the active IME preedit spliced into the editing field
for DISPLAY (spec §5.12 IME).

The preedit is *not* committed text: it is the uncommitted composition string
shown inline over the composition anchor `[at, end)` (byte offsets into the
field's buffer text) and rendered distinctly (an underline), but it is
**never** inserted into the buffer until the composition commits. The compose
phase splices `text` into the buffer text only to paint it, **replacing** the
anchor span `[at, end)` so a composition started over a non-empty selection
shows just the preedit where the selection was — not the preedit alongside the
still-selected text (spec §5.12). When the anchor is collapsed (`at == end`,
the common no-selection case) the splice is a pure insertion at `at`.
