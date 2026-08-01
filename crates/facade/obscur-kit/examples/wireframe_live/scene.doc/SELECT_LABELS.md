The human labels of the select options. Single tokens on purpose: a BARE
`Select` is `display: flex` (label + arrow), and under this engine its flexible
label item resolves to a narrow intrinsic width, so a two-word label ("Option
one") wraps and spills below the box. Single-word options render the field
cleanly with no restyling (the whole point — no author CSS on the control).
