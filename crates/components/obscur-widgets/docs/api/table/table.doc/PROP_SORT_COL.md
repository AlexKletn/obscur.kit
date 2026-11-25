Interned name of the sorted-column-index property (`-1` = unsorted).

Shares the `0x00D1_NNXX` scheme with `NN = 0A` (table): sort-col=`01`,
sortable=`02` (on header cells). Slot `00` was a private `role` property
carrying the text `table`/`row`/`cell`; the parts name themselves in HTML's
own vocabulary now — `table`, `tr`, `th`, `td` under the reserved
`LOCAL_NAME` symbol — so CSS can see them, and nothing reads `00`.
