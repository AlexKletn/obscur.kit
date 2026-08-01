A CSS `outline` (a focus ring): ONE stroked path OUTSIDE the node's border
box. It reserves no layout space and — unlike the background/border — must
NOT be clipped by the node's own rounded clip, so it is emitted at the
subtree's EXIT index rather than inside the node's paint unit.
