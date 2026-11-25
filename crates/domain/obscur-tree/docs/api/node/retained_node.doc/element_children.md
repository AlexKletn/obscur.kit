The element-only projection of [`children`](Self::children), in document
order — the **component tree** a host observes through
[`children`](Self::children) (the accessor). Text and synthetic parts are
content, not component-tree children (spec §4.16), so they are excluded
here. Kept in step with `children` by the arena, which is the only thing
that knows each child's [`NodeKind`]; a bare node cannot derive it alone.
