Resolving §4.16 `::before` / `::after` generated content into paint-only runs.

A matched pseudo-element rule (`.badge::before { content: "…" }`) paints a
string before / after the originating element's own content, without the
generated box ever becoming a tree node, an accessibility node, or a hit-test
target.

[`resolve_generated_content`] returns, per node, the runs the scene
coordinator paints inside that node's own paint unit — so a `::before` run
sorts before the element's real text child in paint order.

The pass is written so that a sheet set which authors no decoration at all —
overwhelmingly the common case — costs one visit per rule and nothing per
element: [`PseudoRuleIndex`] answers "can any rule here produce a `::before`?"
once, ahead of the walk, instead of the walk answering it again for every
element by matching every selector of every rule. A scoped pass narrows it
further, to the nodes an incremental restyle actually touched.
