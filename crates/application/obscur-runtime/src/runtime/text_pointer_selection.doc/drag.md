While a drag is armed, the selection from the press `anchor` to the
pointer's current byte offset over the **same** field — live drag-selection
(spec §5.11).

The offset is resolved at `point` against the anchored field (clamped to
that field's text regardless of where the pointer currently is, mirroring
how [`offset_at`](Self::offset_at) clamps before/after the run), then the
selection is the range `[anchor, current]` — anchor fixed, focus tracking
the pointer, so dragging left of the anchor selects backward. [`None`] when
the field's text box has gone.
