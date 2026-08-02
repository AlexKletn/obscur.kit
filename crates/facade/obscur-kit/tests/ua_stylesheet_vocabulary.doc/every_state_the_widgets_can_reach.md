One view per widget state that emits a name or class no other state does.

Enumerated by hand because the states are: a `Checkbox` emits `.checked` only
when checked and `.checkbox-bar` only when indeterminate, a `Select` emits
`.select-list` / `.select-option` only while open and `.selected` only with a
selection, and a disabled control is a different frame from an enabled one. A
harvest that built each widget once would report a smaller vocabulary than the
toolkit has, and the guard would then reject live rules.
