The `writing-mode` property (Keyword-valued: `horizontal-tb` (initial) /
`vertical-rl` / `vertical-lr`; INHERITED — css-writing-modes-4 §3). See the
reserved `WRITING_MODE_*` handles in [`crate::property::layout_keyword`].
The block axis and the inline axis rotate around it: it is the second input
(beyond `direction`) to [`physical_under`](Self::physical_under), which maps
`block-start` to the physical top / right / left per §6.2, and it selects
the physical axis normal flow stacks children along.
