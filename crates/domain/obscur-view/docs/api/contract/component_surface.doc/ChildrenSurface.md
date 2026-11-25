Which children a [`ComponentSurface`](super::ComponentSurface) permits, named by string tag.

This mirrors [`ChildContract`](crate::contract::ChildContract) in the string
(tooling) world: `ChildContract` decides membership by
[`ComponentId`](obscur_identity::ComponentId) identity, whereas the tooling
surface must decide it by the tag *name* a `.okui` source writes, since the
`.okui` compiler has only strings to work with.

The three cases mirror `ChildContract::{Any, None, Only}`:

- [`Any`](ChildrenSurface::Any) — any child element is allowed (no
  restriction);
- [`None`](ChildrenSurface::None) — a leaf: no children. This used to be
  *unenforceable*: the string contract the `.okui` compiler consumed held
  children in a list whose empty state already meant "any child", so the bridge
  into it had to flatten `None` into `Any` and a declared leaf accepted children
  anyway. The compiler's contract now names the three states, and the
  declaration crosses intact;
- [`Only`](ChildrenSurface::Only) — only the listed child tags are allowed.
