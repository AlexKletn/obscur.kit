Resolving §4.16 `::before` / `::after` generated content into paint-only runs.

A matched pseudo-element rule (`.badge::before { content: "…" }`) paints a
string before / after the originating element's own content, without the
generated box ever becoming a tree node, an accessibility node, or a hit-test
target.

[`resolve_generated_content`] returns, per node, the runs the scene
coordinator paints inside that node's own paint unit — so a `::before` run
sorts before the element's real text child in paint order.
