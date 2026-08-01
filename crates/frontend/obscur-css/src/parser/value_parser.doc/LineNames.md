A bracketed line-name group `[ a b … ]` from a grid track list
(`grid-template-columns: [sidebar-start] 200px [main]`). The names are the
`<custom-ident>`s inside the brackets, in author order; an empty group
(`[]`) carries no names. Only the grid track-list compiler reads this; any
other property degrades to an unsupported-feature diagnostic (spec §4.13).
