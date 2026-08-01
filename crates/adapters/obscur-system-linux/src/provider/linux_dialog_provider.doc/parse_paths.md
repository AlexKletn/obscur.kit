Parses `zenity` file-selection stdout into individual path strings (spec §6.9).

`zenity --multiple` separates selections with `|`, and single selections carry
a trailing newline; this splits on both, trims, and drops empty fragments.
