Every compound of a complex selector — the head and each tail step.

`subject()` alone would read only the rightmost compound, so
`checkbox:hover .checkbox-box` would be checked for `.checkbox-box` and never
for `checkbox`: the half of the sheet written as a descendant of a control
would go unguarded.
